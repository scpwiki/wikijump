//! Runtime-backed Wikidot module expansion.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::compat::CompatHtmlFragments;
use super::compat::text_fragments::CompatTextFragments;
use super::diagnostics::{
    CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace, StageGuard,
};
use super::list_pages::{CountPagesExpansionOptions, ListPagesRuntimeDisplay};
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{
    wikidot_module_argument, wikidot_module_arguments,
    wikidot_module_arguments_ignoring_bare_flags,
};
use super::native_list_context::collect_unproven_scope_ranges;
use super::percent_encoding::percent_encode_path_segment;
use super::runtime_page_queries::find_viewable_list_pages_rows;
use super::service::{
    MAX_LISTPAGES_RENDER_SCAN_ROWS, PAGECALENDAR_MODULE_REGEX, RATE_MODULE_REGEX,
    RATEDPAGES_MODULE_REGEX, REGISTRY_MODULE_REGEX, RenderService, TAGCLOUD_MODULE_REGEX,
    escape_list_pages_html_attr, escape_list_pages_html_text, render_clone_module,
    render_members_module_placeholder, render_new_page_module,
    render_read_only_rate_module,
};
use super::url_arguments::UrlArguments;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DateSelector,
    FoundPageFields, IncludedCategories, OrderBySelector, OrderProperty,
    PageParentSelector, PageQuery, PageTypeSelector, PaginationSelector, RangeSelector,
    ScoreSelector, TagCondition,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::score::ScoreValue;
use crate::services::settings::PageRatingType;
use crate::types::Reference;
use crate::types::{Action, Permission, Resource};
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;

const TAG_CLOUD_DEFAULT_LIMIT: usize = 50;
const TAG_CLOUD_DEFAULT_TARGET: &str = "system:page-tags";
const TAG_CLOUD_DEFAULT_WIDTH: u16 = 300;
const TAG_CLOUD_DEFAULT_HEIGHT: u16 = 300;
const TAG_CLOUD_FONT_UNIT_ERROR: &str =
    "Format for minFontSize and maxFontSize must be the same (px, em, pt or %).";
const TAG_CLOUD_COLOR_ERROR: &str = "Unsupported color format. Use \"RRR,GGG,BBB\" for Red,Green,Blue each within 0-255 range.";
const PAGE_CALENDAR_CATEGORY_ERROR: &str = "The requested categories do not (yet) exist.";

#[derive(Clone, Debug, PartialEq, Eq)]
enum PageCalendarCategorySelector {
    All,
    Names(Vec<String>),
}

#[derive(Clone, Debug)]
struct PageCalendarArguments {
    categories: PageCalendarCategorySelector,
    category_url_value: Option<String>,
    tags_url_value: Option<String>,
    selected_date: Option<String>,
    target_page: String,
    url_attr_prefix: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PageCalendarExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) url: UrlArguments<'a>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SecondaryRuntimeModuleExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) url: UrlArguments<'a>,
    pub(super) trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
}

#[derive(Debug, FromQueryResult)]
struct PageCalendarCategoryRow {
    category_id: i64,
    slug: String,
}

#[derive(Debug, FromQueryResult)]
struct PageCalendarPageRow {
    page_id: i64,
    page_category_id: i64,
    created_at: time::OffsetDateTime,
    tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TagCloudSize {
    value: f32,
    unit: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TagCloudColor {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Clone, Debug)]
struct TagCloudArguments {
    mode_3d: bool,
    min_font_size: TagCloudSize,
    max_font_size: TagCloudSize,
    min_color: TagCloudColor,
    max_color: TagCloudColor,
    limit: usize,
    target: String,
    category: Option<String>,
    show_hidden: bool,
    url_attr_prefix: Option<String>,
    skip_category_from_url: bool,
    width: u16,
    height: u16,
    error: Option<&'static str>,
}

#[derive(Debug, FromQueryResult)]
struct TagCloudPageTags {
    page_id: i64,
    page_category_id: i64,
    tags: Vec<String>,
}

#[derive(Clone, Debug)]
struct TagCloudTag {
    name: String,
    count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RatedPagesOrder {
    RatingDesc,
    RatingAsc,
    DateCreatedDesc,
    DateCreatedAsc,
}

#[derive(Clone, Debug)]
struct RatedPagesArguments {
    category: Option<String>,
    order: RatedPagesOrder,
    min_rating: Option<i64>,
    max_rating: Option<i64>,
    limit: usize,
    comments: bool,
}

fn render_join_module(head: &str) -> String {
    let button = wikidot_module_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Join");
    let class = wikidot_module_argument(head, "class")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("join-box");
    format!(
        concat!(
            r#"<div class="{class}">"#,
            r#"<a href="javascript:;" onclick="WIKIDOT.page.listeners.join(event, 'unified')">{button}</a>"#,
            "</div>",
        ),
        class = escape_list_pages_html_attr(class),
        button = escape_list_pages_html_text(button),
    )
}

fn parse_rated_pages_arguments(head: &str) -> Option<RatedPagesArguments> {
    let parsed = wikidot_module_arguments_ignoring_bare_flags(head)?;
    let mut arguments = RatedPagesArguments {
        category: None,
        order: RatedPagesOrder::RatingDesc,
        min_rating: None,
        max_rating: None,
        limit: 10,
        comments: false,
    };

    for argument in parsed {
        let value = argument.value.trim();
        match argument.key.to_ascii_lowercase().as_str() {
            "category" => {
                arguments.category = (!value.is_empty()).then(|| value.to_owned());
            }
            "order" => {
                arguments.order = match value.to_ascii_lowercase().as_str() {
                    "rating-asc" | "rate-asc" => RatedPagesOrder::RatingAsc,
                    "rating-desc" | "rate-desc" => RatedPagesOrder::RatingDesc,
                    "date-created-asc" => RatedPagesOrder::DateCreatedAsc,
                    "date-created-desc" => RatedPagesOrder::DateCreatedDesc,
                    _ => RatedPagesOrder::RatingDesc,
                };
            }
            "minrating" => {
                arguments.min_rating = value.parse().ok();
            }
            "maxrating" => {
                arguments.max_rating = value.parse().ok();
            }
            "limit" => {
                if let Ok(limit) = value.parse::<usize>()
                    && limit > 0
                {
                    arguments.limit = limit;
                }
            }
            "comments" if argument.key == "comments" => {
                arguments.comments = !value.is_empty();
            }
            _ => {}
        }
    }

    Some(arguments)
}

fn rated_pages_score_selectors(arguments: &RatedPagesArguments) -> Vec<ScoreSelector> {
    let mut selectors = Vec::with_capacity(2);
    if let Some(min_rating) = arguments.min_rating {
        selectors.push(ScoreSelector {
            score: ScoreValue::Integer(min_rating),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        });
    }
    if let Some(max_rating) = arguments.max_rating {
        selectors.push(ScoreSelector {
            score: ScoreValue::Integer(max_rating),
            comparison: ComparisonOperation::LessOrEqualThan,
        });
    }
    selectors
}

fn rated_pages_order(order: RatedPagesOrder) -> OrderBySelector {
    match order {
        RatedPagesOrder::RatingDesc => OrderBySelector {
            property: OrderProperty::Score,
            ascending: false,
        },
        RatedPagesOrder::RatingAsc => OrderBySelector {
            property: OrderProperty::Score,
            ascending: true,
        },
        RatedPagesOrder::DateCreatedDesc => OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: false,
        },
        RatedPagesOrder::DateCreatedAsc => OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: true,
        },
    }
}

