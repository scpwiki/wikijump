/*
 * endpoints/basic_error.rs
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

use super::prelude::*;
use crate::services::basic_error::{BasicErrorOutput, BasicErrorService};
use crate::utils::parse_locales;

/// Generates a closure that can be used to make errors for `.or_raise()`.
macro_rules! make_make_error {
    ($method:ident) => {
        || {
            Error::new(
                format!(
                    "failed to generate basic error message for {}",
                    stringify!($method),
                ),
                ErrorType::BasicError,
            )
        }
    };
}

pub async fn basic_error_missing_site_slug(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_slug: String,
    }

    let Input { locales, site_slug } = parse!(params, BasicError);
    let make_error = make_make_error!(missing_site_slug);

    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::missing_site_slug(ctx, &locales, &site_slug)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_missing_custom_domain(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        domain: String,
    }

    let Input { locales, domain } = parse!(params, BasicError);
    let make_error = make_make_error!(missing_custom_domain);

    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::missing_custom_domain(ctx, &locales, &domain)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_missing_page_slug(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_id: i64,
        page_slug: String,
    }

    let Input {
        locales,
        site_id,
        page_slug,
    } = parse!(params, BasicError);

    let make_error = make_make_error!(missing_page_slug);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::missing_page_slug(ctx, &locales, site_id, &page_slug)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_page_fetch(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_id: i64,
        page_slug: String,
    }

    let Input {
        locales,
        site_id,
        page_slug,
    } = parse!(params, BasicError);

    let make_error = make_make_error!(page_fetch);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::page_fetch(ctx, &locales, site_id, &page_slug)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_missing_file_name(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_id: i64,
        page_slug: String,
        filename: String,
    }

    let Input {
        locales,
        site_id,
        page_slug,
        filename,
    } = parse!(params, BasicError);

    let make_error = make_make_error!(missing_file_name);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::missing_file_name(ctx, &locales, site_id, &page_slug, &filename)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_file_fetch(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_id: i64,
        page_slug: String,
        filename: String,
    }

    let Input {
        locales,
        site_id,
        page_slug,
        filename,
    } = parse!(params, BasicError);

    let make_error = make_make_error!(file_fetch);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::file_fetch(ctx, &locales, site_id, &page_slug, &filename)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_text_block(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        site_id: i64,
        index: String,
        block_type: String,
        reason: String,
    }

    let Input {
        locales,
        site_id,
        index,
        block_type,
        reason,
    } = parse!(params, BasicError);

    let make_error = make_make_error!(text_block);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::text_block(ctx, &locales, site_id, &index, &block_type, &reason)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_file_root(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
    }

    let Input { locales } = parse!(params, BasicError);
    let make_error = make_make_error!(file_root);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::file_root(ctx, &locales)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_blob_fetch(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        s3_hash: String,
    }

    let Input { locales, s3_hash } = parse!(params, BasicError);

    let make_error = make_make_error!(blob_fetch);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::blob_fetch(ctx, &locales, &s3_hash)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_user_fetch(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        user_id: i64,
    }

    let Input { locales, user_id } = parse!(params, BasicError);

    let make_error = make_make_error!(user_fetch);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::user_fetch(ctx, &locales, user_id)
        .await
        .or_raise(make_error)
}

pub async fn basic_error_user_avatar(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<BasicErrorOutput> {
    #[derive(Deserialize, Debug)]
    struct Input {
        locales: Vec<String>,
        user_id: i64,
    }

    let Input { locales, user_id } = parse!(params, BasicError);

    let make_error = make_make_error!(user_avatar);
    let locales = parse_locales(&locales).or_raise(make_error)?;

    BasicErrorService::user_avatar(ctx, &locales, user_id)
        .await
        .or_raise(make_error)
}
