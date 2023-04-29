/*
 * services/view/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use super::prelude::*;
use crate::models::site::Model as SiteModel;
use crate::services::domain::SiteDomainResult;
use crate::services::render::RenderOutput;
use crate::services::special_page::{GetSpecialPageOutput, SpecialPageType};
use crate::services::{
    DomainService, PageRevisionService, PageService, SessionService, SpecialPageService,
    TextService, UserService,
};
use crate::utils::split_category;
use fluent::{FluentArgs, FluentValue};
use ftml::prelude::*;
use ftml::render::html::HtmlOutput;
use ref_map::*;
use unic_langid::LanguageIdentifier;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct ViewService;

impl ViewService {
    pub async fn page(
        ctx: &ServiceContext<'_>,
        GetPageView {
            domain,
            locale: locale_str,
            route,
            session_token,
        }: GetPageView,
    ) -> Result<GetPageViewOutput> {
        tide::log::info!(
            "Getting page view data for domain '{}', route '{:?}', locale '{}'",
            domain,
            route,
            locale_str,
        );

        let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;

        // Attempt to get a viewer helper structure, but if the site doesn't exist
        // then return right away with the "no such site" response.
        let Viewer {
            site,
            redirect_site,
            user_session,
        } = match Self::get_viewer(
            ctx,
            &locale,
            &domain,
            session_token.ref_map(|s| s.as_str()),
        )
        .await?
        {
            ViewerResult::FoundSite(viewer) => viewer,
            ViewerResult::MissingSite(output) => return todo!(),
        };

        // If None, means the main page for the site. Pull from site data.
        let (page_slug, page_extra): (&str, &str) = match &route {
            None => (&site.default_page, ""),
            Some(PageRoute { slug, extra }) => (slug, extra),
        };

        let redirect_page = Self::should_redirect_page(page_slug);
        let options = PageOptions::parse(page_extra);

        // Get page, revision, and text fields
        let (category_slug, page_slug) = split_category(page_slug);
        let page_info = PageInfo {
            page: cow!(page_slug),
            category: cow_opt!(category_slug),
            site: cow!(&site.slug),
            title: cow!(page_slug),
            alt_title: None,
            score: ScoreValue::Integer(0), // TODO configurable default score value
            tags: vec![],
            language: cow!(locale_str),
        };

        // Get wikitext and HTML to return for this page.
        let (page_and_revision, wikitext, compiled_html) =
            match PageService::get_optional(
                ctx,
                site.site_id,
                Reference::Slug(cow!(page_slug)),
            )
            .await?
            {
                // This page exists, return its data directly.
                Some(page) => {
                    // TODO determine if page needs rerender?

                    let page_revision =
                        PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                            .await?;

                    let (wikitext, compiled_html) = try_join!(
                        TextService::get(ctx, &page_revision.wikitext_hash),
                        TextService::get(ctx, &page_revision.compiled_hash),
                    )?;

                    (Some((page, page_revision)), wikitext, compiled_html)
                }
                // The page is missing, fetch the "missing page" data (_404).
                None => {
                    let GetSpecialPageOutput {
                        wikitext,
                        render_output,
                    } = SpecialPageService::get(
                        ctx,
                        &site,
                        SpecialPageType::Missing,
                        &locale,
                        page_info,
                    )
                    .await?;

                    let RenderOutput {
                        html_output:
                            HtmlOutput {
                                body: compiled_html,
                                ..
                            },
                        ..
                    } = render_output;

                    (None, wikitext, compiled_html)
                }
            };

        // TODO Check if user-agent and IP match?

        let viewer = Viewer {
            site,
            redirect_site,
            user_session,
        };

        let output = match page_and_revision {
            Some((page, page_revision)) => GetPageViewOutput::PageFound {
                viewer,
                options,
                page,
                page_revision,
                redirect_page,
                wikitext,
                compiled_html,
            },
            None => GetPageViewOutput::PageMissing {
                viewer,
                options,
                redirect_page,
                wikitext,
                compiled_html,
            },
        };

        Ok(output)
    }

    /// Gets basic data and runs common logic for all web routes.
    ///
    /// All views seen by end users require a few translations before
    /// a request can be serviced:
    ///
    /// * Hostname of request → Site ID and data
    /// * Session token → User ID and their permissions
    ///
    /// Then using this information, the caller can perform some common
    /// operations, such as slug normalization or redirect site aliases.
    pub async fn get_viewer(
        ctx: &ServiceContext<'_>,
        locale: &LanguageIdentifier,
        domain: &str,
        session_token: Option<&str>,
    ) -> Result<ViewerResult> {
        tide::log::info!("Getting viewer data from domain '{domain}' and session token");

        // Get site data
        let (site, redirect_site) =
            match DomainService::site_from_domain_optional(ctx, domain).await? {
                SiteDomainResult::Found(site) => {
                    let redirect_site = Self::should_redirect_site(ctx, &site, domain);
                    (site, redirect_site)
                }
                SiteDomainResult::Slug(slug) => {
                    let html = Self::missing_site_output(ctx, locale, domain, Some(slug))
                        .await?;

                    return todo!();
                }
                SiteDomainResult::CustomDomain(domain) => {
                    let html =
                        Self::missing_site_output(ctx, locale, domain, None).await?;

                    return todo!();
                }
            };

        // Get user data from session token (if present)
        let user_session = match session_token {
            None => None,
            Some(token) if token.is_empty() => None,
            Some(token) => {
                let session = SessionService::get(ctx, token).await?;
                let user = UserService::get(ctx, Reference::Id(session.user_id)).await?;

                Some(UserSession {
                    session,
                    user,
                    user_permissions: (), // TODO add user permissions, get scheme for user and site
                })
            }
        };

        Ok(ViewerResult::FoundSite(Viewer {
            site,
            redirect_site,
            user_session,
        }))
    }

    /// Produce output for cases where a site does not exist.
    async fn missing_site_output(
        ctx: &ServiceContext<'_>,
        locale: &LanguageIdentifier,
        domain: &str,
        site_slug: Option<&str>,
    ) -> Result<String> {
        let config = ctx.config();
        match site_slug {
            // No site with slug error
            Some(site_slug) => {
                let mut args = FluentArgs::new();
                args.set("slug", fluent_str!(site_slug));
                args.set("domain", fluent_str!(config.main_domain_no_dot));

                let html =
                    ctx.localization()
                        .translate(locale, "wiki-page-site-slug", &args)?;

                Ok(html.to_string())
            }

            // Custom domain missing error
            None => {
                let mut args = FluentArgs::new();
                args.set("custom_domain", fluent_str!(domain));
                args.set("domain", fluent_str!(config.main_domain_no_dot));

                let html = ctx.localization().translate(
                    locale,
                    "wiki-page-site-custom",
                    &args,
                )?;

                Ok(html.to_string())
            }
        }
    }

    fn should_redirect_site(
        ctx: &ServiceContext,
        site: &SiteModel,
        domain: &str,
    ) -> Option<String> {
        // NOTE: We have to pass an owned string here, since the Cow borrows from
        //       SiteModel, which we are also passing in the final output struct.
        let preferred_domain = DomainService::domain_for_site(ctx.config(), site);
        if domain == preferred_domain {
            None
        } else {
            Some(preferred_domain.into_owned())
        }
    }

    fn should_redirect_page(slug: &str) -> Option<String> {
        // Fix typos in the page slug.
        // See https://scuttle.atlassian.net/browse/WJ-330
        let mut target = slug.replace(';', ":");

        // Run slug normalization.
        // This also strips _default and merges multiple categories.
        normalize(&mut target);

        // Return
        if slug == target {
            None
        } else {
            Some(target)
        }
    }
}
