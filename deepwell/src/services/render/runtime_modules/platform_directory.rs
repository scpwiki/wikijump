//! Native read-only views for the seeded platform directory pages.

use super::{
    CompatHtmlFragments, ConnectionTrait, DomainService, Error, ErrorType,
    FromQueryResult, LazyLock, LiteralRegionIndex, PageInfo, Regex, Result, ResultExt,
    ServiceContext, Statement, UrlArguments, WikitextSettings,
    escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::models::site::{Column as SiteColumn, Entity as Site};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{Action, Permission, Resource};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

const SITES_PER_PAGE: usize = 20;
const MAX_SITE_SCAN: usize = 1_001;
const ACTIVITY_ROWS: usize = 10;
const ACTIVITY_SCAN: usize = 500;

static PLATFORM_SITES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[module\s+PlatformSites\s*\]\]")
        .expect("PlatformSites module expression is valid")
});
static PLATFORM_ACTIVITY_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[module\s+PlatformActivity\s*\]\]")
        .expect("PlatformActivity module expression is valid")
});

#[derive(Debug, FromQueryResult)]
struct ActivityCandidate {
    site_id: i64,
    site_name: String,
    preferred_domain: Option<String>,
    page_id: i64,
    page_category_id: i64,
    page_slug: String,
    page_title: String,
    revision_number: i32,
    created_at: time::OffsetDateTime,
}

pub(super) async fn expand(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    settings: &WikitextSettings,
    page_info: &PageInfo<'_>,
    viewer_user_id: Option<i64>,
    url: UrlArguments<'_>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if !settings.enable_page_syntax {
        return Ok(wikitext);
    }
    let wikitext = upgrade_seeded_platform_page(wikitext, page_info);

    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    for matched in PLATFORM_SITES_MODULE_REGEX.find_iter(&wikitext) {
        if literal_regions.contains(matched.start()) {
            continue;
        }
        output.push_str(&wikitext[cursor..matched.start()]);
        let html = render_sites(ctx, viewer_user_id, url.page.unwrap_or(1)).await?;
        output.push_str(&retain_rendered_html(html, compat_html));
        cursor = matched.end();
    }
    if cursor > 0 {
        output.push_str(&wikitext[cursor..]);
    } else {
        return expand_activity(ctx, wikitext, settings, viewer_user_id, compat_html)
            .await;
    }

    expand_activity(ctx, output, settings, viewer_user_id, compat_html).await
}

fn upgrade_seeded_platform_page(wikitext: String, page_info: &PageInfo<'_>) -> String {
    if page_info.site != "www" || page_info.category.as_deref() != Some("platform") {
        return wikitext;
    }

    match page_info.page.as_ref() {
        "activity"
            if [
                "RecentWRevisions",
                "MostActiveSites",
                "MostActiveForums",
                "NewWUsers",
                "SomeGlobalStats",
            ]
            .iter()
            .all(|module| legacy_module_is_present(&wikitext, module)) =>
        {
            include_str!("../../../../seeder/platform-activity.ftml").to_owned()
        }
        "sites" if legacy_module_is_present(&wikitext, "ListAllWikis") => {
            include_str!("../../../../seeder/platform-sites.ftml").to_owned()
        }
        _ => wikitext,
    }
}

fn legacy_module_is_present(wikitext: &str, module: &str) -> bool {
    let pattern = format!(r"(?i)\[\[module\s+{module}\s*\]\]");
    Regex::new(&pattern)
        .expect("legacy platform module expression is valid")
        .is_match(wikitext)
}

async fn expand_activity(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    settings: &WikitextSettings,
    viewer_user_id: Option<i64>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if !settings.enable_page_syntax {
        return Ok(wikitext);
    }
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    for matched in PLATFORM_ACTIVITY_MODULE_REGEX.find_iter(&wikitext) {
        if literal_regions.contains(matched.start()) {
            continue;
        }
        output.push_str(&wikitext[cursor..matched.start()]);
        output.push_str(&retain_rendered_html(
            render_activity(ctx, viewer_user_id).await?,
            compat_html,
        ));
        cursor = matched.end();
    }
    if cursor == 0 {
        return Ok(wikitext);
    }
    output.push_str(&wikitext[cursor..]);
    Ok(output)
}