fn format_rated_pages_score(score: f32) -> String {
    if score.fract() == 0.0 {
        (score as i64).to_string()
    } else {
        score.to_string()
    }
}

fn render_rated_pages_module(
    rows: &[crate::services::page_query::FoundPageRow],
    include_comments: bool,
    runtime_displays: &BTreeMap<i64, ListPagesRuntimeDisplay>,
) -> String {
    let mut output = String::from(
        "<div class=\"top-rated-pages-box\">\n\n\t<div class=\"top-rated-pages-list\">\n",
    );
    for row in rows {
        let slug = row.slug.as_deref().unwrap_or_default();
        let title = row.title.as_deref().unwrap_or(slug);
        let rating = format_rated_pages_score(row.score.unwrap_or(0.0));
        let comments = runtime_displays
            .get(&row.page_id)
            .map_or(0, |display| display.comments);
        output.push_str("\t\t\t\t\t<div class=\"list-item\">\n");
        output.push_str(&format!(
            "\t\t\t\t<a href=\"/{}\">{}</a>\n",
            escape_list_pages_html_attr(slug),
            escape_list_pages_html_text(title),
        ));
        let label = if include_comments {
            format!("Rating: {rating}, Comments: {comments}")
        } else {
            format!("Rating: {rating}")
        };
        output.push_str(&format!(
            "\t\t\t\t<span style=\"color: #777\">({})</span>\n",
            escape_list_pages_html_text(&label),
        ));
        output.push_str("\t\t\t</div>\n");
    }
    output.push_str("\t\t\t</div>\n\n</div>");
    output
}

fn current_page_calendar_category(page_info: &PageInfo<'_>) -> String {
    page_info
        .category
        .as_deref()
        .unwrap_or("_default")
        .to_owned()
}

fn current_page_calendar_target(page_info: &PageInfo<'_>) -> String {
    if page_info.page.contains(':') {
        page_info.page.to_string()
    } else if let Some(category) = page_info
        .category
        .as_deref()
        .filter(|category| *category != "_default")
    {
        format!("{category}:{}", page_info.page)
    } else {
        page_info.page.to_string()
    }
}

