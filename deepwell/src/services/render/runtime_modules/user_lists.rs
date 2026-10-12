//! Runtime expansion for ListUsers and ListDrafts.

use crate::services::PageDraftPageType;

use super::{
    CompatHtmlFragments, LISTDRAFTS_EMPTY_HTML, LISTDRAFTS_MODULE_REGEX,
    LISTUSERS_MODULE_REGEX, LISTUSERS_UNSUPPORTED_USERS_ERROR, LiteralRegionIndex,
    PageDraftService, PageDraftView, RenderService, Result, ServiceContext,
    WikidotModuleArgumentValueKind, WikitextSettings, escape_list_pages_html_attr,
    escape_list_pages_html_text, resolve_list_users_viewer,
    substitute_list_users_variables, wikidot_module_argument, wikidot_module_arguments,
};

impl RenderService {
    pub(super) async fn expand_list_users_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !LISTUSERS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let viewer = resolve_list_users_viewer(ctx, viewer_user_id).await?;
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in LISTUSERS_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a ListUsers capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |head| head.as_str());
            let body = captures.name("body").map_or("", |body| body.as_str());
            if wikidot_module_argument(head, "users") != Some(".") {
                output.push_str(&compat_html.push_block_html(format!(
                    r#"<div class="error-block">{}</div>"#,
                    LISTUSERS_UNSUPPORTED_USERS_ERROR,
                )));
            } else if let Some(viewer) = &viewer {
                output.push_str(&substitute_list_users_variables(body, viewer));
            }
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    pub(super) async fn expand_list_drafts_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !LISTDRAFTS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let Some(site_id) = current_site_id else {
            return Ok(Self::expand_list_drafts_without_site(
                wikitext,
                &literal_regions,
                compat_html,
            ));
        };
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in LISTDRAFTS_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a ListDrafts capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |head| head.as_str());
            let drafts = match list_drafts_page_type(head) {
                Some(page_type) => {
                    PageDraftService::list_for_viewer(
                        ctx,
                        site_id,
                        page_type,
                        viewer_user_id,
                    )
                    .await?
                }
                None => Vec::new(),
            };
            output.push_str(&compat_html.push_html(Self::render_list_drafts(&drafts)));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    pub(super) fn render_list_drafts(drafts: &[PageDraftView]) -> String {
        if drafts.is_empty() {
            return LISTDRAFTS_EMPTY_HTML.to_owned();
        }

        let mut output = String::from("<div class=\"list-drafts-box\">\n");
        for draft in drafts {
            output.push_str("            <div class=\"list-drafts-item\">\n");
            output.push_str("                <p><a href=\"/");
            output.push_str(&escape_list_pages_html_attr(&draft.slug));
            output.push_str("\">");
            output.push_str(&escape_list_pages_html_text(&draft.title));
            output.push_str("</a></p>\n            </div>\n");
        }
        output.push_str("            </div>");
        output
    }

    fn expand_list_drafts_without_site(
        wikitext: String,
        literal_regions: &LiteralRegionIndex,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in LISTDRAFTS_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a ListDrafts capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_html(LISTDRAFTS_EMPTY_HTML.to_owned()));
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }
}

fn list_drafts_page_type(head: &str) -> Option<PageDraftPageType> {
    if head.trim() == "pageType" {
        return Some(PageDraftPageType::All);
    }
    let arguments = wikidot_module_arguments(head)?;
    let exists = arguments.len() == 1
        && arguments.iter().any(|argument| {
            argument.key == "pageType"
                && argument.op == "="
                && argument.value_kind == WikidotModuleArgumentValueKind::DoubleQuoted
                && argument.value == "exists"
        });
    Some(if exists {
        PageDraftPageType::Exists
    } else {
        PageDraftPageType::All
    })
}

#[cfg(test)]
mod list_drafts_tests {
    use super::{PageDraftPageType, PageDraftView, RenderService, list_drafts_page_type};

    #[test]
    fn page_type_filter_accepts_only_the_observed_exact_form() {
        assert_eq!(
            list_drafts_page_type(r#" pageType="exists""#),
            Some(PageDraftPageType::Exists),
        );
        for head in [
            "",
            r#" pageType="notexists""#,
            r#" pageType="""#,
            r#" pageType="other""#,
            " pageType='exists'",
            " pageType=exists",
            r#" PAGETYPE="exists""#,
            r#" pageType!="exists""#,
            r#" pageType="exists" other="value""#,
            r#" pageType="exists" pageType="notexists""#,
        ] {
            assert_eq!(
                list_drafts_page_type(head),
                Some(PageDraftPageType::All),
                "head: {head:?}",
            );
        }
        assert_eq!(
            list_drafts_page_type(" pageType"),
            Some(PageDraftPageType::All),
        );
        assert_eq!(list_drafts_page_type(" malformed bare"), None);
    }

    #[test]
    fn renderer_keeps_the_observed_row_hierarchy_and_escapes_values() {
        let html = RenderService::render_list_drafts(&[PageDraftView {
            slug: "run-owned:fixture".to_owned(),
            title: "Draft <one>".to_owned(),
        }]);
        assert!(html.contains(r#"<div class="list-drafts-box">"#));
        assert!(html.contains(r#"<div class="list-drafts-item">"#));
        assert!(
            html.contains(r#"<p><a href="/run-owned:fixture">Draft &lt;one&gt;</a></p>"#)
        );
    }
}
