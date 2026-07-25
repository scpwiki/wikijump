/*
 * services/relation/page_attribution.rs
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

use super::RelationService;
use crate::error::prelude::{Error, ErrorType, ExnError, Result, ResultExt};
use crate::models::relation::{self, Entity as Relation, Model as RelationModel};
use crate::services::PageService;
use crate::services::ServiceContext;
use crate::types::Reference;
use crate::types::{RelationObjectType, RelationType};
use crate::utils::now;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::collections::BTreeSet;
use time::{Date, OffsetDateTime};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PageAttributionKind {
    Author,
    Rewrite,
    Translator,
    Maintainer,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageAttributionMetadata {
    pub attribution_type: PageAttributionKind,
    pub attribution_date: Date,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PageAttribution {
    pub relation_id: i64,
    pub page_id: i64,
    pub user_id: i64,
    pub created_by: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub overwritten_by: Option<i64>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub overwritten_at: Option<OffsetDateTime>,
    pub deleted_by: Option<i64>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub metadata: PageAttributionMetadata,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePageAttribution {
    pub page_id: i64,
    pub user_id: i64,
    pub metadata: PageAttributionMetadata,
    pub created_by: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PageAttributionEntry {
    pub user_id: i64,
    pub metadata: PageAttributionMetadata,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetPageAttributions<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub updated_by: i64,
    pub attributions: Vec<PageAttributionEntry>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClearPageAttributions<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub removed_by: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageAttributions<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
}

impl RelationService {
    /// Creates a page attribution relation without overwriting other attributions
    /// for the same page / user combination. This mirrors the unique constraint of
    /// (page_id, user_id, attribution_type, attribution_date) from the former
    /// dedicated table.
    pub async fn create_page_attribution(
        ctx: &ServiceContext<'_>,
        CreatePageAttribution {
            page_id,
            user_id,
            metadata,
            created_by,
        }: CreatePageAttribution,
    ) -> Result<PageAttribution> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create page attribution entry for user ID {} on page ID {}, created by user ID {} (metadata {:?})",
                    user_id, page_id, created_by, metadata,
                ),
                ErrorType::PageAttributionRelation,
            )
        };

        let metadata_json = serde_json::to_value(metadata).or_raise(make_error)?;

        if let Some(model) = find_page_attribution(ctx, page_id, user_id, &metadata_json)
            .await
            .or_raise(make_error)?
        {
            debug!(
                "Page attribution already exists for page {} user {} ({:?} @ {})",
                page_id, user_id, metadata.attribution_type, metadata.attribution_date,
            );
            let attrib = PageAttribution::try_from(model).or_raise(make_error)?;
            return Ok(attrib);
        }

        debug!(
            "Creating page attribution for page {} user {} ({:?} @ {})",
            page_id, user_id, metadata.attribution_type, metadata.attribution_date,
        );

        RelationType::PageAttribution
            .types()
            .check(RelationObjectType::Page, RelationObjectType::User);

        let model = relation::ActiveModel {
            relation_type: Set(RelationType::PageAttribution),
            dest_type: Set(RelationObjectType::Page),
            dest_id: Set(page_id),
            from_type: Set(RelationObjectType::User),
            from_id: Set(user_id),
            metadata: Set(metadata_json),
            created_by: Set(created_by),
            ..Default::default()
        };

        let model = model.insert(txn).await.or_raise(make_error)?;
        let attrib = PageAttribution::try_from(model).or_raise(make_error)?;
        Ok(attrib)
    }

    #[allow(dead_code)]
    pub async fn page_attribution_exists(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        user_id: i64,
        metadata: &PageAttributionMetadata,
    ) -> Result<bool> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check if page attribution entry for user ID {} on page ID {} (metadata {:?})",
                    user_id, page_id, metadata,
                ),
                ErrorType::PageAttributionRelation,
            )
        };

        let metadata_json = serde_json::to_value(metadata).or_raise(make_error)?;

        find_page_attribution(ctx, page_id, user_id, &metadata_json)
            .await
            .map(|relation| relation.is_some())
    }

    pub async fn get_page_attributions(
        ctx: &ServiceContext<'_>,
        GetPageAttributions { site_id, page }: GetPageAttributions<'_>,
    ) -> Result<Vec<PageAttribution>> {
        let txn = ctx.transaction();
        let page = PageService::get(ctx, site_id, page).await.or_raise(|| {
            Error::new(
                "failed to get page attributions",
                ErrorType::PageAttributionRelation,
            )
        })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to get page attributions for page '{}' (ID {}) on site ID {}",
                    page.slug, page.page_id, site_id,
                ),
                ErrorType::PageAttributionRelation,
            )
        };

        let models = Relation::find()
            .filter(page_attributions_condition(page.page_id))
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut attributions = models
            .into_iter()
            .map(PageAttribution::try_from)
            .collect::<Result<Vec<_>>>()
            .or_raise(make_error)?;

        sort_attributions(&mut attributions);
        Ok(attributions)
    }

    pub async fn set_page_attributions(
        ctx: &ServiceContext<'_>,
        SetPageAttributions {
            site_id,
            page,
            updated_by,
            attributions,
        }: SetPageAttributions<'_>,
    ) -> Result<Vec<PageAttribution>> {
        let txn = ctx.transaction();
        let page = PageService::get(ctx, site_id, page).await.or_raise(|| {
            Error::new(
                "failed to set page attributions",
                ErrorType::PageAttributionRelation,
            )
        })?;

        let attributions_len = attributions.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to set {} page attributions for page '{}' (ID {}) on site ID {}",
                    attributions_len, page.slug, page.page_id, site_id,
                ),
                ErrorType::PageAttributionRelation,
            )
        };

        // Delete existing active attributions for this page.
        let delete_model = relation::ActiveModel {
            deleted_by: Set(Some(updated_by)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        Relation::update_many()
            .set(delete_model)
            .filter(page_attributions_condition(page.page_id))
            .exec(txn)
            .await
            .or_raise(make_error)?;

        // Insert the new set.
        let mut created = Vec::with_capacity(attributions_len);
        let mut seen = BTreeSet::new();
        for PageAttributionEntry { user_id, metadata } in attributions {
            if !seen.insert((user_id, metadata)) {
                continue;
            }

            let attribution = Self::create_page_attribution(
                ctx,
                CreatePageAttribution {
                    page_id: page.page_id,
                    user_id,
                    metadata,
                    created_by: updated_by,
                },
            )
            .await
            .or_raise(make_error)?;

            created.push(attribution);
        }

        sort_attributions(&mut created);
        Ok(created)
    }

    pub async fn clear_page_attributions(
        ctx: &ServiceContext<'_>,
        ClearPageAttributions {
            site_id,
            page,
            removed_by,
        }: ClearPageAttributions<'_>,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let page = PageService::get(ctx, site_id, page).await.or_raise(|| {
            Error::new(
                "failed to clear page attributions",
                ErrorType::PageAttributionRelation,
            )
        })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to clear page attributions for page '{}' (ID {}) on site ID {}",
                    page.slug, page.page_id, site_id,
                ),
                ErrorType::PageAttributionRelation,
            )
        };

        let delete_model = relation::ActiveModel {
            deleted_by: Set(Some(removed_by)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        Relation::update_many()
            .set(delete_model)
            .filter(page_attributions_condition(page.page_id))
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(())
    }
}

/// Builds a condition for querying one specific page attribution entry.
fn page_attribution_condition(
    page_id: i64,
    user_id: i64,
    metadata_json: &serde_json::Value,
) -> Condition {
    Condition::all()
        .add(relation::Column::RelationType.eq(RelationType::PageAttribution))
        .add(relation::Column::DestType.eq(RelationObjectType::Page))
        .add(relation::Column::DestId.eq(page_id))
        .add(relation::Column::FromType.eq(RelationObjectType::User))
        .add(relation::Column::FromId.eq(user_id))
        .add(relation::Column::Metadata.eq(metadata_json.clone()))
        .add(relation::Column::OverwrittenAt.is_null())
        .add(relation::Column::DeletedAt.is_null())
}

/// Builds a condition for querying all page attribution entries for a page.
fn page_attributions_condition(page_id: i64) -> Condition {
    Condition::all()
        .add(relation::Column::RelationType.eq(RelationType::PageAttribution))
        .add(relation::Column::DestType.eq(RelationObjectType::Page))
        .add(relation::Column::FromType.eq(RelationObjectType::User))
        .add(relation::Column::DestId.eq(page_id))
        .add(relation::Column::OverwrittenAt.is_null())
        .add(relation::Column::DeletedAt.is_null())
}

fn convert_model(model: RelationModel) -> Result<PageAttribution> {
    assert_eq!(model.relation_type, RelationType::PageAttribution);
    assert_eq!(model.dest_type, RelationObjectType::Page);
    assert_eq!(model.from_type, RelationObjectType::User);

    let metadata: PageAttributionMetadata = serde_json::from_value(model.metadata)
        .or_raise(|| {
            Error::new(
                "failed to convert RelationModel to PageAttribution",
                ErrorType::PageAttributionRelation,
            )
        })?;

    Ok(PageAttribution {
        relation_id: model.relation_id,
        page_id: model.dest_id,
        user_id: model.from_id,
        created_by: model.created_by,
        created_at: model.created_at,
        overwritten_by: model.overwritten_by,
        overwritten_at: model.overwritten_at,
        deleted_by: model.deleted_by,
        deleted_at: model.deleted_at,
        metadata,
    })
}

impl TryFrom<RelationModel> for PageAttribution {
    type Error = ExnError;

    fn try_from(model: RelationModel) -> Result<Self> {
        convert_model(model)
    }
}

async fn find_page_attribution(
    ctx: &ServiceContext<'_>,
    page_id: i64,
    user_id: i64,
    metadata_json: &serde_json::Value,
) -> Result<Option<RelationModel>> {
    let make_error = || {
        Error::new(
            format!(
                "failed to query for page attribution user ID {} on page ID {}",
                user_id, page_id,
            ),
            ErrorType::PageAttributionRelation,
        )
    };

    let txn = ctx.transaction();
    let model = Relation::find()
        .filter(page_attribution_condition(page_id, user_id, metadata_json))
        .one(txn)
        .await
        .or_raise(make_error)?;

    Ok(model)
}

fn sort_attributions(attributions: &mut [PageAttribution]) {
    attributions.sort_by(|a, b| {
        b.metadata
            .attribution_date
            .cmp(&a.metadata.attribution_date)
            .then_with(|| {
                a.metadata
                    .attribution_type
                    .cmp(&b.metadata.attribution_type)
            })
            .then_with(|| a.user_id.cmp(&b.user_id))
    });
}