fn parse_page_calendar_arguments(
    head: &str,
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
) -> Option<PageCalendarArguments> {
    let parsed = wikidot_module_arguments(head)?;
    let url_attr_prefix = parsed
        .iter()
        .filter(|argument| argument.key.eq_ignore_ascii_case("urlattrprefix"))
        .map(|argument| argument.value.trim())
        .rfind(|prefix| !prefix.is_empty())
        .map(str::to_owned);
    let mut target_page = current_page_calendar_target(page_info);
    let mut category_value = None::<&str>;
    let mut tags_value = None::<&str>;

    for argument in &parsed {
        let value = argument.value.trim();
        match argument.key.to_ascii_lowercase().as_str() {
            "targetpage" | "startpage" if !value.is_empty() => {
                target_page = value.to_owned();
            }
            "category" => category_value = Some(value),
            "tags" => tags_value = Some(value),
            _ => {}
        }
    }

    let (category_value, category_from_url) = match category_value {
        Some(value) => match resolve_page_calendar_url_selector(
            value,
            url.value_for_list_pages_argument(url_attr_prefix.as_deref(), "category"),
        ) {
            PageCalendarUrlSelector::Value { value, from_url } => (value, from_url),
            PageCalendarUrlSelector::Dropped => {
                (current_page_calendar_category(page_info), false)
            }
        },
        None => (current_page_calendar_category(page_info), false),
    };
    let categories = parse_page_calendar_categories(&category_value);
    let category_url_value = category_from_url.then_some(category_value);

    let tags_url_value = tags_value.and_then(|value| {
        match resolve_page_calendar_url_selector(
            value,
            url.value_for_list_pages_argument(url_attr_prefix.as_deref(), "tags"),
        ) {
            PageCalendarUrlSelector::Value { value, .. } => {
                (!value.trim().is_empty()).then(|| page_calendar_link_tag_value(&value))
            }
            PageCalendarUrlSelector::Dropped => None,
        }
    });
    let selected_date = url
        .value_for_list_pages_argument(url_attr_prefix.as_deref(), "date")
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_owned());

    Some(PageCalendarArguments {
        categories,
        category_url_value,
        tags_url_value,
        selected_date,
        target_page,
        url_attr_prefix,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PageCalendarUrlSelector {
    Value { value: String, from_url: bool },
    Dropped,
}

fn resolve_page_calendar_url_selector(
    value: &str,
    url_value: Option<&str>,
) -> PageCalendarUrlSelector {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("@URL") {
        return url_value.map_or(PageCalendarUrlSelector::Dropped, |value| {
            PageCalendarUrlSelector::Value {
                value: value.to_owned(),
                from_url: true,
            }
        });
    }

    if trimmed
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("@URL|"))
    {
        return url_value.map_or_else(
            || PageCalendarUrlSelector::Value {
                value: trimmed[5..].to_owned(),
                from_url: false,
            },
            |value| PageCalendarUrlSelector::Value {
                value: value.to_owned(),
                from_url: true,
            },
        );
    }

    PageCalendarUrlSelector::Value {
        value: trimmed.to_owned(),
        from_url: false,
    }
}

fn parse_page_calendar_categories(value: &str) -> PageCalendarCategorySelector {
    let names = split_page_calendar_values(value)
        .into_iter()
        .filter(|category| !category.is_empty())
        .collect::<Vec<_>>();
    if names.iter().any(|category| category == "*") {
        PageCalendarCategorySelector::All
    } else {
        PageCalendarCategorySelector::Names(names)
    }
}

fn split_page_calendar_values(value: &str) -> Vec<String> {
    value
        .split(|character: char| character.is_whitespace() || character == ',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn page_calendar_link_tag_value(value: &str) -> String {
    value.trim().replace('+', " ")
}

fn page_calendar_argument_key(prefix: Option<&str>, name: &str) -> String {
    match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
        Some(prefix) => format!("{prefix}_{name}"),
        None => name.to_owned(),
    }
}

fn page_calendar_path(arguments: &PageCalendarArguments, date: &str) -> String {
    let mut path = String::from("/");
    path.push_str(&escape_list_pages_html_attr(&arguments.target_page));
    if let Some(tags) = &arguments.tags_url_value {
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
            arguments.url_attr_prefix.as_deref(),
            "tag",
        )));
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(tags));
    }
    if let Some(category) = &arguments.category_url_value {
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
            arguments.url_attr_prefix.as_deref(),
            "category",
        )));
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(category));
    }
    path.push('/');
    path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
        arguments.url_attr_prefix.as_deref(),
        "date",
    )));
    path.push('/');
    path.push_str(&escape_list_pages_html_attr(date));
    path
}

fn page_calendar_month_name(month: time::Month) -> &'static str {
    match month {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    }
}

fn render_page_calendar_error() -> String {
    format!(
        r#"<div class="error-block">{}</div>"#,
        escape_list_pages_html_text(PAGE_CALENDAR_CATEGORY_ERROR),
    )
}

fn render_page_calendar_module(
    arguments: &PageCalendarArguments,
    counts: &BTreeMap<i32, BTreeMap<u8, usize>>,
) -> String {
    let mut output = String::from("<div class=\"page-calendar-box\">\n\t\t\t<ul>\n");
    for (year, months) in counts.iter().rev() {
        let year_count = months.values().sum::<usize>();
        let year_date = year.to_string();
        let year_class = if arguments.selected_date.as_deref() == Some(year_date.as_str())
        {
            " class=\"selected\""
        } else {
            " "
        };
        output.push_str("\t\t\t\t\t<li");
        output.push_str(year_class);
        output.push_str(">\n\t\t\t\t<a href=\"");
        output.push_str(&page_calendar_path(arguments, &year_date));
        output.push_str("\">");
        output.push_str(&year.to_string());
        output.push_str(" (");
        output.push_str(&year_count.to_string());
        output.push_str(")</a>\n\t\t\t\t<ul>\n");

        for (month, count) in months.iter().rev() {
            let Some(month_name) = time::Month::try_from(*month)
                .ok()
                .map(page_calendar_month_name)
            else {
                continue;
            };
            let month_date = format!("{year}.{month}");
            let month_class =
                if arguments.selected_date.as_deref() == Some(month_date.as_str()) {
                    " class=\"selected\""
                } else {
                    " "
                };
            output.push_str("\t\t\t\t\t\t\t\t\t\t\t<li");
            output.push_str(month_class);
            output.push_str(">\n\t\t\t\t\t\t\t<a href=\"");
            output.push_str(&page_calendar_path(arguments, &month_date));
            output.push_str("\">");
            output.push_str(month_name);
            output.push_str(" (");
            output.push_str(&count.to_string());
            output.push_str(")</a>\n\t\t\t\t\t\t</li>\n");
        }

        output.push_str("\t\t\t\t\t\t\t\t\t</ul>\n\t\t\t</li>\n");
    }
    output.push_str("\t\t\t\t</ul>\n\t</div>");
    output
}