async fn render_sites(
    ctx: &ServiceContext<'_>,
    viewer_user_id: Option<i64>,
    page: u32,
) -> Result<String> {
    let make_error =
        || Error::new("failed to load public site directory", ErrorType::Render);
    let requested_page = page.clamp(1, 50) as usize;
    let visible_target = requested_page * SITES_PER_PAGE + 1;
    let mut raw_offset = 0usize;
    let mut visible = Vec::with_capacity(visible_target);
    // Filter before applying the page window; hidden sites never affect visible offsets.
    while raw_offset < MAX_SITE_SCAN && visible.len() < visible_target {
        let batch_size = (MAX_SITE_SCAN - raw_offset).min(100);
        let sites = Site::find()
            .filter(SiteColumn::DeletedAt.is_null())
            .order_by_asc(SiteColumn::Slug)
            .offset(raw_offset as u64)
            .limit(batch_size as u64)
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;
        if sites.is_empty() {
            break;
        }
        raw_offset += sites.len();
        for site in sites {
            let can_view = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: viewer_user_id,
                    site_id: site.site_id,
                    page_reference: None,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: None,
                    action: Action::View,
                },
            )
            .await
            .or_raise(make_error)?;
            if can_view {
                visible.push(site);
                if visible.len() == visible_target {
                    break;
                }
            }
        }
        if raw_offset >= MAX_SITE_SCAN || visible.len() == visible_target {
            break;
        }
    }

    let start = (requested_page - 1) * SITES_PER_PAGE;
    let end = (start + SITES_PER_PAGE).min(visible.len());
    let selected = visible.get(start..end).unwrap_or_default();
    let mut html = String::from("<div class=\"platform-site-list\"><ul>");
    for site in selected {
        let origin = DomainService::preferred_https_origin(ctx.config(), site);
        let href = format!("{origin}/");
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            escape_list_pages_html_attr(&href),
            escape_list_pages_html_text(&site.name),
        ));
    }
    if selected.is_empty() {
        html.push_str("<li class=\"platform-empty\">No sites are available.</li>");
    }
    html.push_str("</ul><nav class=\"platform-pagination\" aria-label=\"Site pages\">");
    if requested_page > 1 {
        html.push_str(&format!(
            "<a rel=\"prev\" href=\"/platform:sites/p/{}\">Previous</a>",
            requested_page - 1,
        ));
    }
    if visible.len() > start + SITES_PER_PAGE {
        html.push_str(&format!(
            "<a rel=\"next\" href=\"/platform:sites/p/{}\">Next</a>",
            requested_page + 1,
        ));
    }
    html.push_str("</nav></div>");
    Ok(html)
}

async fn render_activity(
    ctx: &ServiceContext<'_>,
    viewer_user_id: Option<i64>,
) -> Result<String> {
    let make_error =
        || Error::new("failed to load visible recent activity", ErrorType::Render);
    let candidates = ActivityCandidate::find_by_statement(Statement::from_sql_and_values(
        ctx.transaction().get_database_backend(),
        format!(
            concat!(
                "SELECT s.site_id, s.name AS site_name, s.preferred_domain, ",
                "p.page_id, p.page_category_id, p.slug AS page_slug, r.title AS page_title, ",
                "r.revision_number, r.created_at ",
                "FROM page_revision r ",
                "JOIN page p ON p.page_id = r.page_id AND p.site_id = r.site_id AND p.deleted_at IS NULL ",
                "JOIN site s ON s.site_id = r.site_id AND s.deleted_at IS NULL ",
                "WHERE r.revision_type IN ('regular', 'rollback', 'undo', 'create', 'move') ",
                "ORDER BY r.created_at DESC, r.revision_id DESC LIMIT {limit}"
            ),
            limit = ACTIVITY_SCAN
        ),
        [],
    ))
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;

    let mut html = String::from("<ul class=\"platform-recent-activity\">");
    let mut emitted = 0;
    for item in candidates {
        let can_view = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: viewer_user_id,
                site_id: item.site_id,
                page_reference: Some(crate::types::Reference::Id(item.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(crate::types::Reference::Id(
                    item.page_category_id,
                )),
                action: Action::View,
            },
        )
        .await
        .or_raise(make_error)?;
        if !can_view {
            continue;
        }
        let origin = match item.preferred_domain.as_deref() {
            Some(domain) => {
                DomainService::preferred_https_origin_for_domain(ctx.config(), domain)
            }
            None => {
                let site = Site::find_by_id(item.site_id)
                    .one(ctx.transaction())
                    .await
                    .or_raise(make_error)?
                    .ok_or_else(make_error)?;
                DomainService::preferred_https_origin(ctx.config(), &site)
            }
        };
        let href = format!("{origin}/{}", item.page_slug);
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a> <span class=\"platform-activity-site\">{}</span> <time datetime=\"{}\">{}</time> <span class=\"platform-revision-number\">(revision {})</span></li>",
            escape_list_pages_html_attr(&href),
            escape_list_pages_html_text(&item.page_title),
            escape_list_pages_html_text(&item.site_name),
            item.created_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            item.created_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            item.revision_number,
        ));
        emitted += 1;
        if emitted == ACTIVITY_ROWS {
            break;
        }
    }
    if emitted == 0 {
        html.push_str("<li class=\"platform-empty\">No visible recent edits.</li>");
    }
    html.push_str("</ul>");
    Ok(html)
}

