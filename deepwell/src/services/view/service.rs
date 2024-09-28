/*
 * services/view/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
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
use std::borrow::Cow;
use std::mem;
use unic_langid::LanguageIdentifier;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct ViewService;

impl ViewService {
    pub async fn page(
        ctx: &ServiceContext<'_>,
        GetPageView {
            domain,
            locales: locales_str,
            route,
            session_token,
        }: GetPageView,
    ) -> Result<GetPageViewOutput> {
        info!(
            "Getting page view data for domain '{}', route '{:?}', locales '{:?}'",
            domain, route, locales_str,
        );

        // Parse all locales
        let mut locales = parse_locales(&locales_str)?;
        let config = ctx.config();

        // Attempt to get a viewer helper structure, but if the site doesn't exist
        // then return right away with the "no such site" response.
        let Viewer {
            site,
            redirect_site,
            user_session,
        } = match Self::get_viewer(
            ctx,
            &mut locales,
            &domain,
            session_token.ref_map(|s| s.as_str()),
        )
        .await?
        {
            ViewerResult::FoundSite(viewer) => viewer,
            ViewerResult::MissingSite(html) => {
                return Ok(GetPageViewOutput::SiteMissing { html });
            }
        };

        // If None, means the main page for the site. Pull from site data.
        let (page_full_slug, page_extra): (&str, &str) = match &route {
            None => (&site.default_page, ""),
            Some(PageRoute { slug, extra }) => (slug, extra),
        };

        let redirect_page = Self::should_redirect_page(page_full_slug);
        let options = PageOptions::parse(page_extra);

        // Get page, revision, and text fields
        let (category_slug, page_only_slug) = split_category(page_full_slug);
        let page_info = PageInfo {
            page: cow!(page_only_slug),
            category: cow_opt!(category_slug),
            site: cow!(&site.slug),
            title: cow!(page_only_slug),
            alt_title: None,
            score: ScoreValue::Integer(0), // TODO configurable default score value
            tags: vec![],

            // TODO Determine what locale should be passed here.
            //      There are ways we can determine which locale
            //      was used for a particular message, but there
            //      are several messages in play here, each of
            //      which may technically be a slightly different
            //      locale (in case of fallbacks etc).
            //
            //      For now, just use the declared first locale
            //      passed in by the requester, since that's
            //      presumably what'd they'd *like* the message
            //      to be in, if translations are available.
            language: Cow::Owned(str!(&locales[0])),
        };

        // Helper structure to designate which variant of GetPageViewOutput to return.
        #[derive(Debug)]
        enum PageStatus {
            Found {
                page: PageModel,
                page_revision: PageRevisionModel,
            },
            Missing,
            Private,
            Banned,
        }

        // Get wikitext and HTML to return for this page.
        let (status, wikitext, compiled_html) = match PageService::get_optional(
            ctx,
            site.site_id,
            Reference::Slug(cow!(page_full_slug)),
        )
        .await?
        {
            // This page exists, return its data directly.
            Some(page) => {
                // Get associated revision
                let page_revision =
                    PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                        .await?;

                // Check user access to page
                let user_permissions = match user_session {
                    Some(ref session) => session.user_permissions,
                    None => {
                        debug!("No user for session, getting guest permission scheme");

                        // TODO get permissions from service
                        UserPermissions
                    }
                };

                // Determine whether to return the actual page contents,
                // or the "private page" data (_public).
                //
                // This returns false if the user is banned *and* the site
                // disallows banned viewing.
                if Self::can_access_page(ctx, user_permissions).await? {
                    debug!("User has page access, return text data");

                    if options.rerender
                        && Self::can_edit_page(ctx, user_permissions).await?
                    {
                        info!(
                            "Re-rendering revision: site ID {} page ID {} revision ID {} (depth {})",
                            page.site_id, page.page_id, page_revision.revision_id, 0,
                        );
                        PageRevisionService::rerender(ctx, page.site_id, page.page_id, 0)
                            .await?;
                    };

                    let (wikitext, compiled_html) = try_join!(
                        TextService::get(ctx, &page_revision.wikitext_hash),
                        TextService::get(ctx, &page_revision.compiled_hash),
                    )?;

                    (
                        PageStatus::Found {
                            page,
                            page_revision,
                        },
                        wikitext,
                        compiled_html,
                    )
                } else {
                    warn!("User doesn't have page access, returning permission page");

                    let (page_status, page_type) = if user_permissions.is_banned() {
                        (PageStatus::Banned, SpecialPageType::Banned)
                    } else {
                        (PageStatus::Private, SpecialPageType::Private)
                    };

                    let GetSpecialPageOutput {
                        wikitext,
                        render_output,
                    } = SpecialPageService::get(
                        ctx,
                        &site,
                        page_type,
                        &locales,
                        config.default_page_layout,
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

                    (page_status, wikitext, compiled_html)
                }
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
                    &locales,
                    config.default_page_layout,
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

                (PageStatus::Missing, wikitext, compiled_html)
            }
        };

        // TODO Check if user-agent and IP match?

        let viewer = Viewer {
            site,
            redirect_site,
            user_session,
        };

        let output = match status {
            PageStatus::Found {
                page,
                page_revision,
            } => GetPageViewOutput::PageFound {
                viewer,
                options,
                page,
                page_revision,
                redirect_page,
                wikitext,
                compiled_html,
            },
            PageStatus::Missing => GetPageViewOutput::PageMissing {
                viewer,
                options,
                redirect_page,
                wikitext,
                compiled_html,
            },
            PageStatus::Private => GetPageViewOutput::PagePermissions {
                viewer,
                options,
                redirect_page,
                compiled_html,
                banned: false,
            },
            PageStatus::Banned => GetPageViewOutput::PagePermissions {
                viewer,
                options,
                redirect_page,
                compiled_html,
                banned: true,
            },
        };

        Ok(output)
    }

    pub async fn user(
        ctx: &ServiceContext<'_>,
        GetUserView {
            domain,
            locales: locales_str,
            user: user_ref,
            session_token,
        }: GetUserView<'_>,
    ) -> Result<GetUserViewOutput> {
        info!(
            "Getting user view data for domain '{}', user '{:?}', locales '{:?}'",
            domain, user_ref, locales_str,
        );

        // Parse all locales
        let mut locales = parse_locales(&locales_str)?;

        // Attempt to get a viewer helper structure, but if the site doesn't exist
        // then return right away with the "no such site" response.
        let viewer = match Self::get_viewer(
            ctx,
            &mut locales,
            &domain,
            session_token.ref_map(|s| s.as_str()),
        )
        .await?
        {
            ViewerResult::FoundSite(viewer) => viewer,
            ViewerResult::MissingSite(html) => {
                return Ok(GetUserViewOutput::SiteMissing { html });
            }
        };

        // TODO Check if user-agent and IP match?

        // Get data to return for this user.
        let user = match user_ref {
            Some(user_ref) => UserService::get_optional(ctx, user_ref).await?,
            // For users visiting their own user info page
            None => viewer
                .user_session
                .as_ref()
                .map(|session| session.user.clone()),
        };

        let output = match user {
            Some(user) => GetUserViewOutput::UserFound { viewer, user },
            None => GetUserViewOutput::UserMissing { viewer },
        };

        Ok(output)
    }

    pub async fn admin(
        ctx: &ServiceContext<'_>,
        GetSiteView {
            domain,
            locales: locales_str,
            session_token,
        }: GetSiteView,
    ) -> Result<GetSiteViewOutput> {
        info!(
            "Getting site view data for domain '{}', locales '{:?}'",
            domain, locales_str,
        );

        // Parse all locales
        let mut locales = parse_locales(&locales_str)?;
        let config = ctx.config();

        // Attempt to get a viewer helper structure, but if the site doesn't exist
        // then return right away with the "no such site" response.
        let viewer = match Self::get_viewer(
            ctx,
            &mut locales,
            &domain,
            session_token.ref_map(|s| s.as_str()),
        )
        .await?
        {
            ViewerResult::FoundSite(viewer) => viewer,
            ViewerResult::MissingSite(html) => {
                return Ok(GetSiteViewOutput::SiteMissing { html });
            }
        };

        let page_info = PageInfo {
            page: cow!(""),
            category: cow_opt!(Some("admin")),
            title: cow!(""),
            alt_title: None,
            site: cow!(viewer.site.slug),
            score: ScoreValue::Integer(0),
            tags: vec![],
            language: if !locales.is_empty() {
                Cow::Owned(locales[0].to_string())
            } else {
                cow!(viewer.site.locale)
            },
        };

        let GetSpecialPageOutput {
            wikitext: _,
            render_output,
        } = SpecialPageService::get(
            ctx,
            &viewer.site,
            SpecialPageType::Unauthorized,
            &locales,
            config.default_page_layout,
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

        // Check user access to site settings
        let user_permissions = match viewer.user_session {
            Some(ref session) => session.user_permissions,
            None => {
                debug!("No user for session, disallow admin access");

                return Ok(GetSiteViewOutput::SitePermissions {
                    viewer,
                    html: compiled_html,
                });
            }
        };

        // Determine whether to return the actual admin panel content
        let output = if Self::can_access_admin(ctx, user_permissions).await? {
            debug!("User has admin access, return data");
            GetSiteViewOutput::SiteFound { viewer }
        } else {
            warn!("User doesn't have admin access, returning permission page");

            GetSiteViewOutput::SitePermissions {
                viewer,
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
    /// * Hostname of request → Site ID and data
    /// * Session token → User ID and their permissions
    ///
    /// Then using this information, the caller can perform some common
    /// operations, such as slug normalization or redirect site aliases.
    pub async fn get_viewer(
        ctx: &ServiceContext<'_>,
        locales: &mut Vec<LanguageIdentifier>,
        domain: &str,
        session_token: Option<&str>,
    ) -> Result<ViewerResult> {
        info!("Getting viewer data from domain '{domain}' and session token");

        // Get user data from session token (if present)
        let user_session = match session_token {
            None => None,
            Some("") => None,
            Some(token) => {
                let session = SessionService::get(ctx, token).await?;
                let user = UserService::get(ctx, Reference::Id(session.user_id)).await?;

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

                    let mut user_locales = parse_locales(&user.locales)?;
                    user_locales.append(locales);
                    mem::swap(locales, &mut user_locales);
                    debug_assert!(user_locales.is_empty());
                }

                Some(UserSession {
                    session,
                    user,
                    user_permissions: UserPermissions, // TODO add user permissions, get scheme for user and site
                })
            }
        };

        // Ensure at least one locale was requested
        if locales.is_empty() {
            error!("No locales specified in user settings or Accept-Language header");
            return Err(Error::NoLocalesSpecified);
        }

        // Get site data
        let (site, redirect_site) =
            match DomainService::parse_site_from_domain(ctx, domain).await? {
                SiteDomainResult::Found(site) => {
                    let redirect_site = Self::should_redirect_site(ctx, &site, domain);
                    (site, redirect_site)
                }
                SiteDomainResult::Slug(slug) => {
                    let html =
                        Self::missing_site_output(ctx, locales, domain, Some(slug))
                            .await?;

                    return Ok(ViewerResult::MissingSite(html));
                }
                SiteDomainResult::CustomDomain(domain) => {
                    let html =
                        Self::missing_site_output(ctx, locales, domain, None).await?;

                    return Ok(ViewerResult::MissingSite(html));
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
        locales: &[LanguageIdentifier],
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

                let html = ctx.localization().translate(
                    locales,
                    "wiki-page-site-slug",
                    &args,
                )?;

                Ok(html.to_string())
            }

            // Custom domain missing error
            None => {
                let mut args = FluentArgs::new();
                args.set("custom_domain", fluent_str!(domain));
                args.set("domain", fluent_str!(config.main_domain_no_dot));

                let html = ctx.localization().translate(
                    locales,
                    "wiki-page-site-custom",
                    &args,
                )?;

                Ok(html.to_string())
            }
        }
    }

    async fn can_access_page(
        _ctx: &ServiceContext<'_>,
        permissions: UserPermissions,
    ) -> Result<bool> {
        info!("Checking page access: {permissions:?}");
        debug!("TODO: stub");
        // TODO perform permission checks
        Ok(true)
    }

    async fn can_edit_page(
        _ctx: &ServiceContext<'_>,
        permissions: UserPermissions,
    ) -> Result<bool> {
        info!("Checking page access: {permissions:?}");
        debug!("TODO: stub");
        // TODO perform permission checks
        Ok(true)
    }

    async fn can_access_admin(
        _ctx: &ServiceContext<'_>,
        permissions: UserPermissions,
    ) -> Result<bool> {
        info!("Checking admin access: {permissions:?}");
        debug!("TODO: stub");
        // TODO perform permission checks
        Ok(true)
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

/// Converts an array of strings to a list of locales.
///
/// Empty locales lists _are_ allowed, since we have not
/// yet checked the user's locale preferences.
fn parse_locales<S: AsRef<str>>(locales_str: &[S]) -> Result<Vec<LanguageIdentifier>> {
    let mut locales = Vec::with_capacity(locales_str.len());
    for locale_str in locales_str {
        let locale = LanguageIdentifier::from_bytes(locale_str.as_ref().as_bytes())?;
        locales.push(locale);
    }
    Ok(locales)
}