fn default_tag_cloud_arguments() -> TagCloudArguments {
    TagCloudArguments {
        mode_3d: false,
        min_font_size: TagCloudSize {
            value: 100.0,
            unit: "%",
        },
        max_font_size: TagCloudSize {
            value: 300.0,
            unit: "%",
        },
        // Live Wikidot's default color endpoints are the reverse of the
        // historical table labels: least-common tags are light, most-common
        // tags are dark.
        min_color: TagCloudColor {
            red: 128,
            green: 128,
            blue: 192,
        },
        max_color: TagCloudColor {
            red: 64,
            green: 64,
            blue: 128,
        },
        limit: TAG_CLOUD_DEFAULT_LIMIT,
        target: TAG_CLOUD_DEFAULT_TARGET.to_owned(),
        category: None,
        show_hidden: false,
        url_attr_prefix: None,
        skip_category_from_url: false,
        width: TAG_CLOUD_DEFAULT_WIDTH,
        height: TAG_CLOUD_DEFAULT_HEIGHT,
        error: None,
    }
}

fn parse_tag_cloud_arguments(head: &str) -> Option<TagCloudArguments> {
    let parsed = wikidot_module_arguments(head)?;
    let mut arguments = default_tag_cloud_arguments();
    let mut min_font_size = None;
    let mut max_font_size = None;
    let mut min_color = None;
    let mut max_color = None;

    for argument in parsed {
        let value = argument.value.trim();
        match argument.key.to_ascii_lowercase().as_str() {
            "mode" => arguments.mode_3d = value.eq_ignore_ascii_case("3d"),
            "minfontsize" => min_font_size = Some(value.to_owned()),
            "maxfontsize" => max_font_size = Some(value.to_owned()),
            "mincolor" => min_color = Some(value.to_owned()),
            "maxcolor" => max_color = Some(value.to_owned()),
            "limit" => {
                arguments.limit = value
                    .parse::<usize>()
                    .ok()
                    .filter(|limit| *limit > 0)
                    .unwrap_or(TAG_CLOUD_DEFAULT_LIMIT);
            }
            "target" if !value.is_empty() => arguments.target = value.to_owned(),
            "category" => {
                arguments.category = (!value.is_empty()).then(|| value.to_owned());
            }
            "showhidden" => {
                // Production Wikidot treats any non-empty value, including
                // "false" and "no", as enabling hidden tags.
                arguments.show_hidden = !value.is_empty();
            }
            "urlattrprefix" => {
                arguments.url_attr_prefix = (!value.is_empty()).then(|| value.to_owned());
            }
            "skipcategoryfromurl" => {
                arguments.skip_category_from_url = value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("yes");
            }
            "width" => {
                arguments.width = value
                    .parse::<u16>()
                    .ok()
                    .filter(|width| *width > 0)
                    .unwrap_or(TAG_CLOUD_DEFAULT_WIDTH);
            }
            "height" => {
                arguments.height = value
                    .parse::<u16>()
                    .ok()
                    .filter(|height| *height > 0)
                    .unwrap_or(TAG_CLOUD_DEFAULT_HEIGHT);
            }
            _ => {}
        }
    }

    if let (Some(min), Some(max)) = (min_font_size.as_deref(), max_font_size.as_deref()) {
        let Some(min) = parse_tag_cloud_size(min) else {
            arguments.error = Some(TAG_CLOUD_FONT_UNIT_ERROR);
            return Some(arguments);
        };
        let Some(max) = parse_tag_cloud_size(max) else {
            arguments.error = Some(TAG_CLOUD_FONT_UNIT_ERROR);
            return Some(arguments);
        };
        if min.unit != max.unit {
            arguments.error = Some(TAG_CLOUD_FONT_UNIT_ERROR);
            return Some(arguments);
        }
        arguments.min_font_size = min;
        arguments.max_font_size = max;
    }

    if let (Some(min), Some(max)) = (min_color.as_deref(), max_color.as_deref()) {
        let (Some(min), Some(max)) =
            (parse_tag_cloud_color(min), parse_tag_cloud_color(max))
        else {
            arguments.error = Some(TAG_CLOUD_COLOR_ERROR);
            return Some(arguments);
        };
        arguments.min_color = min;
        arguments.max_color = max;
    }

    Some(arguments)
}

fn parse_tag_cloud_size(value: &str) -> Option<TagCloudSize> {
    let trimmed = value.trim();
    let unit_start = trimmed
        .find(|character: char| !(character.is_ascii_digit() || character == '.'))
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(unit_start);
    if number.is_empty() {
        return None;
    }
    let value = number.parse::<f32>().ok().filter(|value| *value >= 0.0)?;
    let unit = match unit {
        "px" => "px",
        "pt" => "pt",
        "em" => "em",
        "%" => "%",
        _ => return None,
    };
    Some(TagCloudSize { value, unit })
}

fn parse_tag_cloud_color(value: &str) -> Option<TagCloudColor> {
    let parts = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<u8>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    let [red, green, blue]: [u8; 3] = parts.try_into().ok()?;
    Some(TagCloudColor { red, green, blue })
}

