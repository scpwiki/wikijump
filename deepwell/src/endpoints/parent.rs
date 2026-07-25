/*
 * endpoints/parent.rs
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
use crate::models::page_parent::Model as PageParentModel;
use crate::services::page::GetPageReference;
use crate::services::parent::{
    GetParentRelationships, ParentDescription, RemoveParentOutput, UpdateParents,
    UpdateParentsOutput,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{Action, Permission, Reference, Resource};
use futures::future::try_join_all;

pub async fn parent_relationships_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageParentModel>> {
    let GetParentRelationships {
        site_id,
        page: reference,
        relationship_type,
    } = parse!(params, PageParent);

    ParentService::get_relationships(ctx, site_id, reference, relationship_type)
        .await
        .or_raise(|| {
            Error::new(
                "failed to get page parent relationships",
                ErrorType::PageParent,
            )
        })
}

pub async fn parent_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageParentModel>> {
    let input: ParentDescription = parse!(params, PageParent);

    ParentService::get_optional(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get info on one page parent relationship",
            ErrorType::PageParent,
        )
    })
}

pub async fn parent_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageParentModel>> {
    let input: ParentDescription = parse!(params, PageParent);

    info!(
        "Creating parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

    ensure_child_page_edit_permission(
        ctx,
        input.site_id,
        input.child.clone(),
        ctx.request().user_id,
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check page parent create permission",
            ErrorType::PageParent,
        )
    })?;

    ParentService::create(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to create page parent relationship",
            ErrorType::PageParent,
        )
    })
}

pub async fn parent_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RemoveParentOutput> {
    let input: ParentDescription = parse!(params, PageParent);

    info!(
        "Removing parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

    ensure_child_page_edit_permission(
        ctx,
        input.site_id,
        input.child.clone(),
        ctx.request().user_id,
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check page parent remove permission",
            ErrorType::PageParent,
        )
    })?;

    ParentService::remove(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to remove page parent relationship",
            ErrorType::PageParent,
        )
    })
}

pub async fn parent_get_all(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<String>> {
    let GetPageReference { site_id, page } = parse!(params, PageParent);

    let make_error = || {
        Error::new(
            "failed to get all page parents for a child page",
            ErrorType::PageParent,
        )
    };

    let parents: Vec<Reference<'_>> = ParentService::get_parents(ctx, site_id, page)
        .await
        .or_raise(make_error)?
        .iter()
        .map(|p| Reference::from(p.parent_page_id))
        .collect();

    let pages: Vec<String> = PageService::get_pages(ctx, site_id, parents.as_slice())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|p| p.slug)
        .collect();

    Ok(pages)
}

pub async fn parent_update(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UpdateParentsOutput> {
    let input: UpdateParents = parse!(params, PageParent);
    let permission_user_id = ctx.request().user_id;

    info!(
        "Updating multiple parental relationships for child {:?} in site ID {}",
        input.child, input.site_id,
    );

    let make_error = || {
        Error::new(
            "failed to update multiple page parent relationships",
            ErrorType::PageParent,
        )
    };

    ensure_child_page_edit_permission(
        ctx,
        input.site_id,
        input.child.clone(),
        permission_user_id,
    )
    .await
    .or_raise(make_error)?;

    let creation = match input.add {
        Some(parents) => {
            let creation = parents.iter().map(|parent| {
                ParentService::create(
                    ctx,
                    ParentDescription {
                        site_id: input.site_id,
                        parent: parent.to_owned(),
                        child: input.child.clone(),
                    },
                )
            });
            Some(
                try_join_all(creation)
                    .await
                    .or_raise(make_error)?
                    .iter()
                    .flatten()
                    .map(|p| p.parent_page_id)
                    .collect(),
            )
        }
        None => None,
    };

    let removal = match input.remove {
        Some(parents) => {
            let removal = parents.iter().map(|parent| {
                ParentService::remove(
                    ctx,
                    ParentDescription {
                        site_id: input.site_id,
                        parent: parent.to_owned(),
                        child: input.child.clone(),
                    },
                )
            });
            Some(
                try_join_all(removal)
                    .await
                    .or_raise(make_error)?
                    .iter()
                    .map(|p| p.was_deleted)
                    .collect(),
            )
        }
        None => None,
    };

    Ok(UpdateParentsOutput {
        added: creation,
        removed: removal,
    })
}

async fn ensure_child_page_edit_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    child: Reference<'a>,
    user_id: Option<i64>,
) -> Result<()> {
    let make_error = || {
        Error::new(
            "failed to check child page edit permission",
            ErrorType::Permission,
        )
    };

    let page = PageService::get(ctx, site_id, child)
        .await
        .or_raise(make_error)?;

    let can_edit = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id,
            site_id,
            page_reference: Some(Reference::Id(page.page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(page.page_category_id)),
            action: Action::Edit,
        },
    )
    .await
    .or_raise(make_error)?;

    if can_edit {
        Ok(())
    } else {
        Err(Error::new(
            "user does not have permission to edit this child page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}
