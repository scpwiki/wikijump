/*
 * services/render/runtime.rs
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

use super::include_attachment_owners::AttachmentProvenanceRegistry;
use super::runtime_page_queries::{
    ViewableCountPagesRows, ViewableListPagesRows, find_viewable_count_pages_rows,
    find_viewable_list_pages_rows,
};
use super::service::site_matches_wikidot_slug;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::site::Model as SiteModel;
use crate::services::page_query::{PageQuery, PageQueryScoreFilterCache};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{PageRevisionService, PageService, ServiceContext, SiteService};
use crate::types::{Action, Permission, Reference, Resource};
use crate::utils::trim_default;
use ftml::data::PageRef;
use ftml::{self};
use std::collections::{BTreeMap, HashMap};
use std::future::Future;

#[derive(Debug)]
pub(super) struct RenderRuntime<'context, 'transaction> {
    ctx: &'context ServiceContext<'transaction>,
}

impl<'context, 'transaction> RenderRuntime<'context, 'transaction> {
    pub(super) fn new(ctx: &'context ServiceContext<'transaction>) -> Self {
        Self { ctx }
    }

    pub(super) async fn find_viewable_list_pages_rows(
        &self,
        query: PageQuery<'_>,
        target_count: usize,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
        score_filter_cache: Option<&mut PageQueryScoreFilterCache>,
    ) -> Result<ViewableListPagesRows> {
        find_viewable_list_pages_rows(
            self.ctx,
            query,
            target_count,
            permission_cache,
            score_filter_cache,
        )
        .await
    }

    pub(super) async fn find_viewable_count_pages_rows(
        &self,
        query: PageQuery<'_>,
        target_count: usize,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<ViewableCountPagesRows> {
        find_viewable_count_pages_rows(self.ctx, query, target_count, permission_cache)
            .await
    }

    pub(super) async fn fetch_include_source(
        &self,
        current_site_id: i64,
        current_site_slug: &str,
        page_ref: &PageRef,
        cache: &mut IncludeSourceCache,
    ) -> Result<Option<IncludeSource>> {
        match page_ref.site() {
            Some(site_slug) if site_slug != current_site_slug => {
                let current_site = cache
                    .get_site_by_id_or_try_insert_with(current_site_id, || async {
                        SiteService::get_optional(
                            self.ctx,
                            Reference::Id(current_site_id),
                        )
                        .await
                        .or_raise(|| {
                            Error::new(
                                format!(
                                    "failed to get current include site ID {current_site_id}"
                                ),
                                ErrorType::Site,
                            )
                        })
                    })
                    .await?;
                let current_site_matches = current_site
                    .as_ref()
                    .is_some_and(|site| site_matches_wikidot_slug(site, site_slug));

                if current_site_matches
                    && let Some(source) = self
                        .fetch_include_source_from_site(
                            current_site_id,
                            current_site_slug,
                            page_ref.page(),
                            cache,
                        )
                        .await?
                {
                    return Ok(Some(source));
                }

                let Some(site) = cache
                    .get_site_by_slug_or_try_insert_with(site_slug, || async {
                        SiteService::get_optional(self.ctx, Reference::from(site_slug))
                            .await
                            .or_raise(|| {
                                Error::new(
                                    format!("failed to get include site '{site_slug}'"),
                                    ErrorType::Site,
                                )
                            })
                    })
                    .await?
                else {
                    return Ok(None);
                };

                self.fetch_include_source_from_site(
                    site.site_id,
                    &site.slug,
                    page_ref.page(),
                    cache,
                )
                .await
            }
            _ => {
                self.fetch_include_source_from_site(
                    current_site_id,
                    current_site_slug,
                    page_ref.page(),
                    cache,
                )
                .await
            }
        }
    }

    async fn fetch_include_source_from_site(
        &self,
        site_id: i64,
        site_slug: &str,
        page_slug: &str,
        cache: &mut IncludeSourceCache,
    ) -> Result<Option<IncludeSource>> {
        let wikitext = cache
            .get_or_try_insert_with(site_id, page_slug, || async {
                let page_ref = Reference::from(page_slug);
                let Some(page) =
                    PageService::get_optional(self.ctx, site_id, page_ref.clone())
                        .await?
                else {
                    return Ok(None);
                };

                let can_view = PermissionService::check_user_can(
                    self.ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id,
                        page_reference: Some(page_ref),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(page.page_category_id)),
                        action: Action::View,
                    },
                )
                .await?;
                if !can_view {
                    return Ok(None);
                }

                PageRevisionService::get_wikitext_optional(
                    self.ctx,
                    site_id,
                    Reference::Id(page.page_id),
                )
                .await
            })
            .await?;

        Ok(wikitext.map(|wikitext| IncludeSource {
            site_id,
            site_slug: site_slug.to_owned(),
            page_slug: trim_default(page_slug).to_owned(),
            wikitext,
        }))
    }
}

#[derive(Debug)]
pub(super) struct IncludeSource {
    pub(super) site_id: i64,
    pub(super) site_slug: String,
    pub(super) page_slug: String,
    pub(super) wikitext: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IncludeSourceCacheEntry {
    Available(String),
    Unavailable,
}

#[derive(Debug, Default)]
pub(super) struct IncludeSourceCache {
    sources_by_site: HashMap<i64, HashMap<String, IncludeSourceCacheEntry>>,
    sites_by_id: HashMap<i64, Option<SiteModel>>,
    sites_by_slug: HashMap<String, Option<SiteModel>>,
    pub(super) attachment_provenance: AttachmentProvenanceRegistry,
}

impl IncludeSourceCache {
    fn get(&self, site_id: i64, page_slug: &str) -> Option<&IncludeSourceCacheEntry> {
        self.sources_by_site
            .get(&site_id)
            .and_then(|sources| sources.get(page_slug))
    }

    pub(super) async fn get_or_try_insert_with<F, Fut>(
        &mut self,
        site_id: i64,
        page_slug: &str,
        load: F,
    ) -> Result<Option<String>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<String>>>,
    {
        // PageRef already normalizes the slug and separates #section or /path. PageService also treats an explicit `_default:` category as canonical.
        let page_slug = trim_default(page_slug);
        if let Some(entry) = self.get(site_id, page_slug) {
            return Ok(match entry {
                IncludeSourceCacheEntry::Available(wikitext) => Some(wikitext.clone()),
                IncludeSourceCacheEntry::Unavailable => None,
            });
        }

        let wikitext = load().await?;
        // Errors return above and are deliberately never cached. None is safe to retain within this render because missing, denied, and revision-less sources all have the same fail-closed include result.
        let entry = match &wikitext {
            Some(wikitext) => IncludeSourceCacheEntry::Available(wikitext.clone()),
            None => IncludeSourceCacheEntry::Unavailable,
        };
        self.sources_by_site
            .entry(site_id)
            .or_default()
            .insert(page_slug.to_owned(), entry);
        Ok(wikitext)
    }

    pub(super) async fn get_site_by_id_or_try_insert_with<F, Fut>(
        &mut self,
        site_id: i64,
        load: F,
    ) -> Result<Option<SiteModel>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<SiteModel>>>,
    {
        if let Some(site) = self.sites_by_id.get(&site_id) {
            return Ok(site.clone());
        }

        let site = load().await?;
        self.sites_by_id.insert(site_id, site.clone());
        Ok(site)
    }

    pub(super) async fn get_site_by_slug_or_try_insert_with<F, Fut>(
        &mut self,
        site_slug: &str,
        load: F,
    ) -> Result<Option<SiteModel>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<SiteModel>>>,
    {
        if let Some(site) = self.sites_by_slug.get(site_slug) {
            return Ok(site.clone());
        }

        let site = load().await?;
        self.sites_by_slug
            .insert(site_slug.to_owned(), site.clone());
        Ok(site)
    }
}
