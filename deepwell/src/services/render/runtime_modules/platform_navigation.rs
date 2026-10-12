//! Wikijump platform navigation for the default site sidebar.

use super::{
    Cow, DomainService, LazyLock, LiteralRegionIndex, Reference, Regex, Result,
    ServiceContext, SiteService, WikitextSettings,
};

static PLATFORM_NAVIGATION_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[module\s+PlatformNavigation\s*\]\]")
        .expect("PlatformNavigation module expression is valid")
});

#[cfg(test)]
const PLATFORM_NAVIGATION_MODULE: &str = "[[module PlatformNavigation]]";

pub(super) async fn expand(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    settings: &WikitextSettings,
) -> Result<String> {
    if !settings.enable_page_syntax
        || !PLATFORM_NAVIGATION_MODULE_REGEX.is_match(&wikitext)
    {
        return Ok(wikitext);
    }

    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    let mut domain: Option<String> = None;
    for matched in PLATFORM_NAVIGATION_MODULE_REGEX.find_iter(&wikitext) {
        if literal_regions.contains(matched.start()) {
            continue;
        }
        output.push_str(&wikitext[cursor..matched.start()]);
        let platform_origin = match domain.as_ref() {
            Some(domain) => domain.clone(),
            None => {
                let platform_site =
                    SiteService::get(ctx, Reference::Slug(Cow::Borrowed("www"))).await?;
                let resolved_domain =
                    DomainService::preferred_https_origin(ctx.config(), &platform_site);
                domain = Some(resolved_domain.clone());
                resolved_domain
            }
        };
        output.push_str(&render_platform_navigation(&platform_origin));
        cursor = matched.end();
    }
    if cursor == 0 {
        return Ok(wikitext);
    }
    output.push_str(&wikitext[cursor..]);
    Ok(output)
}

fn render_platform_navigation(origin: &str) -> String {
    format!(
        concat!(
            "* [{origin}/platform:activity Recent activity]\n",
            "* [{origin}/platform:sites All wikis]\n",
            "* [{origin}/platform:search Search]",
        ),
        origin = origin,
    )
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{PLATFORM_NAVIGATION_MODULE, render_platform_navigation};
    use crate::services::render::RenderService;
    use crate::services::render::compat::CompatHtmlFragments;
    use crate::services::render::compat::text_fragments::CompatTextFragments;
    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};

    #[test]
    fn platform_navigation_uses_the_selected_www_origin_and_wikidot_link_renderer() {
        let seeded_sidebar = include_str!("../../../../seeder/sidebar.ftml");
        assert!(seeded_sidebar.contains(PLATFORM_NAVIGATION_MODULE));
        assert!(!seeded_sidebar.contains("[[[:www:platform:"));

        let source = render_platform_navigation("https://platform.example.test:18445");
        assert!(source.contains("https://platform.example.test:18445/platform:activity"));
        assert!(source.contains("https://platform.example.test:18445/platform:sites"));
        assert!(source.contains("https://platform.example.test:18445/platform:search"));

        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let page_info = PageInfo {
            page: Cow::Borrowed("nav:side"),
            category: None,
            site: Cow::Borrowed("example-wiki"),
            title: Cow::Borrowed("Navigation"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let mut compat_text = CompatTextFragments::new(&source);
        let mut compat_html = CompatHtmlFragments::new(&source);
        let mut protected = RenderService::finalize_runtime_module_residuals(
            source.clone(),
            &settings,
            false,
            &mut compat_text,
            &mut compat_html,
        );
        ftml::preprocess_for_layout(&mut protected, settings.layout);
        let tokens = ftml::tokenize(&protected);
        let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let rendered = compat_html.restore(&rendered);
        let rendered = compat_text.restore(&rendered);

        for href in [
            "https://platform.example.test:18445/platform:activity",
            "https://platform.example.test:18445/platform:sites",
            "https://platform.example.test:18445/platform:search",
        ] {
            assert!(rendered.contains(href), "missing {href}: {rendered}");
        }
        assert!(!rendered.contains(PLATFORM_NAVIGATION_MODULE));
    }
}
