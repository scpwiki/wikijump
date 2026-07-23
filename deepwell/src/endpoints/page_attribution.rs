/*
 * endpoints/page_attribution.rs
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
use crate::services::MutationAuthorization;
use crate::services::relation::{
    ClearPageAttributions, GetPageAttributions, PageAttribution, RelationService,
    SetPageAttributions,
};
use crate::types::{Action, Permission, Reference, Resource};

async fn require_page_attribution_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page: Reference<'_>,
    submitted_user_id: i64,
    action: &str,
) -> Result<()> {
    MutationAuthorization::require_matching_actor(ctx, submitted_user_id, action)?;
    MutationAuthorization::require_permission(
        ctx,
        site_id,
        Some(page),
        Permission {
            resource_type: Resource::Page,
            resource_category: None,
            action: Action::Edit,
        },
        action,
    )
    .await?;
    Ok(())
}

pub async fn page_attribution_get_page(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageAttribution>> {
    let input: GetPageAttributions<'_> = parse!(params, PageAttribution);

    RelationService::get_page_attributions(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to get page attributions",
                ErrorType::PageAttribution,
            )
        })
}

pub async fn page_attribution_update(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageAttribution>> {
    let input: SetPageAttributions<'_> = parse!(params, PageAttribution);
    require_page_attribution_permission(
        ctx,
        input.site_id,
        input.page.clone(),
        input.updated_by,
        "update page attributions",
    )
    .await?;

    info!(
        "Setting {} page attributions for page {:?} on site {} (requested by {})",
        input.attributions.len(),
        input.page,
        input.site_id,
        input.updated_by,
    );

    RelationService::set_page_attributions(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to update page attributions",
                ErrorType::PageAttribution,
            )
        })
}

pub async fn page_attribution_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: ClearPageAttributions<'_> = parse!(params, PageAttribution);
    require_page_attribution_permission(
        ctx,
        input.site_id,
        input.page.clone(),
        input.removed_by,
        "delete page attributions",
    )
    .await?;

    info!(
        "Clearing page attributions for page {:?} on site {} (requested by {})",
        input.page, input.site_id, input.removed_by,
    );

    RelationService::clear_page_attributions(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to delete page attributions",
                ErrorType::PageAttribution,
            )
        })
}
