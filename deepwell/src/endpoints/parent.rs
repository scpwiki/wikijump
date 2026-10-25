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

use std::collections::{HashMap, HashSet};

use super::prelude::*;
use crate::models::page_parent::Model as PageParentModel;
use crate::services::page::GetPageReference;
use crate::services::parent::{
    GetChildOutput, GetParentRelationships, ParentDescription, RemoveParentOutput,
    TraceParents, TraceParentsOutput, UpdateParents, UpdateParentsOutput,
};
use crate::types::Reference;
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

    info!(
        "Getting all {} pages from {:?} in site ID {}",
        relationship_type.name(),
        reference,
        site_id,
    );

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

    info!(
        "Getting parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

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

    info!("Getting parents for child {page:?} in site ID {site_id}");

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

pub async fn child_get_all(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetChildOutput>> {
    let make_error = || {
        Error::new(
            "failed to get all children pages for a parent page",
            ErrorType::PageParent,
        )
    };

    let GetPageReference { site_id, page } = params.parse().or_raise(make_error)?;

    info!(
        "Getting children for parent {:?} in site ID {}",
        page, site_id,
    );

    let parents: Vec<Reference<'_>> = ParentService::get_children(ctx, site_id, page)
        .await
        .or_raise(make_error)?
        .iter()
        .map(|p| Reference::from(p.child_page_id))
        .collect();

    let page_revision_ids: Vec<i64> =
        PageService::get_pages(ctx, site_id, parents.as_slice())
            .await
            .or_raise(make_error)?
            .into_iter()
            .filter_map(|p| p.latest_revision_id)
            .collect();

    let page_details = PageRevisionService::get_revisions(ctx, page_revision_ids)
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|p| GetChildOutput {
            slug: p.slug,
            title: p.title,
            page_id: p.page_id,
        })
        .collect();

    Ok(page_details)
}

/// Traces the parent chains for one page.
///
/// This problem is similar to constructing a commit graph,
/// with the assumption of no cyclic parenthood, i.e.
/// a directed acyclic graph.
///
/// Cyclic parenthood should have been checked when adding
/// new parent-child relations.
///
/// Here we trace all the relevant pages in the chains
/// and each page's children pages, along with the root
/// parent page. The actual parent graph is constructed in
/// the frontend.
pub async fn parent_trace(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<TraceParentsOutput> {
    let make_error = || {
        Error::new(
            "failed to get trace parent tree for a page",
            ErrorType::PageParent,
        )
    };

    let TraceParents {
        site_id,
        page,
        depth,
    } = params.parse().or_raise(make_error)?;

    let page_id = PageService::get_id(ctx, site_id, page)
        .await
        .or_raise(make_error)?;
    let page_details = PageRevisionService::get_latest(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    let mut children_map: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut parent_map: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut detail_map: HashMap<i64, GetChildOutput> = HashMap::new();
    let mut current_node_list: Vec<i64> = vec![page_id];
    let mut checked_pages: HashSet<i64> = HashSet::new();
    let mut root_parents: HashSet<i64> = HashSet::new();

    detail_map.insert(
        page_id,
        GetChildOutput {
            slug: page_details.slug,
            title: page_details.title,
            page_id,
        },
    );

    use std::collections::hash_map::Entry;

    let mut i = 1;

    while if let Some(depth) = depth {
        i < depth
    } else {
        true
    } {
        let parents = ParentService::get_parents_multi(ctx, current_node_list.clone())
            .await
            .or_raise(make_error)?;

        checked_pages.extend(current_node_list.iter());

        if parents.is_empty() {
            break;
        };

        let mut current_child_list: HashSet<i64> = HashSet::new();

        for p in &parents {
            if let Entry::Vacant(e) = children_map.entry(p.parent_page_id) {
                e.insert(vec![p.child_page_id]);
            } else {
                children_map
                    .get_mut(&p.parent_page_id)
                    .unwrap()
                    .push(p.child_page_id);
            }
            if let Entry::Vacant(e) = parent_map.entry(p.child_page_id) {
                e.insert(vec![p.parent_page_id]);
            } else {
                parent_map
                    .get_mut(&p.child_page_id)
                    .unwrap()
                    .push(p.parent_page_id);
            }
            current_child_list.insert(p.child_page_id);
        }

        // Find nodes that have no parents
        let current_root_parents: Vec<i64> = current_node_list
            .into_iter()
            .collect::<HashSet<i64>>()
            .difference(&current_child_list)
            .map(|id| id.to_owned())
            .collect();

        root_parents.extend(current_root_parents);

        // Move one node up the chains

        current_node_list = parents
            .clone()
            .into_iter()
            .filter_map(|p| {
                // Remove nodes that are already evaluated
                if checked_pages.contains(&p.parent_page_id) {
                    None
                } else {
                    Some(p.parent_page_id)
                }
            })
            .collect();

        // Get newly fetched page details
        let parent_pages_id: Vec<Reference<'_>> = current_node_list
            .iter()
            .map(|i| Reference::Id(*i))
            .collect();

        let page_revision_ids: Vec<i64> =
            PageService::get_pages(ctx, site_id, parent_pages_id.as_slice())
                .await
                .or_raise(make_error)?
                .into_iter()
                .filter_map(|p| p.latest_revision_id)
                .collect();

        let details = PageRevisionService::get_revisions(ctx, page_revision_ids)
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| {
                (
                    p.page_id,
                    GetChildOutput {
                        slug: p.slug,
                        title: p.title,
                        page_id: p.page_id,
                    },
                )
            });

        detail_map.extend(details);

        i += 1;
    }

    root_parents.extend(current_node_list);

    Ok(TraceParentsOutput {
        children_map: children_map
            .drain()
            .map(|(id, mut ch)| {
                ch.insert(0, id);
                ch
            })
            .collect(),
        parent_map: parent_map
            .drain()
            .map(|(id, mut ch)| {
                ch.insert(0, id);
                ch
            })
            .collect(),
        details: detail_map.drain().map(|(_id, ch)| ch).collect(),
        root_parents: root_parents.into_iter().collect(),
    })
}