fn tag_cloud_path(arguments: &TagCloudArguments, tag: &str) -> String {
    let mut path = String::from("/");
    path.push_str(&escape_list_pages_html_attr(&arguments.target));
    path.push('/');
    if let Some(prefix) = &arguments.url_attr_prefix {
        path.push_str(&escape_list_pages_html_attr(prefix));
        path.push('_');
    }
    path.push_str("tag/");
    path.push_str(&percent_encode_path_segment(tag));
    if let Some(category) = &arguments.category
        && !arguments.skip_category_from_url
    {
        path.push('/');
        if let Some(prefix) = &arguments.url_attr_prefix {
            path.push_str(&escape_list_pages_html_attr(prefix));
            path.push('_');
        }
        path.push_str("category/");
        path.push_str(&percent_encode_path_segment(category));
    }
    path
}

fn tag_cloud_ratio(count: usize, min_count: usize, max_count: usize) -> f32 {
    if max_count <= min_count {
        0.0
    } else {
        (count.saturating_sub(min_count) as f32) / ((max_count - min_count) as f32)
    }
}

fn interpolate_tag_cloud_size(
    arguments: &TagCloudArguments,
    count: usize,
    min_count: usize,
    max_count: usize,
) -> String {
    let ratio = tag_cloud_ratio(count, min_count, max_count);
    let value = arguments.min_font_size.value
        + ((arguments.max_font_size.value - arguments.min_font_size.value) * ratio);
    format!(
        "{}{}",
        format_tag_cloud_number(value),
        arguments.min_font_size.unit
    )
}

fn interpolate_tag_cloud_color(
    arguments: &TagCloudArguments,
    count: usize,
    min_count: usize,
    max_count: usize,
) -> TagCloudColor {
    let ratio = tag_cloud_ratio(count, min_count, max_count);
    TagCloudColor {
        red: interpolate_tag_cloud_color_channel(
            arguments.min_color.red,
            arguments.max_color.red,
            ratio,
        ),
        green: interpolate_tag_cloud_color_channel(
            arguments.min_color.green,
            arguments.max_color.green,
            ratio,
        ),
        blue: interpolate_tag_cloud_color_channel(
            arguments.min_color.blue,
            arguments.max_color.blue,
            ratio,
        ),
    }
}

fn interpolate_tag_cloud_color_channel(min: u8, max: u8, ratio: f32) -> u8 {
    (min as f32 + ((max as f32 - min as f32) * ratio)).round() as u8
}

fn format_tag_cloud_number(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        let mut output = format!("{value:.2}");
        while output.contains('.') && output.ends_with('0') {
            output.pop();
        }
        if output.ends_with('.') {
            output.pop();
        }
        output
    }
}

fn render_tag_cloud_error(message: &str) -> String {
    format!(
        r#"<div class="error-block">{}</div>"#,
        escape_list_pages_html_text(message),
    )
}

fn displayed_tag_cloud_tags(
    tag_counts: &[(String, usize)],
    arguments: &TagCloudArguments,
) -> Vec<TagCloudTag> {
    let mut tags = tag_counts
        .iter()
        .filter(|(tag, _)| arguments.show_hidden || !tag.trim().starts_with('_'))
        .map(|(name, count)| TagCloudTag {
            name: name.clone(),
            count: *count,
        })
        .collect::<Vec<_>>();
    tags.sort_by(|left, right| {
        tag_cloud_sort_key(&left.name)
            .cmp(tag_cloud_sort_key(&right.name))
            .then_with(|| left.name.cmp(&right.name))
    });
    tags.truncate(arguments.limit);
    tags
}

fn tag_cloud_sort_key(tag: &str) -> &str {
    tag.trim().trim_start_matches('_')
}

fn tag_cloud_count_bounds(tags: &[TagCloudTag]) -> (usize, usize) {
    let min = tags.iter().map(|tag| tag.count).min().unwrap_or(0);
    let max = tags.iter().map(|tag| tag.count).max().unwrap_or(0);
    (min, max)
}

fn render_tag_cloud_2d(arguments: &TagCloudArguments, tags: &[TagCloudTag]) -> String {
    let (min_count, max_count) = tag_cloud_count_bounds(tags);
    let mut output = String::from("<div class=\"pages-tag-cloud-box\">\n");
    for tag in tags {
        let size = interpolate_tag_cloud_size(arguments, tag.count, min_count, max_count);
        let color =
            interpolate_tag_cloud_color(arguments, tag.count, min_count, max_count);
        output.push_str("\t<a class=\"tag\" href=\"");
        output.push_str(&escape_list_pages_html_attr(&tag_cloud_path(
            arguments, &tag.name,
        )));
        output.push_str("\" style=\"font-size: ");
        output.push_str(&size);
        output.push_str("; color: rgb(");
        output.push_str(&color.red.to_string());
        output.push_str(", ");
        output.push_str(&color.green.to_string());
        output.push_str(", ");
        output.push_str(&color.blue.to_string());
        output.push_str(");\">");
        output.push_str(&escape_list_pages_html_text(&tag.name));
        output.push_str("</a>\n");
    }
    output.push_str("</div>");
    output
}

fn tag_cloud_hex_color(color: TagCloudColor) -> String {
    format!("0x{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

fn escape_tag_cloud_javascript_single_quoted(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\'' => output.push_str("\\'"),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '<' => output.push_str("\\x3c"),
            '>' => output.push_str("\\x3e"),
            '&' => output.push_str("\\x26"),
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            _ => output.push(character),
        }
    }
    output
}

