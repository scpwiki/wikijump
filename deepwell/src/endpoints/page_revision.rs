/*
 * endpoints/page_revision.rs
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
use crate::{
    models::page_revision::Model as PageRevisionModel,
    services::{
        TextService,
        page::GetPageReference,
        page_revision::{
            GetPageRevision, GetPageRevisionDetails, GetPageRevisionRangeDetails,
            PageRevisionCountOutput, PageRevisionModelFiltered,
            UpdatePageRevisionDetails,
        },
    },
    types::PageDetails,
};

pub async fn page_revision_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageRevisionCountOutput> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, PageRevision);

    info!("Getting latest revision for page {reference:?} in site ID {site_id}");

    let make_error =
        || Error::new("failed to get page revision count", ErrorType::PageRevision);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    let revision_count = PageRevisionService::count(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    Ok(PageRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    })
}

pub async fn page_revision_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageRevisionModelFiltered>> {
    let GetPageRevisionDetails {
        input:
            GetPageRevision {
                site_id,
                page_id,
                revision_number,
            },
        details,
    } = parse!(params, PageRevision);

    info!(
        "Getting revision {} for page ID {} in site ID {}",
        revision_number, page_id, site_id,
    );

    let make_error =
        || Error::new("failed to get a page revision", ErrorType::PageRevision);

    let revision =
        PageRevisionService::get_optional(ctx, site_id, page_id, revision_number)
            .await
            .or_raise(make_error)?;

    match revision {
        None => Ok(None),
        Some(revision) => {
            let revision = filter_and_populate_revision(ctx, revision, details)
                .await
                .or_raise(make_error)?;

            Ok(Some(revision))
        }
    }
}

pub async fn page_revision_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageRevisionModelFiltered> {
    let UpdatePageRevisionDetails { input, details } = parse!(params, PageRevision);

    info!(
        "Editing revision ID {} for page ID {} in site ID {}",
        input.revision_id, input.page_id, input.site_id,
    );

    let make_error =
        || Error::new("failed to edit a page revision", ErrorType::PageRevision);

    let revision_id = input.revision_id;
    let (_, revision) = join!(
        PageRevisionService::update(ctx, input),
        PageRevisionService::get_direct(ctx, revision_id),
    );
    let revision = raise_multiple!(revision; make_error);

    filter_and_populate_revision(ctx, revision, details)
        .await
        .or_raise(make_error)
}

pub async fn page_revision_range(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageRevisionModelFiltered>> {
    let GetPageRevisionRangeDetails { input, details } = parse!(params, PageRevision);

    let make_error = || {
        Error::new(
            "failed to get a range of page revisions",
            ErrorType::PageRevision,
        )
    };

    let revisions = PageRevisionService::get_range(ctx, input)
        .await
        .or_raise(make_error)?;

    filter_and_populate_revisions(ctx, revisions, details)
        .await
        .or_raise(make_error)
}

// Helper functions

async fn filter_and_populate_revision(
    ctx: &ServiceContext<'_>,
    model: PageRevisionModel,
    mut details: PageDetails,
) -> Result<PageRevisionModelFiltered> {
    let PageRevisionModel {
        revision_id,
        revision_type,
        created_at,
        updated_at,
        from_wikidot,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext_hash,
        compiled_body_html_hash,
        compiled_top_bar_html_hash,
        compiled_side_bar_html_hash,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        mut alt_title,
        slug,
        tags,
    } = model;

    let make_error = || {
        Error::new(
            "failed to filter and populate revision data",
            ErrorType::PageRevision,
        )
    };

    // Strip hidden fields
    let mut comments = Some(comments);
    let mut title = Some(title);
    // alt-title is already Option and we're not doubling up
    let mut slug = Some(slug);
    let mut tags = Some(tags);

    for field in &hidden {
        // TODO hidden fields aren't standardized yet
        match field.as_str() {
            "wikitext" => details.wikitext = false,
            "compiled" => details.compiled_html = false,
            "comments" => comments = None,
            "title" => title = None,
            "alt_title" => alt_title = None,
            "slug" => slug = None,
            "tags" => tags = None,
            _ => panic!("Unknown field name in hidden: {field}"),
        }
    }

    // Get text data, if requested
    let (wikitext, compiled_body_html, compiled_top_bar_html, compiled_side_bar_html) = join!(
        TextService::get_conditional(ctx, details.wikitext, &wikitext_hash),
        TextService::get_conditional(
            ctx,
            details.compiled_html,
            &compiled_body_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &compiled_top_bar_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &compiled_side_bar_html_hash,
        ),
    );

    let wikitext = wikitext.or_raise(make_error)?;
    let compiled_body_html = compiled_body_html.or_raise(make_error)?;
    let compiled_top_bar_html = compiled_top_bar_html.or_raise(make_error)?;
    let compiled_side_bar_html = compiled_side_bar_html.or_raise(make_error)?;

    Ok(PageRevisionModelFiltered {
        revision_id,
        revision_type,
        created_at,
        updated_at,
        from_wikidot,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext,
        compiled_body_html,
        compiled_top_bar_html,
        compiled_side_bar_html,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        alt_title,
        slug,
        tags,
    })
}

async fn filter_and_populate_revisions(
    ctx: &ServiceContext<'_>,
    revisions: Vec<PageRevisionModel>,
    details: PageDetails,
) -> Result<Vec<PageRevisionModelFiltered>> {
    let mut f_revisions = Vec::new();

    let make_error = || {
        Error::new(
            "failed to populate a list of revisions",
            ErrorType::PageRevision,
        )
    };

    for revision in revisions {
        let f_revision = filter_and_populate_revision(ctx, revision, details)
            .await
            .or_raise(make_error)?;

        f_revisions.push(f_revision)
    }

    Ok(f_revisions)
}
