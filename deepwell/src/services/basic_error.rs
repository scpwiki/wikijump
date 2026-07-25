/*
 * services/basic_error.rs
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

//! The "basic error" service.
//!
//! This produces localized HTML output which corresponds
//! to a particular "basic" error condition.
//!
//! A basic error is a low-level error state which occurs
//! when something fundamental about a web request is wrong.
//!
//! Normally, error pages source their text from templates
//! like `_404`, but if the site referenced does not even exist,
//! or the request is to something unexpected like the root of
//! `wjfiles.com`, or the site has no template for a missing pages,
//! then a more "basic" error needs to be returned.

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::{DomainService, SiteService};
use crate::types::Reference;
use fluent::{FluentArgs, FluentValue};
use unic_langid::LanguageIdentifier;

#[derive(Serialize, Debug, Clone)]
pub struct BasicErrorOutput {
    pub title: String,
    pub body: String,
}

#[derive(Debug)]
pub struct BasicErrorService;

impl BasicErrorService {
    /// Error for when a canonical site does not exist.
    pub async fn missing_site_slug(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_slug: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate missing_site_slug basic error for site '{}'",
                    site_slug,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        args.set("main_domain", fluent_str!(config.main_domain_no_dot));
        args.set("files_domain", fluent_str!(config.files_domain_no_dot));
        args.set("slug", fluent_str!(site_slug));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-site-slug.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-site-slug", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Error for when a custom domain does not exist.
    pub async fn missing_custom_domain(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        domain: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate missing_custom_domain basic error for domain '{}'",
                    domain,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        args.set("main_domain", fluent_str!(config.main_domain_no_dot));
        args.set("files_domain", fluent_str!(config.files_domain_no_dot));
        args.set("custom_domain", fluent_str!(domain));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-site-custom.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-site-custom", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Error for when a page slug does not exist.
    ///
    /// This is _not_ used as the regular "page missing" error
    /// (see the `_404` special page), but instead in contexts
    /// like in wjfiles where we need to display an error for
    /// when a file doesn't exist because the page it's supposedly
    /// attached to doesn't exist.
    pub async fn missing_page_slug(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_id: i64,
        page_slug: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate missing_page_slug basic error for site ID {} page '{}'",
                    site_id, page_slug,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let domain = DomainService::preferred_domain(config, &site);
        args.set("domain", fluent_str!(domain));
        args.set("page_slug", fluent_str!(page_slug));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-page-slug.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-page-slug", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Error for when a page cannot be fetched.
    ///
    /// Note the same caveats as above.
    pub async fn page_fetch(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_id: i64,
        page_slug: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate page_fetch basic error for site ID {} page '{}'",
                    site_id, page_slug,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        let domain = DomainService::preferred_domain(config, &site);
        args.set("domain", fluent_str!(domain));
        args.set("page_slug", fluent_str!(page_slug));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-page-fetch.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-page-fetch", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Error for when a file (referred to by filename) does not exist.
    pub async fn missing_file_name(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_id: i64,
        page_slug: &str,
        filename: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate missing_file_name basic error for site ID {} page '{}' file '{}'",
                    site_id, page_slug, filename,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        let domain = DomainService::preferred_domain(config, &site);
        args.set("domain", fluent_str!(domain));
        args.set("page_slug", fluent_str!(page_slug));
        args.set("filename", fluent_str!(filename));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-file-name.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-file-name", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    pub async fn file_fetch(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_id: i64,
        page_slug: &str,
        filename: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate file_fetch basic error for site ID {} page '{}' filename '{}'",
                    site_id, page_slug, filename,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let domain = DomainService::preferred_domain(config, &site);
        args.set("domain", fluent_str!(domain));
        args.set("page_slug", fluent_str!(page_slug));
        args.set("filename", fluent_str!(filename));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-file-fetch.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-file-fetch", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Multi-faceted error for issues with hosted text blocks.
    pub async fn text_block(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
        site_id: i64,
        index: &str,
        block_type: &str,
        reason: &str,
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate text_block basic error for site ID {} {} block at index {}, with reason {}",
                    site_id, block_type, index, reason,
                ),
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        let domain = DomainService::preferred_domain(config, &site);
        args.set("domain", fluent_str!(domain));
        args.set("index", fluent_str!(index));
        args.set("block_type", fluent_str!(block_type));
        args.set("reason", fluent_str!(reason));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-text-block.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-text-block", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }

    /// Error for when a user tries to access wjfiles without passing in a site slug.
    pub async fn file_root(
        ctx: &ServiceContext<'_>,
        locales: &[LanguageIdentifier],
    ) -> Result<BasicErrorOutput> {
        let make_error = || {
            Error::new(
                "failed to generate file_root basic error",
                ErrorType::BasicError,
            )
        };

        assert!(!locales.is_empty(), "No languages specified");
        let config = ctx.config();
        let mut args = FluentArgs::new();
        args.set("main_domain", fluent_str!(config.main_domain_no_dot));
        args.set("files_domain", fluent_str!(config.files_domain_no_dot));

        let title = ctx
            .localization()
            .translate(locales, "basic-error-file-root.title", &args)
            .or_raise(make_error)?;

        let body = ctx
            .localization()
            .translate(locales, "basic-error-file-root", &args)
            .or_raise(make_error)?;

        Ok(BasicErrorOutput {
            title: title.to_string(),
            body: body.to_string(),
        })
    }
}