fn render_tag_cloud_3d(arguments: &TagCloudArguments, tags: &[TagCloudTag]) -> String {
    let (min_count, max_count) = tag_cloud_count_bounds(tags);
    let flash_id = format!(
        "flashcontent-{}",
        tags.iter().fold(273_000_u64, |hash, tag| {
            hash.wrapping_mul(33)
                .wrapping_add(tag.name.bytes().map(u64::from).sum::<u64>())
        }) % 1_000_000
    );
    let mut output = String::from("<div class=\"pages-tag-cloud-box\">\n");
    output.push_str("\t<script type=\"text/javascript\" src=\"http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--javascript/tagcloud/swfobject.js\"></script>\n");
    output.push_str("\t<div id=\"");
    output.push_str(&escape_list_pages_html_attr(&flash_id));
    output.push_str("\"></div>\n");
    output.push_str("\t<script type=\"text/javascript\">\n//<![CDATA[\n");
    output.push_str(&format!(
        "var so = new SWFObject(\"/common--javascript/tagcloud/tagcloud.swf\", \"tagcloud\", \"{}\", \"{}\", \"7\", \"#FFFFFF\");\n",
        arguments.width, arguments.height,
    ));
    output.push_str("so.addParam(\"wmode\", \"transparent\");\n");
    output.push_str("so. addVariable(\"mode\", \"tags\");\n");
    output.push_str("so. addVariable(\"distr\", \"true\");\n");
    output.push_str(&format!(
        "so.addVariable(\"tcolor\", \"{}\");\n",
        tag_cloud_hex_color(arguments.max_color),
    ));
    output.push_str(&format!(
        "so.addVariable(\"tcolor2\", \"{}\");\n",
        tag_cloud_hex_color(arguments.min_color),
    ));
    output.push_str(&format!(
        "so.addVariable(\"hicolor\", \"{}\");\n",
        tag_cloud_hex_color(arguments.max_color),
    ));
    output.push_str("so.addVariable(\"tagcloud\", \"<tags>\"+\n");
    for tag in tags {
        let ratio = tag_cloud_ratio(tag.count, min_count, max_count);
        let weight = (12.0 + (18.0 * ratio)).round() as i32;
        let href = tag_cloud_path(arguments, &tag.name);
        output.push_str("\t        encodeURIComponent('<a href=\"' + location.protocol + '//' + location.hostname + '");
        output.push_str(&escape_tag_cloud_javascript_single_quoted(&href));
        output.push_str("\" style=\"");
        output.push_str(&weight.to_string());
        output.push_str("\">");
        output.push_str(&escape_tag_cloud_javascript_single_quoted(&tag.name));
        output.push_str("</a>') +\n");
    }
    output.push_str("\t\"</tags>\");\n");
    output.push_str("so.write(\"");
    output.push_str(&escape_tag_cloud_javascript_single_quoted(&flash_id));
    output.push_str("\");\n//]]>\n</script>\n");
    output.push_str("</div>");
    output
}

fn render_tag_cloud_module(
    arguments: &TagCloudArguments,
    tag_counts: &[(String, usize)],
) -> String {
    if let Some(error) = arguments.error {
        return render_tag_cloud_error(error);
    }

    let tags = displayed_tag_cloud_tags(tag_counts, arguments);
    if arguments.mode_3d {
        render_tag_cloud_3d(arguments, &tags)
    } else {
        render_tag_cloud_2d(arguments, &tags)
    }
}

impl RenderService {
    pub(super) fn expand_rate_modules_with_registry(
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        rating_type: PageRatingType,
        compat_html: &mut CompatHtmlFragments,
        compat_text: &mut CompatTextFragments,
    ) -> String {
        if !settings.enable_page_syntax || !RATE_MODULE_REGEX.is_match(&wikitext) {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let footnote_ranges = collect_unproven_scope_ranges(&wikitext, &literal_regions)
            .into_iter()
            .filter(|range| wikidot_scope_head_is(&wikitext, range.start, "footnote"))
            .collect::<Vec<_>>();
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for matched in RATE_MODULE_REGEX.find_iter(&wikitext) {
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let line_start = wikitext[..matched.start()]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            if wikitext[line_start..matched.start()]
                .trim_start()
                .starts_with('>')
            {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            if footnote_ranges
                .iter()
                .any(|range| range.start < matched.start() && matched.end() <= range.end)
            {
                output.push_str(&compat_text.push_escaped_html_text(matched.as_str()));
                cursor = matched.end();
                continue;
            }
            output.push_str(&compat_html.push_block_html(render_read_only_rate_module(
                page_info.score,
                &page_info.language,
                rating_type,
            )));
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    pub(super) fn expand_registry_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        Self::expand_registry_modules_matching(wikitext, settings, compat_html, |_| true)
    }

    fn expand_registry_modules_matching(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        mut should_expand: impl FnMut(&str) -> bool,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        // Keep one index over the authored source for the complete pass. A replacement must not expose a later candidate that the original literal, comment, or tag boundaries protected, so malformed cross-boundary input remains fail closed.
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in REGISTRY_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a registry module capture always has a name")
                .as_str();
            if !should_expand(name) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let rendered = if name.eq_ignore_ascii_case("Members") {
                let group = wikidot_module_argument(head, "group")
                    .unwrap_or("members")
                    .trim();
                render_members_module_placeholder(group)
            } else if name.eq_ignore_ascii_case("NewPage") {
                render_new_page_module(head)
            } else if name.eq_ignore_ascii_case("Clone") {
                render_clone_module(head)
            } else {
                debug_assert!(name.eq_ignore_ascii_case("Join"));
                render_join_module(head)
            };
            let marker = if name.eq_ignore_ascii_case("Join") {
                compat_html.push_block_html(rendered)
            } else {
                compat_html.push_html(rendered)
            };
            output.push_str(&marker);
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    #[cfg(test)]
    pub(super) fn expand_members_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Members"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_new_page_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("NewPage"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_clone_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Clone"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_join_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Join"),
        );
        fragments.restore(&protected)
    }

    pub(super) async fn expand_secondary_runtime_modules(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: SecondaryRuntimeModuleExpansionOptions<'_>,
        compat_text: &mut CompatTextFragments,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::CountPages);
            Self::expand_count_pages(
                ctx,
                wikitext,
                page_info,
                settings,
                CountPagesExpansionOptions {
                    current_site_id: options.current_site_id,
                    current_page_id: options.current_page_id,
                    url: options.url,
                },
                compat_text,
            )
            .await
            .or_raise(make_error)?
        };
        if PAGECALENDAR_MODULE_REGEX.is_match(&wikitext) {
            wikitext = {
                let _stage =
                    StageGuard::new(options.trace, CorpusRenderStage::PageCalendar);
                Self::expand_page_calendar_modules(
                    ctx,
                    wikitext,
                    page_info,
                    settings,
                    PageCalendarExpansionOptions {
                        current_site_id: options.current_site_id,
                        current_page_id: options.current_page_id,
                        url: options.url,
                    },
                    compat_html,
                )
                .await
                .or_raise(make_error)?
            };
        }
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::RatedPages);
            Self::expand_rated_pages_modules(
                ctx,
                wikitext,
                settings,
                options.current_site_id,
                compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::TagCloud);
            Self::expand_tag_cloud_modules(
                ctx,
                wikitext,
                page_info,
                settings,
                options.current_site_id,
                options.current_page_id,
                compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        Ok(wikitext)
    }

