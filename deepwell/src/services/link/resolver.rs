/*
 * services/link/resolver.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::alias::{self, Entity as Alias};
use crate::models::page::{self, Entity as Page};
use crate::models::site::{self, Entity as Site};
use crate::services::ServiceContext;
use crate::types::{AliasType, ConnectionType};
use crate::utils::trim_default;
use ftml::data::PageRef;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

const CROSS_SITE_MISSING_PREFIX: &str = "\u{1f}wikijump-cross-site\u{1f}";

pub(super) struct ResolvedConnectionCounts {
    pub present: HashMap<(i64, ConnectionType), i32>,
    pub missing: HashMap<(i64, String, ConnectionType), i32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ReferenceLookup {
    site_slug: Option<String>,
    page_slug: String,
}

#[derive(Debug, Default)]
struct ReferencePlan {
    counts: HashMap<ReferenceLookup, HashMap<(String, ConnectionType), i32>>,
}

impl ReferencePlan {
    fn from_batches<'a>(
        batches: impl IntoIterator<Item = (&'a [PageRef], ConnectionType)>,
    ) -> Self {
        let mut plan = Self::default();

        for (page_refs, connection_type) in batches {
            for page_ref in page_refs {
                let lookup = ReferenceLookup {
                    site_slug: page_ref.site.clone(),
                    page_slug: str!(trim_default(&page_ref.page)),
                };
                let original_counts = plan.counts.entry(lookup).or_default();
                let count = original_counts
                    .entry((page_ref.page.clone(), connection_type))
                    .or_insert(0);
                *count += 1;
            }
        }

        plan
    }

    fn explicit_site_slugs(&self) -> HashSet<String> {
        self.counts
            .keys()
            .filter_map(|lookup| lookup.site_slug.clone())
            .collect()
    }

    fn page_slugs_by_site(
        &self,
        source_site_id: i64,
        explicit_sites: &HashMap<String, Option<i64>>,
    ) -> HashMap<i64, HashSet<String>> {
        let mut page_slugs_by_site = HashMap::<_, HashSet<_>>::new();

        for lookup in self.counts.keys() {
            let site_id = match &lookup.site_slug {
                None => Some(source_site_id),
                Some(site_slug) => explicit_sites.get(site_slug).copied().flatten(),
            };
            if let Some(site_id) = site_id {
                page_slugs_by_site
                    .entry(site_id)
                    .or_default()
                    .insert(lookup.page_slug.clone());
            }
        }

        page_slugs_by_site
    }

    fn finish(
        self,
        source_site_id: i64,
        explicit_sites: &HashMap<String, Option<i64>>,
        pages: &HashMap<(i64, String), i64>,
    ) -> ResolvedConnectionCounts {
        let mut present = HashMap::new();
        let mut missing = HashMap::new();

        for (lookup, original_counts) in self.counts {
            let resolved_site_id = match &lookup.site_slug {
                None => Some(source_site_id),
                Some(site_slug) => explicit_sites.get(site_slug).copied().flatten(),
            };

            let Some(resolved_site_id) = resolved_site_id else {
                let site_slug = lookup
                    .site_slug
                    .as_deref()
                    .expect("only explicit sites can fail resolution");
                for ((original_slug, connection_type), count) in original_counts {
                    let missing_slug = format!(
                        "{CROSS_SITE_MISSING_PREFIX}{site_slug}\u{1f}{original_slug}"
                    );
                    *missing
                        .entry((source_site_id, missing_slug, connection_type))
                        .or_insert(0) += count;
                }
                continue;
            };

            match pages.get(&(resolved_site_id, lookup.page_slug)) {
                Some(&page_id) => {
                    for ((_, connection_type), count) in original_counts {
                        *present.entry((page_id, connection_type)).or_insert(0) += count;
                    }
                }
                None => {
                    for ((original_slug, connection_type), count) in original_counts {
                        *missing
                            .entry((resolved_site_id, original_slug, connection_type))
                            .or_insert(0) += count;
                    }
                }
            }
        }

        ResolvedConnectionCounts { present, missing }
    }
}

pub(super) async fn resolve_connection_counts<'a>(
    ctx: &ServiceContext<'_>,
    source_site_id: i64,
    batches: impl IntoIterator<Item = (&'a [PageRef], ConnectionType)>,
) -> Result<ResolvedConnectionCounts> {
    let plan = ReferencePlan::from_batches(batches);
    let make_error = || {
        Error::new(
            format!("failed to batch-resolve connections for site ID {source_site_id}"),
            ErrorType::PageLink,
        )
    };

    let explicit_sites = resolve_explicit_sites(ctx, plan.explicit_site_slugs())
        .await
        .or_raise(make_error)?;
    let page_slugs_by_site = plan.page_slugs_by_site(source_site_id, &explicit_sites);
    let pages = resolve_pages(ctx, page_slugs_by_site)
        .await
        .or_raise(make_error)?;

    Ok(plan.finish(source_site_id, &explicit_sites, &pages))
}

async fn resolve_explicit_sites(
    ctx: &ServiceContext<'_>,
    site_slugs: HashSet<String>,
) -> Result<HashMap<String, Option<i64>>> {
    if site_slugs.is_empty() {
        return Ok(HashMap::new());
    }

    let txn = ctx.transaction();
    let make_error = || Error::new("failed to batch-resolve sites", ErrorType::PageLink);
    let site_slugs = site_slugs.into_iter().collect::<Vec<_>>();
    let aliases = Alias::find()
        .filter(alias::Column::AliasType.eq(AliasType::Site))
        .filter(alias::Column::Slug.is_in(site_slugs.clone()))
        .all(txn)
        .await
        .or_raise(make_error)?;
    let aliases_by_slug = aliases
        .into_iter()
        .map(|alias| (alias.slug, alias.target_id))
        .collect::<HashMap<_, _>>();
    let alias_target_ids = aliases_by_slug.values().copied().collect::<HashSet<_>>();
    let direct_slugs = site_slugs
        .iter()
        .filter(|slug| !aliases_by_slug.contains_key(*slug))
        .cloned()
        .collect::<HashSet<_>>();

    let mut condition = Condition::any();
    if !alias_target_ids.is_empty() {
        condition = condition.add(site::Column::SiteId.is_in(alias_target_ids));
    }
    if !direct_slugs.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(site::Column::Slug.is_in(direct_slugs.clone()))
                .add(site::Column::DeletedAt.is_null()),
        );
    }

    let sites = Site::find()
        .filter(condition)
        .all(txn)
        .await
        .or_raise(make_error)?;
    let sites_by_id = sites
        .iter()
        .map(|site| (site.site_id, site.site_id))
        .collect::<HashMap<_, _>>();
    let sites_by_slug = sites
        .iter()
        .filter(|site| site.deleted_at.is_none() && direct_slugs.contains(&site.slug))
        .map(|site| (site.slug.clone(), site.site_id))
        .collect::<HashMap<_, _>>();

    Ok(site_slugs
        .into_iter()
        .map(|site_slug| {
            let site_id = match aliases_by_slug.get(&site_slug) {
                Some(target_id) => sites_by_id.get(target_id).copied(),
                None => sites_by_slug.get(&site_slug).copied(),
            };
            (site_slug, site_id)
        })
        .collect())
}

async fn resolve_pages(
    ctx: &ServiceContext<'_>,
    page_slugs_by_site: HashMap<i64, HashSet<String>>,
) -> Result<HashMap<(i64, String), i64>> {
    if page_slugs_by_site.is_empty() {
        return Ok(HashMap::new());
    }

    let condition = page_slugs_by_site.into_iter().fold(
        Condition::any(),
        |condition, (site_id, page_slugs)| {
            condition.add(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.is_in(page_slugs)),
            )
        },
    );
    let pages = Page::find()
        .filter(condition)
        .filter(page::Column::DeletedAt.is_null())
        .all(ctx.transaction())
        .await
        .or_raise(|| Error::new("failed to batch-resolve pages", ErrorType::PageLink))?;

    Ok(pages
        .into_iter()
        .map(|page| ((page.site_id, page.slug), page.page_id))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftml::data::PageRef;

    #[test]
    fn repeated_explicit_references_share_one_lookup_and_keep_count() {
        let mut references = vec![PageRef::page_and_site("test", "target"); 175];
        references.push(PageRef {
            site: Some(str!("test")),
            page: str!("_default:target"),
            extra: None,
        });
        let plan = ReferencePlan::from_batches([(
            references.as_slice(),
            ConnectionType::IncludeMessy,
        )]);

        assert_eq!(plan.explicit_site_slugs(), HashSet::from([str!("test")]));

        let explicit_sites = HashMap::from([(str!("test"), Some(7))]);
        let pages = HashMap::from([((7, str!("target")), 11)]);
        let counts = plan.finish(3, &explicit_sites, &pages);
        assert_eq!(
            counts.present,
            HashMap::from([((11, ConnectionType::IncludeMessy), 176)])
        );
    }
}
