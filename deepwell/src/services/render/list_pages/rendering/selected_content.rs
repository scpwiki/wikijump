/*
 * services/render/list_pages/rendering/selected_content.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

#![allow(clippy::wildcard_imports)]

use super::super::super::module_arguments::wikidot_module_argument;
use super::*;
use crate::services::render::service::wikidot_inline_span_marker_open;
use ftml::tree::{SocialButtons, SocialSelection, SocialService};

pub(super) fn select_list_pages_rows(
    pages: Vec<FoundPageRow>,
    exact_visible_tags: bool,
    current_visible_tags: &BTreeSet<String>,
    relative_range: Option<RangeSelector>,
    current_page_id: i64,
    exclude_current_page: bool,
    offset: u32,
) -> Vec<FoundPageRow> {
    let pages = pages
        .into_iter()
        .filter(|page| {
            !exact_visible_tags
                || page
                    .tags
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .filter(|tag| is_list_pages_visible_tag(tag))
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
                    == current_visible_tags
                        .iter()
                        .map(String::as_str)
                        .collect::<BTreeSet<_>>()
        })
        .collect::<Vec<_>>();
    let pages = if let Some(relative_range) = relative_range {
        let current_index = pages
            .iter()
            .position(|page| page.page_id == current_page_id);
        match (relative_range, current_index) {
            (RangeSelector::Before, Some(index)) => {
                pages.into_iter().take(index).collect()
            }
            (RangeSelector::After, Some(index)) => {
                pages.into_iter().skip(index + 1).collect()
            }
            _ => Vec::new(),
        }
    } else {
        pages
    };
    pages
        .into_iter()
        .filter(|page| !exclude_current_page || page.page_id != current_page_id)
        .skip(offset as usize)
        .collect()
}

static LISTPAGES_CONTENT_CONTEXT_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
            r#"(?is)\[\[module\s+(?P<name>Clone|Backlinks|PreviousPage|NextPage|PetitionAdmin|SiteGrid|ThemePreviewer)\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]|\[\[social(?P<social_head>\s+[^\]]*)?\]\]"#,
        )
        .expect("ListPages selected-content module expression is valid")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectedContentIncludeMode {
    Execute,
    Preserve,
}

#[derive(Debug)]
pub(super) struct RenderedListPagesSource {
    pub(super) body: String,
    pub(super) styles: Vec<String>,
    pub(super) included_pages: Vec<PageRef>,
    pub(super) expanded_include_count: usize,
}

#[derive(Clone, Copy)]
struct WikidotSocialProvider {
    service: SocialService,
    title: &'static str,
    image: &'static str,
    href: &'static str,
    onclick: Option<&'static str>,
}

const WIKIDOT_SOCIAL_PROVIDERS: &[WikidotSocialProvider] = &[
    WikidotSocialProvider {
        service: SocialService::BlinkList,
        title: "BlinkList",
        image: "blinklist.png",
        href: "http://www.blinklist.com/index.php?Action=Blink/addblink.php&Description=&Url={url}&Title=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Blogmarks,
        title: "blogmarks",
        image: "blogmarks.png",
        href: "http://blogmarks.net/my/new.php?mini=1&simple=1&url={url}&title=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Delicious,
        title: "del.icio.us",
        image: "delicious.png",
        href: "http://del.icio.us/post?url={url}&title=TITLE",
        onclick: Some(
            "window.open('http://del.icio.us/post?v=4&noui&jump=close&url='+encodeURIComponent(location.href)+'&title='+encodeURIComponent(document.title), 'delicious','toolbar=no,width=700,height=400'); return false;",
        ),
    },
    WikidotSocialProvider {
        service: SocialService::Digg,
        title: "digg",
        image: "digg.png",
        href: "http://digg.com/submit?phase=2&url={url}&title=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Fark,
        title: "Fark",
        image: "fark.png",
        href: "http://cgi.fark.com/cgi/fark/edit.pl?new_url={url}&new_comment=TITLE&new_comment={site}&linktype=Misc",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::FeedMeLinks,
        title: "feedmelinks",
        image: "feedmelinks.png",
        href: "http://feedmelinks.com/categorize?from=toolbar&op=submit&url={url}&name=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Furl,
        title: "Furl",
        image: "furl.png",
        href: "http://www.furl.net/storeIt.jsp?u={url}&t=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::LinkaGoGo,
        title: "LinkaGoGo",
        image: "linkagogo.png",
        href: "http://www.linkagogo.com/go/AddNoPopup?url={url}&title=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::NewsVine,
        title: "NewsVine",
        image: "newsvine.png",
        href: "http://www.newsvine.com/_tools/seed&save?u={url}&h=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Netvouz,
        title: "Netvouz",
        image: "netvouz.png",
        href: "http://www.netvouz.com/action/submitBookmark?url={url}&title=TITLE&description=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Reddit,
        title: "Reddit",
        image: "reddit.png",
        href: "http://reddit.com/submit?url={url}&title=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::YahooMyWeb,
        title: "YahooMyWeb",
        image: "yahoomyweb.png",
        href: "http://myweb2.search.yahoo.com/myresults/bookmarklet?u={url}&=TITLE",
        onclick: None,
    },
    WikidotSocialProvider {
        service: SocialService::Facebook,
        title: "Facebook",
        image: "facebook.gif",
        href: "http://www.facebook.com/share.php?u={url}",
        onclick: Some(
            "window.open('http://www.facebook.com/sharer.php?u='+encodeURIComponent(location.href)+'&t='+encodeURIComponent(document.title),'sharer','toolbar=0,status=0,width=626,height=436');return false;",
        ),
    },
];

fn wikidot_social_percent_encode(value: &str, spaces_as_plus: bool) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(char::from(byte));
        } else if spaces_as_plus && byte == b' ' {
            output.push('+');
        } else {
            use std::fmt::Write as _;
            write!(&mut output, "%{byte:02X}")
                .expect("writing percent encoding to String cannot fail");
        }
    }
    output
}

fn wikidot_social_nonce(source: &str, offset: usize) -> u32 {
    let hash = source
        .bytes()
        .chain(offset.to_le_bytes())
        .fold(5381_u32, |hash, byte| {
            hash.wrapping_mul(33).wrapping_add(u32::from(byte))
        });
    10_000 + hash % 90_000
}

pub(in crate::services::render) fn render_wikidot_social_module(
    social: &SocialButtons,
    endpoint: &str,
    site_name: &str,
    social_id: &str,
) -> String {
    let selected: Vec<Option<&WikidotSocialProvider>> = match social.selection() {
        None => WIKIDOT_SOCIAL_PROVIDERS.iter().map(Some).collect(),
        Some(selection) => selection
            .iter()
            .map(|selection| match selection {
                SocialSelection::Service(service) => WIKIDOT_SOCIAL_PROVIDERS
                    .iter()
                    .find(|provider| provider.service == *service),
                SocialSelection::Empty => None,
            })
            .collect(),
    };
    let endpoint = wikidot_social_percent_encode(endpoint, false);
    let site_name = wikidot_social_percent_encode(site_name, true);
    let mut output = format!("\n\n<span id=\"{social_id}\">");
    for provider in selected {
        let Some(provider) = provider else {
            output.push_str(
                "<a href=\"\" style=\"margin: 0 2px\" title=\"\"><img src=\"http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--images/social/\" alt=\"\" /></a>",
            );
            continue;
        };
        let href = provider
            .href
            .replace("{url}", &endpoint)
            .replace("{site}", &site_name);
        output.push_str("<a href=\"");
        output.push_str(&escape_list_pages_html_attr(&href));
        output.push_str("\" style=\"margin: 0 2px\" title=\"");
        output.push_str(provider.title);
        output.push('"');
        if let Some(onclick) = provider.onclick {
            output.push_str(" onclick=\"");
            output.push_str(&escape_list_pages_html_attr(onclick));
            output.push('"');
        }
        output.push_str("><img src=\"http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--images/social/");
        output.push_str(provider.image);
        output.push_str("\" alt=\"");
        output.push_str(provider.title);
        output.push_str("\" /></a>");
    }
    output.push_str("</span>\n<script type=\"text/javascript\">\n//<![CDATA[\n\n");
    output.push_str("            var socialspan = $j(\"#");
    output.push_str(social_id);
    output.push_str("\")[0];\n");
    output.push_str(
        concat!(
            "            var els = socialspan.getElementsByTagName(\"a\");\n",
            "            for (var i=0;i<els.length;i++) {\n",
            "                els[i].href = els[i].href.replace(\"TITLE\", encodeURIComponent(document.title));\n",
            "            }\n",
            "//]]>\n",
            "</script>",
        ),
    );
    output
}

fn prepare_list_pages_selected_content_runtime(
    wikitext: String,
    fragments: &mut CompatHtmlFragments,
    site_slug: &str,
    site_name: &str,
) -> String {
    if !LISTPAGES_CONTENT_CONTEXT_MODULE_REGEX.is_match(&wikitext) {
        return wikitext;
    }

    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    for captures in LISTPAGES_CONTENT_CONTEXT_MODULE_REGEX.captures_iter(&wikitext) {
        let matched = captures
            .get(0)
            .expect("a selected-content module capture always has a match");
        if literal_regions.contains(matched.start()) {
            continue;
        }
        output.push_str(&wikitext[cursor..matched.start()]);
        let replacement = match captures.name("name").map(|name| name.as_str()) {
            None => {
                let social = SocialButtons::parse(
                    captures
                        .name("social_head")
                        .map_or("", |head| head.as_str()),
                );
                let social_id = format!("social{}", wikidot_social_nonce(&wikitext, matched.start()));
                let endpoint =
                    format!("http://{site_slug}.wikidot.com/ajax-module-connector.php");
                fragments.push_html(render_wikidot_social_module(
                    &social,
                    &endpoint,
                    site_name,
                    &social_id,
                ))
            }
            Some(name) if name.eq_ignore_ascii_case("Clone") => {
                fragments.push_block_html(
                    r#"<div class="error-block">You should be logged in to clone a site.</div>"#
                        .to_owned(),
                )
            }
            Some(name) if name.eq_ignore_ascii_case("Backlinks") => {
                fragments.push_block_html(
                    r#"<div class="backlinks-module-box"></div>"#.to_owned(),
                )
            }
            Some(name)
                if name.eq_ignore_ascii_case("PreviousPage")
                    || name.eq_ignore_ascii_case("NextPage") =>
            {
                fragments.push_block_html(
                    r#"<div class="error-block">The ListPages module does not work recursively.</div>"#
                        .to_owned(),
                )
            }
            Some(name) if name.eq_ignore_ascii_case("PetitionAdmin") => {
                fragments.push_block_html(
                    r#"<div class="error-block"><div class="title">Permission error</div>This tool is for use by the administrators of this site</div>"#
                        .to_owned(),
                )
            }
            Some(name) if name.eq_ignore_ascii_case("SiteGrid") => {
                fragments.push_block_html(
                    r#"<div class="error-block">No sites provided.</div>"#
                        .to_owned(),
                )
            }
            Some(name) if name.eq_ignore_ascii_case("ThemePreviewer") => {
                let head = captures.name("head").map_or("", |head| head.as_str());
                if wikidot_module_argument(head, "noUi")
                    .is_some_and(|value| value.eq_ignore_ascii_case("true"))
                {
                    String::new()
                } else {
                    matched.as_str().to_owned()
                }
            }
            Some(_) => unreachable!("the selected-content regex names every module"),
        };
        output.push_str(&replacement);
        cursor = matched.end();
    }
    if cursor == 0 {
        return wikitext;
    }
    output.push_str(&wikitext[cursor..]);
    output
}

fn protect_selected_content_spans(
    source: &str,
    fragments: &mut CompatHtmlFragments,
) -> String {
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut open_spans = 0usize;
    while cursor < source.len() {
        let open = source[cursor..].find("[[span");
        let close = source[cursor..].find("[[/span]]");
        let Some((relative, closing)) = (match (open, close) {
            (Some(open), Some(close)) if close < open => Some((close, true)),
            (Some(open), _) => Some((open, false)),
            (None, Some(close)) => Some((close, true)),
            (None, None) => None,
        }) else {
            break;
        };
        let start = cursor + relative;
        if literal_regions.contains(start) {
            let end = start
                + if closing {
                    "[[/span]]".len()
                } else {
                    "[[span".len()
                };
            output.push_str(&source[cursor..end]);
            cursor = end;
            continue;
        }
        if closing {
            let end = start + "[[/span]]".len();
            if open_spans > 0 {
                output.push_str(&source[cursor..start]);
                output.push_str(&fragments.push_html("</span>".to_owned()));
                open_spans -= 1;
                cursor = end;
            } else {
                output.push_str(&source[cursor..end]);
                cursor = end;
            }
            continue;
        }
        let marker_start = &source[start..];
        let Some(relative_end) = marker_start.find("]]") else {
            output.push_str(&source[cursor..]);
            return output;
        };
        let end = start + relative_end + 2;
        let marker = &source[start..end];
        let Some(open_tag) = wikidot_inline_span_marker_open(marker) else {
            output.push_str(&source[cursor..end]);
            cursor = end;
            continue;
        };
        output.push_str(&source[cursor..start]);
        output.push_str(&fragments.push_html(open_tag));
        open_spans += 1;
        cursor = end;
    }
    output.push_str(&source[cursor..]);
    output
}

/// Selected `%%content%%` is rendered in a nested pass so ordinary Wikidot
/// syntax in the row still receives the page renderer. Includes are a
/// deliberate exception: executing them here would make selected page rows
/// consume the caller's include budget. Hide only include and image directives
/// from that nested pass and restore them as authored text after rendering.
/// Literal regions are excluded so comments, raw text, and code do not acquire
/// a new executable boundary.
fn protect_selected_content_includes(
    source: &str,
    compat_text: &mut CompatTextFragments,
) -> String {
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let bytes = source.as_bytes();
    let mut protected = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while cursor + 1 < bytes.len() {
        let Some(relative_start) =
            bytes[cursor..].windows(2).position(|pair| pair == b"[[")
        else {
            break;
        };
        let start = cursor + relative_start;
        if literal_regions.contains(start) {
            protected.push_str(&source[cursor..start + 2]);
            cursor = start + 2;
            continue;
        }
        let mut head = start + 2;
        while bytes.get(head).is_some_and(u8::is_ascii_whitespace) {
            head += 1;
        }
        let is_include = bytes
            .get(head..head.saturating_add(7))
            .is_some_and(|keyword| keyword.eq_ignore_ascii_case(b"include"))
            && !bytes
                .get(head + 7)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
        let is_image = bytes
            .get(head..head.saturating_add(5))
            .is_some_and(|keyword| keyword.eq_ignore_ascii_case(b"image"))
            && !bytes
                .get(head + 5)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
        if !is_include && !is_image {
            protected.push_str(&source[cursor..start + 2]);
            cursor = start + 2;
            continue;
        }
        let end = source[start + 2..]
            .find("]]")
            .map_or(source.len(), |relative_end| start + 2 + relative_end + 2);
        protected.push_str(&source[cursor..start]);
        protected.push_str(&compat_text.push_escaped_html_text(&source[start..end]));
        cursor = end;
    }
    protected.push_str(&source[cursor..]);
    protected
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn render_list_pages_default_summary_source(
    ctx: &ServiceContext<'_>,
    wikitext: &str,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    current_site_id: i64,
    current_category_id: i64,
    current_page_id: i64,
    page_preview: bool,
    viewer_user_id: Option<i64>,
    max_include_expansions: usize,
    render_cost_budget: SharedRenderCostBudget,
    url: UrlArguments<'_>,
) -> Result<RenderedListPagesSource> {
    render_cost_budget
        .charge(1, "default-summary render")
        .map_err(|error| Error::new(error.to_string(), ErrorType::Render))?;
    let _nested_render_guard = render_cost_budget
        .enter_nested_render(MAX_SELECTED_CONTENT_RENDER_DEPTH)
        .map_err(|error| Error::new(error.to_string(), ErrorType::Render))?;
    let mut summary_settings = settings.clone();
    summary_settings.enable_html_blocks = true;
    let render_context = RenderContext::list_pages_default_summary(
        current_site_id,
        current_category_id,
        current_page_id,
        page_preview,
    );
    let rendered = Box::pin(RenderService::render_inner(
        ctx,
        wikitext.to_owned(),
        page_info,
        &summary_settings,
        RenderInnerOptions {
            observed_user_info: None,
            render_context,
            viewer_user_id,
            current_page_data_form_values: None,
            max_include_expansions,
            render_cost_budget,
            trace: None,
            persist_compiled_text: false,
            url,
        },
    ))
    .await?;
    let mut html_output = rendered.html_output;
    Ok(RenderedListPagesSource {
        body: html_output.body,
        styles: std::mem::take(&mut html_output.styles),
        included_pages: std::mem::take(&mut html_output.backlinks.included_pages),
        expanded_include_count: rendered.expanded_include_count,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn render_list_pages_selected_content_source(
    ctx: &ServiceContext<'_>,
    wikitext: &str,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    current_site_id: i64,
    current_category_id: i64,
    current_page_id: i64,
    viewer_user_id: Option<i64>,
    include_mode: SelectedContentIncludeMode,
    max_include_expansions: usize,
    render_cost_budget: SharedRenderCostBudget,
    url: UrlArguments<'_>,
) -> Result<RenderedListPagesSource> {
    render_cost_budget
        .charge(1, "selected-content render")
        .map_err(|error| Error::new(error.to_string(), ErrorType::Render))?;
    let _nested_render_guard = render_cost_budget
        .enter_nested_render(MAX_SELECTED_CONTENT_RENDER_DEPTH)
        .map_err(|error| Error::new(error.to_string(), ErrorType::Render))?;
    let mut selected_content_fragments = CompatHtmlFragments::new(wikitext);
    let selected_content_site =
        if LISTPAGES_CONTENT_CONTEXT_MODULE_REGEX.is_match(wikitext) {
            Some(SiteService::get(ctx, Reference::Id(current_site_id)).await?)
        } else {
            None
        };
    let wikitext = prepare_list_pages_selected_content_runtime(
        wikitext.to_owned(),
        &mut selected_content_fragments,
        selected_content_site
            .as_ref()
            .map_or(page_info.site.as_ref(), |site| site.slug.as_str()),
        selected_content_site
            .as_ref()
            .map_or("", |site| site.name.as_str()),
    );
    let wikitext =
        protect_selected_content_spans(&wikitext, &mut selected_content_fragments);
    let mut selected_content_text = CompatTextFragments::new(&wikitext);
    let wikitext = match include_mode {
        SelectedContentIncludeMode::Execute => wikitext,
        SelectedContentIncludeMode::Preserve => {
            protect_selected_content_includes(&wikitext, &mut selected_content_text)
        }
    };
    let mut selected_content_settings = settings.clone();
    selected_content_settings.enable_page_syntax = true;
    let wikitext = RenderService::suppress_rate_modules_in_list_pages_content(
        wikitext,
        &selected_content_settings,
    );
    selected_content_settings.enable_html_blocks = true;
    let rendered = Box::pin(RenderService::render_inner(
        ctx,
        wikitext,
        page_info,
        &selected_content_settings,
        RenderInnerOptions {
            observed_user_info: None,
            render_context: RenderContext::list_pages_default_summary(
                current_site_id,
                current_category_id,
                current_page_id,
                false,
            ),
            viewer_user_id,
            current_page_data_form_values: None,
            max_include_expansions,
            render_cost_budget,
            trace: None,
            persist_compiled_text: false,
            url,
        },
    ))
    .await?;
    let mut html_output = rendered.html_output;
    let rendered_body = selected_content_text.restore(&html_output.body);
    Ok(RenderedListPagesSource {
        body: selected_content_fragments.restore(&rendered_body),
        styles: std::mem::take(&mut html_output.styles),
        included_pages: std::mem::take(&mut html_output.backlinks.included_pages),
        expanded_include_count: rendered.expanded_include_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_authored_spans_without_touching_unmatched_closers() {
        let source = r#"before [[span class="bi-y"]]body[[/span]] after [[/span]]"#;
        let mut fragments = CompatHtmlFragments::new(source);
        let protected = protect_selected_content_spans(source, &mut fragments);

        assert_eq!(
            fragments.restore(&format!("<p>{protected}</p>")),
            r#"<p>before <span class="bi-y">body</span> after [[/span]]</p>"#,
        );
    }

    fn row(page_id: i64, tags: Option<&[&str]>) -> FoundPageRow {
        FoundPageRow {
            page_id,
            site_id: 1,
            title: None,
            alt_title: None,
            slug: None,
            page_category_id: None,
            page_revision_id: None,
            tags: tags.map(|tags| tags.iter().map(|tag| (*tag).to_owned()).collect()),
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
            revision_count: None,
        }
    }

    fn ids(rows: &[FoundPageRow]) -> Vec<i64> {
        rows.iter().map(|row| row.page_id).collect()
    }

    #[test]
    fn row_selection_applies_offset_and_current_page_exclusion() {
        let pages = vec![row(1, None), row(2, None), row(3, None)];
        let tags = BTreeSet::new();

        let all = select_list_pages_rows(pages.clone(), false, &tags, None, 2, false, 0);
        assert_eq!(ids(&all), vec![1, 2, 3]);

        let offset =
            select_list_pages_rows(pages.clone(), false, &tags, None, 2, false, 1);
        assert_eq!(ids(&offset), vec![2, 3]);

        let excluded =
            select_list_pages_rows(pages.clone(), false, &tags, None, 2, true, 0);
        assert_eq!(ids(&excluded), vec![1, 3]);
    }

    #[test]
    fn row_selection_filters_exact_visible_tags() {
        let pages = vec![
            row(1, Some(&["alpha", "_hidden"])),
            row(2, Some(&["alpha"])),
            row(3, Some(&["beta"])),
        ];
        let mut current = BTreeSet::new();
        current.insert("alpha".to_owned());

        let selected = select_list_pages_rows(pages, true, &current, None, 1, false, 0);
        assert_eq!(
            ids(&selected),
            vec![1, 2],
            "only rows whose visible tags match are kept",
        );
    }

    #[test]
    fn relative_range_selects_before_and_after_the_current_page() {
        let pages = vec![row(1, None), row(2, None), row(3, None)];
        let tags = BTreeSet::new();

        let before = select_list_pages_rows(
            pages.clone(),
            false,
            &tags,
            Some(RangeSelector::Before),
            2,
            false,
            0,
        );
        assert_eq!(ids(&before), vec![1]);

        let after = select_list_pages_rows(
            pages.clone(),
            false,
            &tags,
            Some(RangeSelector::After),
            2,
            false,
            0,
        );
        assert_eq!(ids(&after), vec![3]);

        let missing = select_list_pages_rows(
            pages,
            false,
            &tags,
            Some(RangeSelector::Before),
            99,
            false,
            0,
        );
        assert!(missing.is_empty(), "a missing current page selects nothing");
    }

    #[test]
    fn social_percent_encoding_matches_the_form_contract() {
        assert_eq!(wikidot_social_percent_encode("a b", false), "a%20b");
        assert_eq!(wikidot_social_percent_encode("a b", true), "a+b");
        assert_eq!(
            wikidot_social_percent_encode("A/B?c=d", false),
            "A%2FB%3Fc%3Dd"
        );
        assert_eq!(wikidot_social_percent_encode("~._-", false), "~._-");
        assert_eq!(wikidot_social_percent_encode("é", false), "%C3%A9");
    }

    #[test]
    fn social_nonce_is_bounded_and_offset_sensitive() {
        let base = wikidot_social_nonce("source", 0);
        assert!((10_000..100_000).contains(&base));
        assert_eq!(base, wikidot_social_nonce("source", 0));
        assert_ne!(base, wikidot_social_nonce("source", 1));
        assert_ne!(base, wikidot_social_nonce("other", 0));
    }

    #[test]
    fn selected_content_protects_only_include_and_image_openings() {
        let source = concat!(
            "a [[include component:x]] b [[image foo.png]] ",
            "c [[span]]d[[/span]] e [[includex]] f [[imagefoo]]",
        );
        let mut fragments = CompatTextFragments::new(source);
        let protected = protect_selected_content_includes(source, &mut fragments);

        assert!(
            !protected.contains("[[include component:x]]"),
            "{protected}"
        );
        assert!(!protected.contains("[[image foo.png]]"), "{protected}");
        assert!(protected.contains("[[span]]"), "{protected}");
        assert!(protected.contains("[[includex]]"), "{protected}");
        assert!(protected.contains("[[imagefoo]]"), "{protected}");
        assert_eq!(fragments.restore(&protected), source);
    }

    #[test]
    fn selected_content_leaves_includes_inside_literal_regions() {
        let source = "before @@[[include component:x]]@@ after";
        let mut fragments = CompatTextFragments::new(source);
        let protected = protect_selected_content_includes(source, &mut fragments);

        assert!(protected.contains("[[include component:x]]"), "{protected}");
        assert_eq!(protected, source);
    }

    #[test]
    fn selected_content_protects_spaced_and_leading_include_openings() {
        // Whitespace between `[[` and the keyword is allowed.
        let spaced = "x [[ include component:y]] z";
        let mut fragments = CompatTextFragments::new(spaced);
        let protected = protect_selected_content_includes(spaced, &mut fragments);
        assert!(
            !protected.contains("[[ include component:y]]"),
            "{protected}"
        );
        assert_eq!(fragments.restore(&protected), spaced);

        // An opening at index 0 is protected without underflowing.
        let leading = "[[include a]] tail";
        let mut fragments = CompatTextFragments::new(leading);
        let protected = protect_selected_content_includes(leading, &mut fragments);
        assert!(!protected.contains("[[include a]]"), "{protected}");
        assert_eq!(fragments.restore(&protected), leading);
    }
}
