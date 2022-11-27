/*
 * services/system_filter/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::models::sea_orm_active_enums::SystemFilterType;
use crate::models::system_filter::{
    self, Entity as SystemFilter, Model as SystemFilterModel,
};
use regex::{Regex, RegexSet};

#[derive(Debug)]
pub struct SystemFilterService;

impl SystemFilterService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateSystemFilter {
            filter_type,
            regex,
            reason,
        }: CreateSystemFilter,
    ) -> Result<SystemFilterModel> {
        let txn = ctx.transaction();

        tide::log::info!(
            "Creating system filter with regex '{regex}' because '{reason}'",
        );

        // Ensure the regular expression is valid
        if Regex::new(&regex).is_err() {
            tide::log::error!("Passed regular expression pattern is invalid: {regex}");
            return Err(Error::BadRequest);
        }

        // Ensure there aren't conflicts
        Self::check_conflicts(ctx, filter_type, &regex, "create").await?;

        let model = system_filter::ActiveModel {
            filter_type: Set(filter_type),
            regex: Set(regex),
            reason: Set(reason),
            ..Default::default()
        };
        let filter = model.insert(txn).await?;
        Ok(filter)
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateSystemFilter {
            filter_id,
            regex,
            reason,
        }: UpdateSystemFilter,
    ) -> Result<SystemFilterModel> {
        let txn = ctx.transaction();

        tide::log::info!("Updating system filter with ID {filter_id}: regex '{regex}', reason '{reason}'");

        let model = system_filter::ActiveModel {
            filter_id: Set(filter_id),
            regex: Set(regex),
            reason: Set(reason),
            updated_at: Set(Some(now())),
            ..Default::default()
        };
        let filter = model.update(txn).await?;
        Ok(filter)
    }

    pub async fn delete(ctx: &ServiceContext<'_>, filter_id: i64) -> Result<()> {
        let txn = ctx.transaction();

        tide::log::info!("Deleting system filter with ID {filter_id}");

        // Ensure filter exists
        let filter = Self::get(ctx, filter_id).await?;
        if filter.deleted_at.is_some() {
            tide::log::error!("Attempting to delete already-deleted system filter");
            return Err(Error::BadRequest);
        }

        // Delete the filter
        let model = system_filter::ActiveModel {
            filter_id: Set(filter_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await?;
        Ok(())
    }

    pub async fn undelete(
        ctx: &ServiceContext<'_>,
        filter_id: i64,
    ) -> Result<SystemFilterModel> {
        let txn = ctx.transaction();

        tide::log::info!("Undeleting system filter with ID {filter_id}");

        let filter = Self::get(ctx, filter_id).await?;
        if filter.deleted_at.is_none() {
            tide::log::error!("Attempting to un-delete extant system filter");
            return Err(Error::BadRequest);
        }

        // Ensure it doesn't conflict with a since-added filter
        Self::check_conflicts(ctx, filter.filter_type, &filter.regex, "undelete").await?;

        // Un-delete the filter
        let model = system_filter::ActiveModel {
            filter_id: Set(filter_id),
            deleted_at: Set(None),
            ..Default::default()
        };
        let filter = model.update(txn).await?;
        Ok(filter)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        filter_id: i64,
    ) -> Result<SystemFilterModel> {
        find_or_error(Self::get_optional(ctx, filter_id)).await
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        filter_id: i64,
    ) -> Result<Option<SystemFilterModel>> {
        tide::log::info!("Getting system filter with ID {filter_id}");

        let txn = ctx.transaction();
        let filter = SystemFilter::find_by_id(filter_id).one(txn).await?;
        Ok(filter)
    }

    /// Get all system filters of a type.
    ///
    /// The `deleted` argument:
    /// * If it is `Some(true)`, then it only returns filters which have been deleted.
    /// * If it is `Some(false)`, then it only returns filters which are extant.
    /// * If it is `None`, then it returns all filters regardless of deletion status.
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        filter_type: SystemFilterType,
        deleted: Option<bool>,
    ) -> Result<Vec<SystemFilterModel>> {
        let txn = ctx.transaction();

        tide::log::info!("Getting all system filters of type {filter_type}");

        let deleted_condition = match deleted {
            Some(true) => Some(system_filter::Column::DeletedAt.is_not_null()),
            Some(false) => Some(system_filter::Column::DeletedAt.is_null()),
            None => None,
        };

        let filters = SystemFilter::find()
            .filter(
                Condition::all()
                    .add(system_filter::Column::FilterType.eq(filter_type))
                    .add_option(deleted_condition),
            )
            .all(txn)
            .await?;

        Ok(filters)
    }

    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        filter_type: SystemFilterType,
        regex: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let result = SystemFilter::find()
            .filter(
                Condition::all()
                    .add(system_filter::Column::FilterType.eq(filter_type))
                    .add(system_filter::Column::Regex.eq(regex))
                    .add(system_filter::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        match result {
            None => Ok(()),
            Some(_) => {
                tide::log::error!("System filter '{regex}' for {filter_type} already exists, cannot {action}");
                Err(Error::Conflict)
            }
        }
    }
}