fn retain_rendered_html(html: String, compat_html: &mut CompatHtmlFragments) -> String {
    compat_html.push_block_html(html)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};

    use super::{retain_rendered_html, upgrade_seeded_platform_page};
    use crate::services::render::RenderService;
    use crate::services::render::compat::CompatHtmlFragments;
    use crate::services::render::compat::text_fragments::CompatTextFragments;

    #[test]
    fn seeded_platform_pages_use_owned_runtime_modules() {
        let activity = include_str!("../../../../seeder/platform-activity.ftml");
        let sites = include_str!("../../../../seeder/platform-sites.ftml");
        assert!(activity.contains("[[module PlatformActivity]]"));
        assert!(!activity.contains("[[module RecentWRevisions]]"));
        assert!(!activity.contains("[[module MostActiveSites]]"));
        assert!(!activity.contains("[[module MostActiveForums]]"));
        assert!(!activity.contains("[[module NewWUsers]]"));
        assert!(!activity.contains("[[module SomeGlobalStats]]"));
        assert!(sites.contains("[[module PlatformSites]]"));
        assert!(!sites.contains("[[module ListAllWikis]]"));
    }

    #[test]
    fn persisted_first_party_pages_upgrade_without_rewriting_user_pages() {
        let mut page_info = PageInfo {
            page: Cow::Borrowed("activity"),
            category: Some(Cow::Borrowed("platform")),
            site: Cow::Borrowed("www"),
            title: Cow::Borrowed("Activity"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let legacy_activity = concat!(
            "[[module RecentWRevisions]] [[module MostActiveSites]] ",
            "[[module MostActiveForums]] [[module NewWUsers]] ",
            "[[module SomeGlobalStats]]",
        );
        assert_eq!(
            upgrade_seeded_platform_page(legacy_activity.to_owned(), &page_info),
            include_str!("../../../../seeder/platform-activity.ftml"),
        );

        page_info.page = Cow::Borrowed("sites");
        assert_eq!(
            upgrade_seeded_platform_page(
                "[[module ListAllWikis]]".to_owned(),
                &page_info,
            ),
            include_str!("../../../../seeder/platform-sites.ftml"),
        );

        page_info.site = Cow::Borrowed("example");
        let authored = "[[module ListAllWikis]]".to_owned();
        assert_eq!(
            upgrade_seeded_platform_page(authored.clone(), &page_info),
            authored,
        );
    }

    #[test]
    fn directory_html_remains_html_through_wikitext_rendering() {
        let html = concat!(
            "<div class=\"platform-site-list\"><ul>",
            "<li><a href=\"https://public.wikijump.localhost/\">Public site</a></li>",
            "</ul></div>",
        );
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let page_info = PageInfo {
            page: Cow::Borrowed("sites"),
            category: Some(Cow::Borrowed("platform")),
            site: Cow::Borrowed("www"),
            title: Cow::Borrowed("All sites"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let mut compat_text = CompatTextFragments::new(html);
        let mut compat_html = CompatHtmlFragments::new(html);
        let source = retain_rendered_html(html.to_owned(), &mut compat_html);
        let mut protected = RenderService::finalize_runtime_module_residuals(
            source,
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

        assert!(rendered.contains(html), "{rendered}");
        assert!(!rendered.contains("&lt;div class=\"platform-site-list\""));
    }
}