    async fn expand_rated_pages_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !RATEDPAGES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let Some(current_site_id) = current_site_id else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in RATEDPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a RatedPages capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(arguments) = parse_rated_pages_arguments(head) else {
                continue;
            };
            expanded.push_str(&wikitext[cursor..matched.start()]);
            let rendered =
                Self::render_rated_pages_query(ctx, current_site_id, &arguments).await?;
            expanded.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn render_rated_pages_query(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        arguments: &RatedPagesArguments,
    ) -> Result<String> {
        let categories = arguments
            .category
            .as_deref()
            .map(|category| vec![Cow::Borrowed(category)]);
        let included_categories = categories
            .as_deref()
            .map_or(IncludedCategories::All, IncludedCategories::List);
        let score = rated_pages_score_selectors(arguments);
        let score_order = matches!(
            arguments.order,
            RatedPagesOrder::RatingAsc | RatedPagesOrder::RatingDesc
        );
        let query_limit = if score_order {
            u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
        } else {
            arguments.limit as u64
        };
        let query = PageQuery {
            current_page_id: 0,
            current_site_id,
            queried_site_id: None,
            page_type: PageTypeSelector::Normal,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &score,
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
            data_form_fields: &[],
            order: Some(rated_pages_order(arguments.order)),
            candidate_limit: Some(query_limit),
            pagination: PaginationSelector {
                limit: Some(query_limit),
                per_page: PaginationSelector::default().per_page,
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                title: true,
                slug: true,
                page_category_id: true,
                score: true,
                ..FoundPageFields::default()
            },
        };
        let mut permission_cache = BTreeMap::new();
        let rows = find_viewable_list_pages_rows(
            ctx,
            query,
            arguments.limit,
            &mut permission_cache,
            None,
        )
        .await?;
        let runtime_displays = if arguments.comments {
            Self::load_list_pages_runtime_displays(ctx, &rows.pages.pages).await?
        } else {
            BTreeMap::new()
        };
        Ok(render_rated_pages_module(
            &rows.pages.pages,
            arguments.comments,
            &runtime_displays,
        ))
    }

