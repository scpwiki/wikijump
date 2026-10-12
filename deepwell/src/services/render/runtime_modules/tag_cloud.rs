//! Wikidot TagCloud runtime expansion.

use super::{
    BTreeMap, CompatHtmlFragments, CompatTextFragments, ConnectionTrait, Cow, Error,
    ErrorType, FromQueryResult, LiteralRegionIndex, PageInfo, PermissionService,
    RenderService, Result, ResultExt, ServiceContext, Statement, TAGCLOUD_MODULE_REGEX,
    Value, WikitextSettings, escape_list_pages_html_attr, escape_list_pages_html_text,
    percent_encode_path_segment, wikidot_module_arguments,
};

const TAG_CLOUD_DEFAULT_LIMIT: usize = 50;
/// PostgreSQL accepts at most 65,535 bind parameters per statement. Chunk the
/// revision-tag lookup well below that so a large site cannot fail the whole
/// render with a protocol-limit error.
const REVISION_ID_QUERY_CHUNK: usize = 10_000;
const TAG_CLOUD_DEFAULT_TARGET: &str = "system:page-tags";
const TAG_CLOUD_DEFAULT_WIDTH: u16 = 300;
const TAG_CLOUD_DEFAULT_HEIGHT: u16 = 300;
const TAG_CLOUD_FONT_UNIT_ERROR: &str =
    "Format for minFontSize and maxFontSize must be the same (px, em, pt or %).";
const TAG_CLOUD_COLOR_ERROR: &str = "Unsupported color format. Use \"RRR,GGG,BBB\" for Red,Green,Blue each within 0-255 range.";

#[derive(Clone, Copy, Debug)]
pub(super) struct TagCloudExpansionOptions {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
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
struct TagCloudPage {
    page_category_id: i64,
    latest_revision_id: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct TagCloudRevisionTags {
    tags: Vec<String>,
}

#[derive(Clone, Debug)]
struct TagCloudTag {
    name: String,
    count: usize,
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

pub(super) fn wikitext_has_executable_tag_cloud_module(wikitext: &str) -> bool {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(wikitext);
    TAGCLOUD_MODULE_REGEX
        .captures_iter(wikitext)
        .any(|captures| {
            let module = captures
                .get(0)
                .expect("a TagCloud capture always has a complete match");
            !literal_regions.contains(module.start())
                && parse_tag_cloud_arguments(
                    captures.name("head").map_or("", |head| head.as_str()),
                )
                .is_some_and(|arguments| !arguments.mode_3d)
        })
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

fn render_tag_cloud_module(
    arguments: &TagCloudArguments,
    tag_counts: &[(String, usize)],
) -> String {
    if let Some(error) = arguments.error {
        return render_tag_cloud_error(error);
    }

    let tags = displayed_tag_cloud_tags(tag_counts, arguments);
    render_tag_cloud_2d(arguments, &tags)
}

impl RenderService {
    pub(super) async fn expand_tag_cloud_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: TagCloudExpansionOptions,
        compat_text: &mut CompatTextFragments,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !TAGCLOUD_MODULE_REGEX.is_match(&wikitext) {
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
            if arguments.mode_3d {
                expanded.push_str(&compat_text.push_escaped_html_text(matched.as_str()));
                cursor = matched.end();
                continue;
            }
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
                "SELECT p.page_category_id, p.latest_revision_id \
                 FROM page p \
                 JOIN page_category pc ON pc.category_id = p.page_category_id \
                 WHERE p.site_id = $1 \
                   AND p.deleted_at IS NULL \
                   {category_filter}",
            ),
            values,
        );
        let pages = TagCloudPage::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;
        // The TagCloud viewer is anonymous, so a page reference cannot grant a
        // page-scoped virtual role such as PageAuthor. Every scanned page can
        // therefore share one permission derivation plus one category probe per
        // distinct category, instead of a full page-scoped check per page.
        let category_permissions = &mut BTreeMap::new();
        let category_ids = pages
            .iter()
            .map(|page| Some(page.page_category_id))
            .collect::<Vec<_>>();
        let viewable = PermissionService::batch_check_page_view_without_page_roles(
            ctx,
            None,
            current_site_id,
            &category_ids,
            category_permissions,
        )
        .await
        .or_raise(make_error)?;

        let visible_revision_ids = pages
            .iter()
            .zip(viewable)
            .filter(|(_, can_view)| *can_view)
            .filter_map(|(page, _)| page.latest_revision_id)
            .collect::<Vec<_>>();

        if visible_revision_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch revision tags in bounded chunks so a site with more pages than
        // PostgreSQL's bind-parameter ceiling still renders instead of failing.
        let mut counts = BTreeMap::<String, usize>::new();
        for chunk in visible_revision_ids.chunks(REVISION_ID_QUERY_CHUNK) {
            let revision_values =
                chunk.iter().copied().map(Value::from).collect::<Vec<_>>();
            let revision_placeholders = (1..=revision_values.len())
                .map(|index| format!("${index}"))
                .collect::<Vec<_>>()
                .join(", ");
            let revision_statement = Statement::from_sql_and_values(
                txn.get_database_backend(),
                format!(
                    "SELECT pr.tags \
                     FROM page_revision pr \
                     WHERE pr.revision_id IN ({revision_placeholders})",
                ),
                revision_values,
            );
            let revisions = TagCloudRevisionTags::find_by_statement(revision_statement)
                .all(txn)
                .await
                .or_raise(make_error)?;
            for revision in revisions {
                if let Some(branch_tag) = current_branch_tag
                    && !revision.tags.iter().any(|tag| tag == branch_tag)
                {
                    continue;
                }
                for tag in revision.tags {
                    if tag.trim().is_empty() {
                        continue;
                    }
                    *counts.entry(tag).or_default() += 1;
                }
            }
        }

        Ok(counts.into_iter().collect())
    }
}
