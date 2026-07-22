/*
 * services/view/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

//! The view service, processing high-level requests to Framerail for rendering web routes.
//!
//! This is one of the highest-level services, as it bundles the data from numerous
//! other services into responses which Framerail can use when rendering specific routes.
//! For instance, the `PageView` structure represents a request to any page (i.e. `/slug`),
//! gathering all the relevant data and sending it back in one convenient `PageViewOutput`
//! response.
//!
//! The service also contains the core method `ViewService::get_viewer()`, which converts the
//! requesting domain and session token into a site and user, respectively.

use super::article_cache::ArticlePageCache;
use super::prelude::*;
use super::redirect::wikidot_redirect_location;
use crate::license::WikidotLicense;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::site::Model as SiteModel;
use crate::services::blueprint::{BlueprintPageType, GetBlueprintPageOutput};
use crate::services::page_revision::RerenderType;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::relation::{
    GetPageAttributions, GetSiteBan, PageAttribution, RelationService,
};
use crate::services::render::RenderOutput;
use crate::services::settings::{NavigationPageHtml, SettingsService};
use crate::services::view::ViewType;
use crate::services::{
    BlueprintPageService, CategoryService, DomainService, PageRevisionService,
    PageService, SessionService, SiteService, TextService, UserService,
};
use crate::types::{Action, PageId, PageOrder, Permission, RerenderDepth, Resource};
use crate::utils::{get_category_name, parse_locales, split_category};
use ftml::prelude::*;
use ftml::render::html::HtmlOutput;
use ref_map::*;
use sea_orm::{FromQueryResult, Statement};
use std::borrow::Cow;
use std::mem;
use time::OffsetDateTime;
use unic_langid::LanguageIdentifier;
use wikidot_normalize::normalize;

fn wikidot_redirect_module_allowed(
    page: &PageModel,
    page_revision: &PageRevisionModel,
) -> bool {
    // Wikidot Redirect modules are compatibility behavior for imported Wikidot
    // pages only. Requiring both the page and the served revision to carry
    // import provenance prevents ordinary editable wikitext from creating
    // permanent external redirects on the local Wikijump domain.
    page.from_wikidot && page_revision.from_wikidot
}

#[derive(Debug)]
pub struct ViewService;

impl ViewService {
    pub async fn article(
        ctx: &ServiceContext<'_>,
        mut input: GetPageView,
    ) -> Result<GetArticleViewOutput> {
        let mut preload = Self::preload(
            ctx,
            GetPreloadView {
                site_id: input.site_id,
                session_token: input.session_token.clone(),
                locales: input.locales.clone(),
            },
        )
        .await?;
        if let Some(user_session) = &preload.viewer.user_session {
            let mut locales = user_session.user.locales.clone();
            locales.extend(
                input
                    .locales
                    .iter()
                    .filter(|locale| !user_session.user.locales.contains(locale))
                    .cloned(),
            );
            input.locales = locales;
        }
        if !input.locales.contains(&preload.viewer.site.locale) {
            input.locales.push(preload.viewer.site.locale.clone());
        }
        Self::apply_article_category_license(ctx, &mut preload.viewer, &input).await?;
        let cache_metadata = ArticlePageCache::metadata(ctx, &input).await?;
        if let Some(cache_key) =
            cache_metadata.as_ref().map(|metadata| &metadata.cache_key)
            && let Some(mut page) = ArticlePageCache::get(ctx, cache_key).await?
            && Self::cached_article_page_visible_to_viewer(ctx, &preload.viewer, &input)
                .await?
        {
            if let GetPageViewOutput::Found {
                page: page_model,
                wikidot_breadcrumbs,
                ..
            } = &mut page
                && page_model.from_wikidot
            {
                *wikidot_breadcrumbs = Self::get_wikidot_breadcrumbs(
                    ctx,
                    page_model.site_id,
                    preload
                        .viewer
                        .user_session
                        .as_ref()
                        .map(|session| session.user.user_id),
                    page_model.page_id,
                )
                .await?;
            }

            return Ok(GetArticleViewOutput {
                viewer: preload.viewer,
                page,
                article_page_cache_key: Some(cache_key.clone()),
                public_content_cache_fence: cache_metadata
                    .as_ref()
                    .map(|metadata| metadata.public_content_cache_fence.clone()),
                anonymous_permission_cache_fence: cache_metadata
                    .as_ref()
                    .map(|metadata| metadata.anonymous_permission_cache_fence.clone()),
            });
        }

        let page_view = Self::page(ctx, input).await?;
        if let (Some(cache_key), GetPageViewOutput::Found { page, .. }) = (
            cache_metadata.as_ref().map(|metadata| &metadata.cache_key),
            &page_view,
        ) && page.from_wikidot
        {
            ArticlePageCache::set(ctx, cache_key, &page_view).await?;
        }

        Ok(GetArticleViewOutput {
            viewer: preload.viewer,
            page: page_view,
            article_page_cache_key: cache_metadata
                .as_ref()
                .map(|metadata| metadata.cache_key.clone()),
            public_content_cache_fence: cache_metadata
                .as_ref()
                .map(|metadata| metadata.public_content_cache_fence.clone()),
            anonymous_permission_cache_fence: cache_metadata
                .as_ref()
                .map(|metadata| metadata.anonymous_permission_cache_fence.clone()),
        })
    }

    async fn apply_article_category_license(
        ctx: &ServiceContext<'_>,
        viewer: &mut Viewer,
        input: &GetPageView,
    ) -> Result<()> {
        let page_full_slug = input
            .route
            .as_ref()
            .map_or(viewer.site.default_page.as_str(), |route| {
                route.slug.as_str()
            });
        let (category_slug, _) = split_category(page_full_slug);
        let category_slug = category_slug.unwrap_or("_default");
        let category_id =
            Self::get_category_id(ctx, viewer.site.site_id, Some(category_slug)).await?;
        let license =
            SettingsService::get_license(ctx, viewer.site.site_id, category_id).await?;
        let locales = parse_locales(&input.locales)?;
        match license {
            WikidotLicense::Standard(license) => {
                viewer.license_name = license.translate(ctx.localization(), &locales)?;
                viewer.license_url = license.url();
                viewer.license_kind = ViewerLicenseKind::Standard;
                viewer.license_html = None;
            }
            WikidotLicense::Other(html) => {
                viewer.license_name.clear();
                viewer.license_url = "";
                viewer.license_kind = ViewerLicenseKind::Other;
                viewer.license_html =
                    Some(html.replace(
                        "%%year%%",
                        &OffsetDateTime::now_utc().year().to_string(),
                    ));
            }
            WikidotLicense::Copyright => {
                viewer.license_name.clear();
                viewer.license_url = "";
                viewer.license_kind = ViewerLicenseKind::Copyright;
                viewer.license_html = Some(String::new());
            }
        }
        Ok(())
    }

    pub async fn article_cache_metadata(
        ctx: &ServiceContext<'_>,
        mut input: GetPageView,
    ) -> Result<GetArticleViewCacheMetadataOutput> {
        if !matches!(input.session_token.as_deref(), None | Some("")) {
            return Ok(GetArticleViewCacheMetadataOutput {
                article_page_cache_key: None,
                public_content_cache_fence: None,
                anonymous_permission_cache_fence: None,
            });
        }

        let preload = Self::preload(
            ctx,
            GetPreloadView {
                site_id: input.site_id,
                session_token: None,
                locales: input.locales.clone(),
            },
        )
        .await?;
        if !input.locales.contains(&preload.viewer.site.locale) {
            input.locales.push(preload.viewer.site.locale);
        }

        let cache_metadata = ArticlePageCache::metadata(ctx, &input).await?;

        Ok(GetArticleViewCacheMetadataOutput {
            article_page_cache_key: cache_metadata
                .as_ref()
                .map(|metadata| metadata.cache_key.clone()),
            public_content_cache_fence: cache_metadata
                .as_ref()
                .map(|metadata| metadata.public_content_cache_fence.clone()),
            anonymous_permission_cache_fence: cache_metadata
                .as_ref()
                .map(|metadata| metadata.anonymous_permission_cache_fence.clone()),
        })
    }

    async fn cached_article_page_visible_to_viewer(
        ctx: &ServiceContext<'_>,
        viewer: &Viewer,
        input: &GetPageView,
    ) -> Result<bool> {
        let page_full_slug = input
            .route
            .as_ref()
            .map_or(viewer.site.default_page.as_str(), |route| {
                route.slug.as_str()
            });
        let (category_slug, _) = split_category(page_full_slug);
        let category_id =
            Self::get_category_id(ctx, viewer.site.site_id, category_slug).await?;

        PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: viewer
                    .user_session
                    .as_ref()
                    .map(|session| session.user.user_id),
                site_id: viewer.site.site_id,
                page_reference: None,
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: category_id.map(Reference::Id),
                action: Action::View,
            },
        )
        .await
    }

    pub async fn preload(
        ctx: &ServiceContext<'_>,
        GetPreloadView {
            site_id,
            locales: locales_str,
            session_token,
        }: GetPreloadView,
    ) -> Result<GetPreloadViewOutput> {
        info!("Getting preload data for site ID {site_id}, locales '{locales_str:?}'");

        let make_error = || {
            Error::new(
                format!(
                    "failed to get preload data for site ID {}, locales '{:?}'",
                    site_id, locales_str,
                ),
                ErrorType::GetView(ViewType::Preload),
            )
        };

        let mut locales = parse_locales(&locales_str)?;
        let viewer = Self::get_viewer(
            ctx,
            &mut locales,
            site_id,
            session_token.ref_map(|s| s.as_str()),
            ViewType::Preload,
        )
        .await
        .or_raise(make_error)?;

        Ok(GetPreloadViewOutput { viewer })
    }

    pub async fn page(
        ctx: &ServiceContext<'_>,
        GetPageView {
            site_id,
            locales: locales_str,
            route,
            session_token,
        }: GetPageView,
    ) -> Result<GetPageViewOutput> {
        info!(
            "Getting page view data for site ID {site_id}, route '{route:?}', locales '{locales_str:?}'"
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to generate page view for site ID {}, route '{:?}', locales '{:?}'",
                    site_id, route, locales_str,
                ),
                ErrorType::GetView(ViewType::Page),
            )
        };

        let locales = parse_locales(&locales_str)?;
        let config = ctx.config();

        // Get site information
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let user_session = Self::get_session(ctx, session_token.ref_map(|s| s.as_str()))
            .await
            .or_raise(make_error)?;

        // If None, means the main page for the site. Pull from site data.
        let (page_full_slug, page_extra): (&str, &str) = match &route {
            None => (&site.default_page, ""),
            Some(PageRoute { slug, extra }) => (slug, extra),
        };

        let redirect_page = Self::should_redirect_page(page_full_slug);
        let options = PageOptions::parse(page_extra);

        // Get page, revision, and text fields
        let (category_slug, page_only_slug) = split_category(page_full_slug);
        let category_id = Self::get_category_id(ctx, site_id, category_slug)
            .await
            .or_raise(make_error)?;
        let page_info = PageInfo {
            page: cow!(page_only_slug),
            category: cow_opt!(category_slug),
            site: cow!(&site.slug),
            title: cow!(page_only_slug),
            alt_title: None,
            score: ScoreValue::Integer(0), // TODO configurable default score value
            tags: vec![],
            // NOTE: The [[date]] block is now localized through
            // PageInfo.language, which means the value passed here
            // should be &site.locale.
            //
            // If we ever want to change this, we need to evaluate the
            // impact on FTML first.
            language: cow!(&site.locale),
        };

        // Helper structures to designate which variant of GetPageViewOutput to return.

        #[derive(Debug)]
        enum PageStatus {
            Found {
                page: PageModel,
                page_revision: PageRevisionModel,
                wikidot_snapshot: Option<WikidotPageSnapshotView>,
                wikidot_breadcrumbs: Vec<WikidotPageBreadcrumbView>,
                attributions: Vec<PageAttribution>,
            },
            Missing,
            Private,
            Banned,
        }

        #[derive(Debug)]
        struct PageReturn {
            page_status: PageStatus,
            wikitext: String,
            new_page_wikitext: Option<String>,
            page_templates: Vec<PageTemplateSummary>,
            selected_template_page_id: Option<i64>,
            compiled_body_html: String,
            compiled_body_styles: Vec<String>,
            compiled_top_bar_html: Option<String>,
            compiled_side_bar_html: Option<String>,
        }

        // Get wikitext and HTML to return for this page.
        let PageReturn {
            page_status,
            wikitext,
            new_page_wikitext,
            page_templates,
            selected_template_page_id,
            compiled_body_html,
            compiled_body_styles,
            compiled_top_bar_html,
            compiled_side_bar_html,
        } = match PageService::get_optional(
            ctx,
            site.site_id,
            Reference::Slug(cow!(page_full_slug)),
        )
        .await
        .or_raise(make_error)?
        {
            // This page exists, return its data directly.
            Some(page) => {
                // Get associated revision
                let page_revision =
                    PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                        .await
                        .or_raise(make_error)?;

                // Check user access to page
                let [user_can_access_page, user_can_edit_page] =
                    PermissionService::batch_check_user_can(
                        ctx,
                        &CheckPermissionContext {
                            user_id: user_session.as_ref().map(|s| s.user.user_id),
                            site_id,
                            page_reference: None,
                        },
                        [
                            Permission {
                                resource_type: Resource::Page,
                                resource_category: category_id.map(Reference::Id),
                                action: Action::View,
                            },
                            Permission {
                                resource_type: Resource::Page,
                                resource_category: category_id.map(Reference::Id),
                                action: Action::Edit,
                            },
                        ],
                    )
                    .await
                    .or_raise(make_error)?;

                // Determine whether to return the actual page contents,
                // or the "private page" data (_public).
                //
                // This returns false if the user is banned *and* the site
                // disallows banned viewing.
                if user_can_access_page {
                    debug!("User has page access, return text data");

                    if options.rerender && user_can_edit_page {
                        let depth = RerenderDepth::default();
                        info!(
                            "Re-rendering revision: site ID {} page ID {} revision ID {} (depth {})",
                            page.site_id, page.page_id, page_revision.revision_id, depth,
                        );
                        PageRevisionService::rerender(
                            ctx,
                            PageId::from_page_model(&page),
                            depth,
                            RerenderType::Full,
                        )
                        .await
                        .or_raise(make_error)?;
                    };

                    let (
                        wikitext_result,
                        compiled_body_result,
                        compiled_body_styles_result,
                        compiled_top_bar_result,
                        compiled_side_bar_result,
                    ) = join!(
                        TextService::get(ctx, &page_revision.wikitext_hash),
                        TextService::get(ctx, &page_revision.compiled_body_html_hash),
                        TextService::get_option(
                            ctx,
                            &page_revision.compiled_body_styles_hash,
                        ),
                        TextService::get_option(
                            ctx,
                            &page_revision.compiled_top_bar_html_hash,
                        ),
                        TextService::get_option(
                            ctx,
                            &page_revision.compiled_side_bar_html_hash,
                        ),
                    );

                    let (
                        wikitext,
                        compiled_body_html,
                        compiled_body_styles,
                        compiled_top_bar_html,
                        compiled_side_bar_html,
                    ) = raise_multiple!(wikitext_result, compiled_body_result, compiled_body_styles_result, compiled_top_bar_result, compiled_side_bar_result; make_error);
                    let compiled_body_styles = compiled_body_styles
                        .map(|styles| serde_json::from_str(&styles))
                        .transpose()
                        .or_raise(make_error)?
                        .unwrap_or_default();

                    let attributions = RelationService::get_page_attributions(
                        ctx,
                        GetPageAttributions {
                            site_id: page.site_id,
                            page: Reference::Id(page.page_id),
                        },
                    )
                    .await
                    .or_raise(make_error)?;

                    let wikidot_snapshot =
                        Self::get_wikidot_snapshot_page_info(ctx, page.page_id)
                            .await
                            .or_raise(make_error)?;
                    let wikidot_breadcrumbs = Self::get_wikidot_breadcrumbs(
                        ctx,
                        site_id,
                        user_session.as_ref().map(|s| s.user.user_id),
                        page.page_id,
                    )
                    .await
                    .or_raise(make_error)?;

                    PageReturn {
                        page_status: PageStatus::Found {
                            page,
                            page_revision,
                            wikidot_snapshot,
                            wikidot_breadcrumbs,
                            attributions,
                        },
                        wikitext,
                        new_page_wikitext: None,
                        page_templates: Vec::new(),
                        selected_template_page_id: None,
                        compiled_body_html,
                        compiled_body_styles,
                        compiled_top_bar_html,
                        compiled_side_bar_html,
                    }
                } else {
                    warn!("User doesn't have page access, returning permission page");

                    let user_is_banned = match user_session {
                        Some(ref session) => RelationService::active_site_ban_exists(
                            ctx,
                            GetSiteBan {
                                site_id,
                                user_id: session.user.user_id,
                            },
                        )
                        .await
                        .or_raise(make_error)?,

                        // TODO: This will need to change when IP bans are implemented.
                        // For now, if user is not logged in, consider them not banned.
                        None => false,
                    };

                    let (page_status, page_type) = if user_is_banned {
                        (PageStatus::Banned, BlueprintPageType::Banned)
                    } else {
                        (PageStatus::Private, BlueprintPageType::Private)
                    };

                    let GetBlueprintPageOutput {
                        wikitext,
                        render_output,
                    } = BlueprintPageService::get(
                        ctx,
                        &site,
                        page_type,
                        &locales,
                        config.default_page_layout,
                        page_info,
                    )
                    .await
                    .or_raise(make_error)?;

                    let RenderOutput {
                        html_output:
                            HtmlOutput {
                                body: compiled_body_html,
                                styles: compiled_body_styles,
                                ..
                            },
                        ..
                    } = render_output;

                    // Even though the page isn't visible to this user,
                    // we display its nav pages since they're already
                    // set up for us.
                    let (compiled_top_bar_result, compiled_side_bar_result) = join!(
                        TextService::get_option(
                            ctx,
                            &page_revision.compiled_top_bar_html_hash,
                        ),
                        TextService::get_option(
                            ctx,
                            &page_revision.compiled_side_bar_html_hash,
                        ),
                    );
                    let (compiled_top_bar_html, compiled_side_bar_html) = raise_multiple!(compiled_top_bar_result, compiled_side_bar_result; make_error);

                    PageReturn {
                        page_status,
                        wikitext,
                        new_page_wikitext: None,
                        page_templates: Vec::new(),
                        selected_template_page_id: None,
                        compiled_body_html,
                        compiled_body_styles,
                        compiled_top_bar_html,
                        compiled_side_bar_html,
                    }
                }
            }
            // The page is missing, fetch the "missing page" data (_404).
            None => {
                let GetBlueprintPageOutput {
                    wikitext,
                    render_output,
                } = BlueprintPageService::get(
                    ctx,
                    &site,
                    BlueprintPageType::Missing,
                    &locales,
                    config.default_page_layout,
                    page_info,
                )
                .await
                .or_raise(make_error)?;

                let RenderOutput {
                    html_output:
                        HtmlOutput {
                            body: compiled_body_html,
                            styles: compiled_body_styles,
                            ..
                        },
                    ..
                } = render_output;

                let NavigationPageHtml {
                    compiled_top_bar_html,
                    compiled_side_bar_html,
                } = SettingsService::get_nav_page_html(ctx, site_id, category_id)
                    .await
                    .or_raise(make_error)?;
                let (page_templates, category_template_page_id) = if options.edit {
                    let create_category = CategoryService::get_optional(
                        ctx,
                        site_id,
                        Reference::Slug(cow!(get_category_name(page_full_slug))),
                    )
                    .await
                    .or_raise(make_error)?;
                    let user_can_create_page = match user_session.as_ref() {
                        Some(session) => PermissionService::check_user_can(
                            ctx,
                            &CheckPermissionContext {
                                user_id: Some(session.user.user_id),
                                site_id,
                                page_reference: None,
                            },
                            Permission {
                                resource_type: Resource::Page,
                                resource_category: create_category
                                    .as_ref()
                                    .map(|category| Reference::Id(category.category_id)),
                                action: Action::Create,
                            },
                        )
                        .await
                        .or_raise(make_error)?,
                        None => false,
                    };

                    if user_can_create_page {
                        (
                            Self::get_page_templates(ctx, site_id)
                                .await
                                .or_raise(make_error)?,
                            create_category
                                .and_then(|category| category.template_page_id),
                        )
                    } else {
                        (Vec::new(), None)
                    }
                } else {
                    (Vec::new(), None)
                };
                let selected_template_page_id = options
                    .template
                    .filter(|page_id| {
                        page_templates
                            .iter()
                            .any(|template| template.page_id == *page_id)
                    })
                    .or(category_template_page_id);
                let new_page_wikitext = selected_template_page_id.and_then(|page_id| {
                    page_templates
                        .iter()
                        .find(|template| template.page_id == page_id)
                        .map(|template| template.wikitext.clone())
                });

                PageReturn {
                    page_status: PageStatus::Missing,
                    wikitext,
                    new_page_wikitext,
                    page_templates,
                    selected_template_page_id,
                    compiled_body_html,
                    compiled_body_styles,
                    compiled_top_bar_html,
                    compiled_side_bar_html,
                }
            }
        };

        // TODO Check if user-agent and IP match?

        let (redirect_page, redirect_kind) = if let Some(redirect_page) = redirect_page {
            (Some(redirect_page), None)
        } else if let PageStatus::Found {
            page,
            page_revision,
            ..
        } = &page_status
            && wikidot_redirect_module_allowed(page, page_revision)
        {
            let redirect_page =
                wikidot_redirect_location(&wikitext, page_full_slug, options.no_redirect);
            let redirect_kind = redirect_page
                .as_ref()
                .map(|_| PageRedirectKind::WikidotModule);
            (redirect_page, redirect_kind)
        } else {
            (None, None)
        };

        let output = match page_status {
            PageStatus::Found {
                page,
                page_revision,
                wikidot_snapshot,
                wikidot_breadcrumbs,
                attributions,
            } => GetPageViewOutput::Found {
                options,
                page,
                page_revision,
                wikidot_snapshot,
                wikidot_breadcrumbs,
                attributions,
                redirect_page,
                redirect_kind,
                wikitext,
                compiled_body_html,
                compiled_body_styles,
                compiled_top_bar_html,
                compiled_side_bar_html,
            },
            PageStatus::Missing => GetPageViewOutput::Missing {
                options,
                redirect_page,
                redirect_kind,
                wikitext,
                new_page_wikitext,
                page_templates,
                selected_template_page_id,
                compiled_body_html,
                compiled_body_styles,
                compiled_top_bar_html,
                compiled_side_bar_html,
            },
            PageStatus::Private => GetPageViewOutput::Permissions {
                options,
                redirect_page,
                redirect_kind,
                compiled_body_html,
                compiled_body_styles,
                compiled_top_bar_html,
                compiled_side_bar_html,
                banned: false,
            },
            PageStatus::Banned => GetPageViewOutput::Permissions {
                options,
                redirect_page,
                redirect_kind,
                compiled_body_html,
                compiled_body_styles,
                compiled_top_bar_html,
                compiled_side_bar_html,
                banned: true,
            },
        };

        Ok(output)
    }

    async fn get_wikidot_snapshot_page_info(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<Option<WikidotPageSnapshotView>> {
        #[derive(FromQueryResult, Debug)]
        struct WikidotSnapshotRow {
            source_site: String,
            source_revision_count: Option<i32>,
            source_updated_at: Option<OffsetDateTime>,
            imported_rating: Option<i64>,
            comments: Option<i32>,
        }

        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT source_site, source_revision_count, source_updated_at, imported_rating, comments FROM wikidot_page_snapshot WHERE page_id = {}",
                page_id,
            ),
        );

        let row = WikidotSnapshotRow::find_by_statement(statement)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to load imported Wikidot page snapshot",
                    ErrorType::GetView(ViewType::Page),
                )
            })?;

        Ok(row.and_then(
            |WikidotSnapshotRow {
                 source_site,
                 source_revision_count,
                 source_updated_at,
                 imported_rating,
                 comments,
             }| {
                Some(WikidotPageSnapshotView {
                    source_site,
                    source_revision_count: source_revision_count?,
                    source_updated_at: source_updated_at?,
                    imported_rating,
                    comments,
                })
            },
        ))
    }

    async fn get_wikidot_breadcrumbs(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: Option<i64>,
        page_id: i64,
    ) -> Result<Vec<WikidotPageBreadcrumbView>> {
        #[derive(FromQueryResult, Debug)]
        struct WikidotBreadcrumbRow {
            source_fullname: String,
            title_shown: Option<String>,
            page_category_id: Option<i64>,
        }

        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                r#"
WITH RECURSIVE
breadcrumb_chain(depth, page_id, source_site, source_fullname, title_shown, parent_fullname) AS (
  SELECT 0, page_id, source_site, source_fullname, title_shown, parent_fullname
  FROM wikidot_page_snapshot
  WHERE page_id = {page_id}
  UNION ALL
  SELECT
    breadcrumb_chain.depth + 1,
    parent.page_id,
    parent.source_site,
    parent.source_fullname,
    parent.title_shown,
    parent.parent_fullname
  FROM breadcrumb_chain
  JOIN wikidot_page_snapshot parent
    ON parent.source_site = breadcrumb_chain.source_site
   AND parent.source_fullname = breadcrumb_chain.parent_fullname
  WHERE breadcrumb_chain.parent_fullname IS NOT NULL
    AND breadcrumb_chain.depth < 12
)
SELECT
  breadcrumb_chain.source_fullname,
  breadcrumb_chain.title_shown,
  page.page_category_id
FROM breadcrumb_chain
LEFT JOIN page
  ON page.page_id = breadcrumb_chain.page_id
 AND page.deleted_at IS NULL
ORDER BY breadcrumb_chain.depth ASC
"#,
            ),
        );

        let rows = WikidotBreadcrumbRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to load imported Wikidot page breadcrumb chain",
                    ErrorType::GetView(ViewType::Page),
                )
            })?;

        let mut breadcrumbs = Vec::new();
        for WikidotBreadcrumbRow {
            source_fullname,
            title_shown,
            page_category_id,
        } in rows
        {
            let Some(page_category_id) = page_category_id else {
                break;
            };
            let user_can_view_ancestor = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id,
                    site_id,
                    // View permissions are cached per category, not per page.
                    // Supplying an ancestor ID would make page-author roles leak
                    // one page-specific decision into other ancestors.
                    page_reference: None,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(page_category_id)),
                    action: Action::View,
                },
            )
            .await?;

            if !user_can_view_ancestor {
                break;
            }

            breadcrumbs.push(WikidotPageBreadcrumbView {
                title: title_shown.unwrap_or_else(|| source_fullname.clone()),
                slug: source_fullname,
            });
        }

        breadcrumbs.reverse();

        if breadcrumbs.len() <= 1 {
            breadcrumbs.clear();
        }

        Ok(breadcrumbs)
    }

    pub async fn user(
        ctx: &ServiceContext<'_>,
        GetUserView {
            site_id,
            locales: locales_str,
            user: user_ref,
            session_token,
        }: GetUserView<'_>,
    ) -> Result<GetUserViewOutput> {
        info!(
            "Getting user view data for site ID {site_id}, user '{user_ref:?}', locales '{locales_str:?}'"
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to generate user view for site ID {}, user '{:?}', locales '{:?}'",
                    site_id, user_ref, locales_str,
                ),
                ErrorType::GetView(ViewType::User),
            )
        };

        let user_session = Self::get_session(ctx, session_token.ref_map(|s| s.as_str()))
            .await
            .or_raise(make_error)?;

        // TODO Check if user-agent and IP match?

        // Get data to return for this user.
        let user = match user_ref {
            Some(ref user_ref) => UserService::get_optional(ctx, user_ref.borrow())
                .await
                .or_raise(make_error)?,

            // For users visiting their own user info page
            None => user_session.map(|session| session.user),
        };

        let output = match user {
            Some(user) => GetUserViewOutput::UserFound { user },
            None => GetUserViewOutput::UserMissing,
        };

        Ok(output)
    }

    async fn get_page_templates(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<PageTemplateSummary>> {
        if CategoryService::get_optional(ctx, site_id, Reference::from("template"))
            .await?
            .is_none()
        {
            return Ok(Vec::new());
        }

        let mut pages = PageService::get_all(
            ctx,
            site_id,
            Some(Reference::from("template")),
            Some(false),
            PageOrder::default(),
        )
        .await?;
        pages.sort_by(|left, right| left.slug.cmp(&right.slug));

        let mut templates = Vec::with_capacity(pages.len());
        for page in pages {
            let revision =
                PageRevisionService::get_latest(ctx, site_id, page.page_id).await?;
            let wikitext = TextService::get(ctx, &revision.wikitext_hash).await?;
            templates.push(PageTemplateSummary {
                page_id: page.page_id,
                slug: page.slug,
                title: revision.title,
                wikitext,
            });
        }

        Ok(templates)
    }

    pub async fn admin(
        ctx: &ServiceContext<'_>,
        GetAdminView {
            site_id,
            locales: locales_str,
            session_token,
        }: GetAdminView,
    ) -> Result<GetAdminViewOutput> {
        info!("Getting site view data for site ID {site_id}, locales '{locales_str:?}'");

        let make_error = || {
            Error::new(
                format!(
                    "failed to generate user view for site ID {}, locales '{:?}'",
                    site_id, locales_str,
                ),
                ErrorType::GetView(ViewType::Admin),
            )
        };

        let locales = parse_locales(&locales_str)?;
        let config = ctx.config();

        // Get site information
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let user_session = Self::get_session(ctx, session_token.ref_map(|s| s.as_str()))
            .await
            .or_raise(make_error)?;

        let page_info = PageInfo {
            page: cow!(""),
            category: cow_opt!(Some("admin")),
            title: cow!(""),
            alt_title: None,
            site: cow!(site.slug),
            score: ScoreValue::Integer(0),
            tags: vec![],
            language: cow!(site.locale),
        };

        let GetBlueprintPageOutput {
            wikitext: _,
            render_output,
        } = BlueprintPageService::get(
            ctx,
            &site,
            BlueprintPageType::Unauthorized,
            &locales,
            config.default_page_layout,
            page_info,
        )
        .await
        .or_raise(make_error)?;

        let RenderOutput {
            html_output:
                HtmlOutput {
                    body: compiled_html,
                    ..
                },
            ..
        } = render_output;

        // Check user access to site settings
        let user_id = match user_session {
            Some(ref session) => Some(session.user.user_id),
            None => {
                debug!("No user for session, disallow admin access");
                return Ok(GetAdminViewOutput::AdminPermissions {
                    html: compiled_html,
                });
            }
        };

        let user_can_access_admin = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id,
                site_id,
                page_reference: None,
            },
            Permission {
                resource_type: Resource::Site,
                resource_category: None,
                action: Action::Edit,
            },
        )
        .await
        .or_raise(make_error)?;

        // Determine whether to return the actual admin panel content
        let output = if user_can_access_admin {
            debug!("User has admin access, return data");
            let categories = CategoryService::get_all(ctx, site_id)
                .await
                .or_raise(make_error)?;
            let page_templates = Self::get_page_templates(ctx, site_id)
                .await
                .or_raise(make_error)?;
            GetAdminViewOutput::SiteFound {
                categories,
                page_templates,
            }
        } else {
            warn!("User doesn't have admin access, returning permission page");

            GetAdminViewOutput::AdminPermissions {
                html: compiled_html,
            }
        };

        Ok(output)
    }

    /// Gets basic data and runs common logic for all web routes.
    ///
    /// All views seen by end users require a few translations before
    /// a request can be serviced:
    ///
    /// * Site ID → Site data
    /// * Session token → User ID and their permissions
    ///
    /// Note that we do *not* need to get the site ID from the domain
    /// since WWS has already done the domain lookup logic for us.
    ///
    /// Then using this information, the caller can perform some common
    /// operations, such as slug normalization or redirect site aliases.
    pub async fn get_viewer(
        ctx: &ServiceContext<'_>,
        locales: &mut Vec<LanguageIdentifier>,
        site_id: i64,
        session_token: Option<&str>,
        view_type: ViewType,
    ) -> Result<Viewer> {
        info!("Getting viewer data site ID {site_id} and session token");

        let config = ctx.config();
        let make_error = || {
            Error::new(
                format!("failed to get common viewer for site ID {}", site_id),
                ErrorType::GetView(view_type),
            )
        };

        // Get user data from session token (if present)
        let user_session = match session_token {
            Some("") | None => None,
            Some(token) => {
                let session =
                    SessionService::get(ctx, token).await.or_raise(make_error)?;

                let user = UserService::get(ctx, Reference::Id(session.user_id))
                    .await
                    .or_raise(make_error)?;

                // Prefer what the user has set over what the browser is requesting
                {
                    // Get the list of user locales
                    //
                    // Our goal is to insert this list of user locales at the front.
                    // For instance, if the browser is requesting [X, Y], but the user
                    // prefers [A, B], we want to end up with [A, B, X, Y].
                    //
                    // But the most efficient method to use here is append().
                    // So we append all the requested locales to the end of the user
                    // locales we just got, then swap the contents.
                    //
                    // The end goal is that 'locales' ends up with the new locales at
                    // the start before the previous items, and 'user_locales' ends up
                    // drained since it was inserted into the preserved 'locales' vector.

                    let mut user_locales =
                        parse_locales(&user.locales).or_raise(make_error)?;
                    user_locales.append(locales);
                    mem::swap(locales, &mut user_locales);
                    debug_assert!(user_locales.is_empty());
                }

                Some(UserSession { session, user })
            }
        };

        // Ensure at least one locale was requested
        if locales.is_empty() {
            error!("No locales specified in user settings or Accept-Language header");
            bail!(Error::new(
                "no locales are specified in the user's settings or their Accept-Language header",
                ErrorType::NoLocalesSpecified
            ));
        }

        // Get site information
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let site_file_domain = DomainService::get_files(config, &site.slug);
        let license_name = site.license.translate(ctx.localization(), locales)?;
        let license_url = site.license.url();

        // Return
        Ok(Viewer {
            site,
            site_file_domain,
            license_name,
            license_url,
            license_kind: ViewerLicenseKind::Standard,
            license_html: None,
            user_session,
        })
    }

    async fn get_session(
        ctx: &ServiceContext<'_>,
        session_token: Option<&str>,
    ) -> Result<Option<UserSession>> {
        let make_error = || Error::new("failed to get user session", ErrorType::Session);

        // Get user data from session token (if present)
        let user_session = match session_token {
            Some("") | None => None,
            Some(token) => {
                let session =
                    SessionService::get(ctx, token).await.or_raise(make_error)?;

                let user = UserService::get(ctx, Reference::Id(session.user_id))
                    .await
                    .or_raise(make_error)?;

                Some(UserSession { session, user })
            }
        };

        Ok(user_session)
    }

    fn should_redirect_page(slug: &str) -> Option<String> {
        // Fix typos in the page slug.
        // See https://scuttle.atlassian.net/browse/WJ-330
        let mut target = slug.replace(';', ":");

        // Run slug normalization.
        // This also strips _default and merges multiple categories.
        normalize(&mut target);

        // Return
        if slug == target { None } else { Some(target) }
    }

    /// If this category is specified and exists, get its ID.
    ///
    /// * `category_slug` is `None` → `None`.
    /// * `category_slug` is `Some` but exists → `Some`.
    /// * `category_slug` is `Some` but doesn't exist → `None`.
    async fn get_category_id(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: Option<&str>,
    ) -> Result<Option<i64>> {
        match category_slug {
            Some(category_slug) => {
                let category = CategoryService::get_optional(
                    ctx,
                    site_id,
                    Reference::Slug(cow!(category_slug)),
                )
                .await
                .or_raise(|| {
                    Error::new(
                        format!("faild to get category ID for '{}'", category_slug),
                        ErrorType::PageCategory,
                    )
                })?;

                Ok(category.map(|cat| cat.category_id))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod wikidot_redirect_module_allowed_tests {
    use super::*;
    use crate::types::PageRevisionType;

    fn page(from_wikidot: bool) -> PageModel {
        PageModel {
            page_id: 1,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: None,
            deleted_at: None,
            from_wikidot,
            site_id: 1,
            latest_revision_id: Some(1),
            page_category_id: 1,
            slug: "source".to_owned(),
            discussion_thread_id: None,
            layout: None,
        }
    }

    fn revision(from_wikidot: bool) -> PageRevisionModel {
        PageRevisionModel {
            revision_id: 1,
            revision_type: PageRevisionType::Regular,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: None,
            revision_number: 1,
            page_id: 1,
            site_id: 1,
            user_id: 1,
            from_wikidot,
            changes: vec![],
            wikitext_hash: vec![],
            compiled_body_html_hash: vec![],
            compiled_body_styles_hash: None,
            compiled_top_bar_html_hash: None,
            compiled_side_bar_html_hash: None,
            compiled_at: OffsetDateTime::UNIX_EPOCH,
            compiled_generator: "test".to_owned(),
            comments: String::new(),
            hidden: vec![],
            title: "Source".to_owned(),
            alt_title: None,
            slug: "source".to_owned(),
            tags: vec![],
        }
    }

    #[test]
    fn allows_only_imported_page_and_imported_revision() {
        assert!(wikidot_redirect_module_allowed(
            &page(true),
            &revision(true)
        ));
        assert!(!wikidot_redirect_module_allowed(
            &page(false),
            &revision(true)
        ));
        assert!(!wikidot_redirect_module_allowed(
            &page(true),
            &revision(false)
        ));
        assert!(!wikidot_redirect_module_allowed(
            &page(false),
            &revision(false)
        ));
    }
}