    pub(super) async fn expand_tag_cloud_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !TAGCLOUD_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let current_branch_tag = page_info
            .tags
            .iter()
            .find(|tag| tag.starts_with("branch-"))
            .map(Cow::as_ref);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in TAGCLOUD_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a TagCloud capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }

            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(arguments) = parse_tag_cloud_arguments(head) else {
                continue;
            };
            expanded.push_str(&wikitext[cursor..matched.start()]);
            let tags = Self::load_tag_cloud_counts(
                ctx,
                current_site_id,
                current_page_id,
                current_branch_tag,
                arguments.category.as_deref(),
            )
            .await?;
            expanded.push_str(
                &compat_html.push_block_html(render_tag_cloud_module(&arguments, &tags)),
            );
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    pub(super) async fn expand_page_calendar_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: PageCalendarExpansionOptions<'_>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !PAGECALENDAR_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (options.current_site_id, options.current_page_id)
        else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let current_branch_tag = page_info
            .tags
            .iter()
            .find(|tag| tag.starts_with("branch-"))
            .map(Cow::as_ref);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in PAGECALENDAR_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a PageCalendar capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }

            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(arguments) =
                parse_page_calendar_arguments(head, page_info, options.url)
            else {
                continue;
            };
            expanded.push_str(&wikitext[cursor..matched.start()]);
            match Self::load_page_calendar_counts(
                ctx,
                current_site_id,
                current_page_id,
                current_branch_tag,
                &arguments.categories,
            )
            .await?
            {
                Some(counts) => {
                    expanded.push_str(&compat_html.push_block_html(
                        render_page_calendar_module(&arguments, &counts),
                    ));
                }
                None => {
                    expanded.push_str(
                        &compat_html.push_block_html(render_page_calendar_error()),
                    );
                }
            }
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_page_calendar_counts(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        _current_page_id: i64,
        current_branch_tag: Option<&str>,
        categories: &PageCalendarCategorySelector,
    ) -> Result<Option<BTreeMap<i32, BTreeMap<u8, usize>>>> {
        let make_error =
            || Error::new("failed to render PageCalendar module", ErrorType::Render);
        let txn = ctx.transaction();
        let category_ids = match categories {
            PageCalendarCategorySelector::All => None,
            PageCalendarCategorySelector::Names(names) => {
                let requested = names
                    .iter()
                    .map(|name| name.trim())
                    .filter(|name| !name.is_empty())
                    .collect::<BTreeSet<_>>();
                if requested.is_empty() {
                    return Ok(None);
                }

                let mut values = Vec::<Value>::with_capacity(requested.len() + 1);
                values.push(current_site_id.into());
                values.extend(requested.iter().map(|name| (*name).into()));
                let placeholders = (2..(requested.len() + 2))
                    .map(|index| format!("${index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let categories = PageCalendarCategoryRow::find_by_statement(
                    Statement::from_sql_and_values(
                        txn.get_database_backend(),
                        format!(
                            "SELECT category_id, slug \
                             FROM page_category \
                             WHERE site_id = $1 \
                               AND slug IN ({placeholders})",
                        ),
                        values,
                    ),
                )
                .all(txn)
                .await
                .or_raise(make_error)?;
                let found = categories
                    .iter()
                    .map(|category| category.slug.as_str())
                    .collect::<BTreeSet<_>>();
                if found.len() != requested.len() {
                    return Ok(None);
                }
                Some(
                    categories
                        .into_iter()
                        .map(|category| category.category_id)
                        .collect::<Vec<_>>(),
                )
            }
        };

        let mut values = vec![current_site_id.into()];
        let category_filter = if let Some(category_ids) = &category_ids {
            values.extend(category_ids.iter().copied().map(Value::from));
            let placeholders = (2..(category_ids.len() + 2))
                .map(|index| format!("${index}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" AND p.page_category_id IN ({placeholders})")
        } else {
            String::new()
        };
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.created_at, pr.tags \
                 FROM page p \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE p.site_id = $1 \
                   AND p.deleted_at IS NULL \
                   {category_filter}",
            ),
            values,
        );
        let pages = PageCalendarPageRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut counts = BTreeMap::<i32, BTreeMap<u8, usize>>::new();
        let mut category_permissions = HashMap::new();
        for page in pages {
            let can_view = if let Some(can_view) =
                category_permissions.get(&page.page_category_id)
            {
                *can_view
            } else {
                let can_view = PermissionService::check_user_can(
                    ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id: current_site_id,
                        page_reference: Some(Reference::Id(page.page_id)),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(page.page_category_id)),
                        action: Action::View,
                    },
                )
                .await
                .or_raise(make_error)?;
                category_permissions.insert(page.page_category_id, can_view);
                can_view
            };
            if !can_view {
                continue;
            }

            if let Some(branch_tag) = current_branch_tag
                && !page.tags.iter().any(|tag| tag == branch_tag)
            {
                continue;
            }
            let month = u8::from(page.created_at.month());
            *counts
                .entry(page.created_at.year())
                .or_default()
                .entry(month)
                .or_default() += 1;
        }

        Ok(Some(counts))
    }

    async fn load_tag_cloud_counts(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        _current_page_id: i64,
        current_branch_tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(String, usize)>> {
        let make_error =
            || Error::new("failed to render TagCloud module", ErrorType::Render);
        let txn = ctx.transaction();
        let mut values = vec![current_site_id.into()];
        let category_filter = if let Some(category) = category {
            values.push(category.into());
            " AND pc.slug = $2"
        } else {
            ""
        };
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, pr.tags \
                 FROM page p \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 JOIN page_category pc ON pc.category_id = p.page_category_id \
                 WHERE p.site_id = $1 \
                   AND p.deleted_at IS NULL \
                   {category_filter}",
            ),
            values,
        );
        let pages = TagCloudPageTags::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut counts = BTreeMap::<String, usize>::new();
        let mut category_permissions = HashMap::new();
        for page in pages {
            let can_view = if let Some(can_view) =
                category_permissions.get(&page.page_category_id)
            {
                *can_view
            } else {
                let can_view = PermissionService::check_user_can(
                    ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id: current_site_id,
                        page_reference: Some(Reference::Id(page.page_id)),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(page.page_category_id)),
                        action: Action::View,
                    },
                )
                .await
                .or_raise(make_error)?;
                category_permissions.insert(page.page_category_id, can_view);
                can_view
            };
            if !can_view {
                continue;
            }

            if let Some(branch_tag) = current_branch_tag
                && !page.tags.iter().any(|tag| tag == branch_tag)
            {
                continue;
            }
            for tag in page.tags {
                if tag.trim().is_empty() {
                    continue;
                }
                *counts.entry(tag).or_default() += 1;
            }
        }

        Ok(counts.into_iter().collect())
    }
}

fn wikidot_scope_head_is(source: &str, start: usize, expected: &str) -> bool {
    let Some(tail) = source.get(start + 2..) else {
        return false;
    };
    let Some(end) = tail.find("]]") else {
        return false;
    };
    tail[..end].trim().eq_ignore_ascii_case(expected)
}
