/*
 * services/filter/service.rs
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

use super::matcher::{FilterMatcher, FilterSummary};
use super::structs::{CreateFilter, FilterClass, FilterType, UpdateFilter};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::filter::{self, Entity as Filter, Model as FilterModel};
use crate::services::ServiceContext;
use crate::types::Maybe;
use crate::utils::now;
use crate::utils::trim_start_matches_in_place;
use paste::paste;
use regex::{Regex, RegexSet};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};

#[derive(Debug)]
pub struct FilterService;

impl FilterService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: Option<i64>,
        CreateFilter {
            affects_user,
            affects_email,
            affects_page,
            affects_file,
            affects_forum,
            case_sensitive,
            mut regex,
            description,
        }: CreateFilter,
    ) -> Result<FilterModel> {
        let txn = ctx.transaction();

        info!("Creating filter with regex '{regex}' because '{description}'");

        let regex_2 = regex.clone();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create filter with base regex pattern '{}': user {}, email {}, page {}, file {}, forum {}, case sensitive {}",
                    regex_2,
                    affects_user,
                    affects_email,
                    affects_page,
                    affects_file,
                    affects_forum,
                    case_sensitive,
                ),
                ErrorType::Filter,
            )
        };

        Self::check_conflicts(ctx, site_id, &regex, "create")
            .await
            .or_raise(make_error)?;

        if !case_sensitive {
            regex = str!(regex.trim_start_matches("(?i)"));
            regex.insert_str(0, "(?i)");
        }

        let _ = Regex::new(&regex).or_raise(|| {
            Error::new(
                format!("failed to create filter with regex pattern '{}'", regex),
                ErrorType::FilterRegexInvalid { regex: str!(regex) },
            )
        })?;

        let model = filter::ActiveModel {
            site_id: Set(site_id),
            affects_user: Set(affects_user),
            affects_email: Set(affects_email),
            affects_page: Set(affects_page),
            affects_file: Set(affects_file),
            affects_forum: Set(affects_forum),
            regex: Set(regex),
            description: Set(description),
            ..Default::default()
        };
        let filter = model.insert(txn).await.or_raise(make_error)?;
        Ok(filter)
    }

    #[allow(dead_code)] // TEMP
    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateFilter {
            filter_id,
            affects_user,
            affects_email,
            affects_page,
            affects_file,
            affects_forum,
            case_sensitive,
            mut regex,
            description,
        }: UpdateFilter,
    ) -> Result<FilterModel> {
        let txn = ctx.transaction();

        info!("Updating filter with ID {filter_id}");

        let regex2 = regex.to_option().cloned();
        let make_error = || {
            let mut message = format!("failed to update filter ID {filter_id}");

            if let Some(regex) = regex2 {
                str_write!(&mut message, " (new regex '{regex}')");
            }

            Error::new(message, ErrorType::Filter)
        };

        let mut model = filter::ActiveModel {
            filter_id: Set(filter_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        if let Maybe::Set(case_sensitive) = case_sensitive {
            match regex {
                Maybe::Set(ref mut regex) if !case_sensitive => {
                    regex.insert_str(0, "(?i)")
                }

                Maybe::Set(_) => {}

                Maybe::Unset => {
                    let mut model_regex = str!(model.get(filter::Column::Regex).as_ref());
                    trim_start_matches_in_place(&mut model_regex, "(?i)");

                    if !case_sensitive {
                        model_regex.insert_str(0, "(?i)");
                    }

                    model.regex = Set(model_regex);
                }
            }
        };

        if let Maybe::Set(affects) = affects_user {
            model.affects_user = Set(affects);
        }

        if let Maybe::Set(affects) = affects_email {
            model.affects_email = Set(affects);
        }

        if let Maybe::Set(affects) = affects_page {
            model.affects_page = Set(affects);
        }

        if let Maybe::Set(affects) = affects_file {
            model.affects_file = Set(affects);
        }

        if let Maybe::Set(affects) = affects_forum {
            model.affects_forum = Set(affects);
        }

        if let Maybe::Set(regex) = regex {
            model.regex = Set(regex);
        }

        if let Maybe::Set(description) = description {
            model.description = Set(description);
        }

        let filter = model.update(txn).await.or_raise(make_error)?;
        Ok(filter)
    }

    #[allow(dead_code)] // TEMP
    pub async fn delete(ctx: &ServiceContext<'_>, filter_id: i64) -> Result<()> {
        info!("Deleting filter with ID {filter_id}");
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to delete filter ID {}", filter_id),
                ErrorType::Filter,
            )
        };

        let filter = Self::get(ctx, filter_id).await.or_raise(make_error)?;

        if filter.deleted_at.is_some() {
            error!("Attempting to remove already-deleted filter");
            bail!(Error::new(
                format!("cannot delete filter ID {}, is already deleted", filter_id),
                ErrorType::FilterNotFound,
            ));
        }

        let model = filter::ActiveModel {
            filter_id: Set(filter_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await.or_raise(make_error)?;
        Ok(())
    }

    /// Restores a filter, causing it to be undeleted.
    #[allow(dead_code)] // TEMP
    pub async fn restore(
        ctx: &ServiceContext<'_>,
        filter_id: i64,
    ) -> Result<FilterModel> {
        let txn = ctx.transaction();

        info!("Undeleting filter with ID {filter_id}");

        let make_error = || {
            Error::new(
                format!("failed to restore (undelete) filter ID {}", filter_id),
                ErrorType::Filter,
            )
        };

        let filter = Self::get(ctx, filter_id).await?;
        if filter.deleted_at.is_none() {
            error!("Attempting to un-delete extant filter");
            bail!(Error::new(
                format!(
                    "cannot restore (undelete) filter ID {} that isn't deleted",
                    filter_id,
                ),
                ErrorType::FilterNotDeleted
            ));
        }

        Self::check_conflicts(ctx, filter.site_id, &filter.regex, "restore")
            .await
            .or_raise(make_error)?;

        let model = filter::ActiveModel {
            filter_id: Set(filter_id),
            deleted_at: Set(None),
            ..Default::default()
        };
        let filter = model.update(txn).await.or_raise(make_error)?;
        Ok(filter)
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, filter_id: i64) -> Result<FilterModel> {
        find_or_error!(Self::get_optional(ctx, filter_id), "filter", Filter)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        filter_id: i64,
    ) -> Result<Option<FilterModel>> {
        let make_error = || {
            Error::new(
                format!("failed to get filter ID {}", filter_id),
                ErrorType::Filter,
            )
        };

        let txn = ctx.transaction();
        let filter = Filter::find_by_id(filter_id)
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(filter)
    }

    /// Get all filters of a type.
    ///
    /// For the `filter_class` argument, see `FilterClass`.
    /// Note that this argument is what provides the `site_id` argument (or not).
    ///
    /// The `filter_type` argument:
    /// * If it is `Some(_)`, it determines what kind of object is being filtered.
    /// * If it is `None`, then it returns everything.
    ///
    /// The `deleted` argument:
    /// * If it is `Some(true)`, then it only returns filters which have been deleted.
    /// * If it is `Some(false)`, then it only returns filters which are extant.
    /// * If it is `None`, then it returns all filters regardless of deletion status.
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        filter_class: FilterClass,
        filter_type: Option<FilterType>,
        deleted: Option<bool>,
    ) -> Result<Vec<FilterModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            let mut message = format!("failed to get all {} ", filter_class.name());

            match deleted {
                Some(true) => message.push_str("deleted "),
                Some(false) => message.push_str("extant "),
                None => (),
            }

            message.push_str("filters ");

            if let Some(filter_type) = filter_type {
                str_write!(&mut message, "of type {:?} ", filter_type);
            }

            match filter_class {
                FilterClass::Platform => message.push_str("on the platform"),
                FilterClass::Site(site_id) => {
                    str_write!(&mut message, "on site ID {}", site_id)
                }
                FilterClass::PlatformAndSite(site_id) => {
                    str_write!(&mut message, "on the platform and on site ID {}", site_id)
                }
            }

            Error::new(message, ErrorType::Filter)
        };

        let filter_condition =
            filter_type.map(|filter_type| filter_type.into_column().eq(true));

        let deleted_condition = match deleted {
            Some(true) => Some(filter::Column::DeletedAt.is_not_null()),
            Some(false) => Some(filter::Column::DeletedAt.is_null()),
            None => None,
        };

        let filters = Filter::find()
            .filter(
                Condition::all()
                    .add(filter_class.to_condition())
                    .add_option(filter_condition)
                    .add_option(deleted_condition),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(filters)
    }

    /// Get all filters of a type, specifically extracting the regular expressions.
    ///
    /// This only pulls extant filters, as those are the only ones which are enforced.
    // TODO cache this somehow
    //      maybe so that it stores the RegexSet and deletes it if an insert/update/etc
    //      above occurs to that filter class/type
    pub async fn get_matcher(
        ctx: &ServiceContext<'_>,
        filter_class: FilterClass,
        filter_type: FilterType,
    ) -> Result<FilterMatcher> {
        let make_error = || Error::new("failed to get filter matcher", ErrorType::Filter);

        let filters = Self::get_all(ctx, filter_class, Some(filter_type), Some(false))
            .await
            .or_raise(make_error)?;

        let mut regexes = Vec::new();
        let mut filter_data = Vec::new();

        for FilterModel {
            filter_id,
            regex,
            description,
            ..
        } in filters
        {
            filter_data.push(FilterSummary {
                filter_id,
                regex: str!(regex.as_str()),
                description,
            });
            regexes.push(regex);
        }

        let regex_set = RegexSet::new(regexes).or_raise(|| {
            Error::new(
                "invalid regular expression found in database",
                ErrorType::Filter,
            )
        })?;

        Ok(FilterMatcher::new(regex_set, filter_data))
    }

    /// Checks if creating / reinstating this filter would cause constraint violations.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        site_id: Option<i64>,
        regex: &str,
        action: &str,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed conflict check for filter '{}' for {:?}, cannot {}",
                    regex, site_id, action,
                ),
                ErrorType::Filter,
            )
        };

        let txn = ctx.transaction();
        let result = Filter::find()
            .filter(
                Condition::all()
                    .add(filter::Column::SiteId.eq(site_id))
                    // Check for both case sensitive and insensitive variants
                    .add(
                        filter::Column::Regex
                            .eq(regex)
                            .or(filter::Column::Regex.eq(format!("(?i){regex}"))),
                    )
                    .add(filter::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        match result {
            None => Ok(()),
            Some(_) => {
                error!(
                    "Filter '{}' for {:?} already exists, cannot {}",
                    regex, site_id, action,
                );
                bail!(Error::new(
                    format!(
                        "cannot {}, filter '{}' for {:?} already exists",
                        action, regex, site_id,
                    ),
                    ErrorType::FilterExists,
                ));
            }
        }
    }
}
