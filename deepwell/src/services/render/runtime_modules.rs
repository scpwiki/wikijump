//! Runtime-backed Wikidot module expansion.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use sea_orm::{ConnectionTrait, FromQueryResult, Statement};

use super::compat::CompatHtmlFragments;
use super::compat::text_fragments::CompatTextFragments;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{wikidot_module_argument, wikidot_module_arguments};
use super::native_list_context::collect_unproven_scope_ranges;
use super::percent_encoding::percent_encode_path_segment;
use super::service::{
    RATE_MODULE_REGEX, REGISTRY_MODULE_REGEX, RenderService, TAGCLOUD_MODULE_REGEX,
    escape_list_pages_html_attr, escape_list_pages_html_text, render_clone_module,
    render_members_module_placeholder, render_new_page_module,
    render_read_only_rate_module,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
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
