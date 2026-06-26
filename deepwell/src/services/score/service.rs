/*
 * services/score/service.rs
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

use super::impls::*;
use super::prelude::*;
use sea_orm::Statement;

#[derive(Debug)]
pub struct ScoreService;

impl ScoreService {
    pub async fn score(ctx: &ServiceContext<'_>, page_id: i64) -> Result<ScoreValue> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!("failed to evaluate score for page ID {}", page_id),
                ErrorType::PageVote,
            )
        };

        let imported_rating = Self::imported_rating(ctx, page_id)
            .await
            .or_raise(make_error)?;
        let condition = Self::build_condition(page_id, imported_rating.is_some());
        let scorer = Self::get_scorer(ctx, page_id).await.or_raise(make_error)?;
        let local_score = scorer.score(txn, condition).await.or_raise(make_error)?;

        match imported_rating {
            Some(imported_rating) => {
                Self::combine_imported_rating(imported_rating, local_score)
            }
            None => Ok(local_score),
        }
    }

    /// Gets the correct `Scorer` implementation for this page.
    ///
    /// Site-level score settings are not implemented yet, so use Wikidot's
    /// ordinary signed-vote sum as the deterministic default.
    pub async fn get_scorer(
        _ctx: &ServiceContext<'_>,
        _page_id: i64,
    ) -> Result<impl Scorer> {
        Ok(SumScorer)
    }

    /// Helper method for retrieving a `VoteMap` for a page.
    /// Takes inputs as used in `Scorer.score()`.
    ///
    /// This can become a full service method, see above.
    pub(crate) async fn collect_votes(
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<VoteMap> {
        // Query for votes aggregated by value.
        //
        // As raw SQL:
        //
        // SELECT value, COUNT(value)
        // FROM page_vote
        // WHERE page_id = $1
        // AND deleted_at IS NULL
        // AND disabled_at IS NULL
        // GROUP BY value;

        #[derive(FromQueryResult, Debug)]
        struct VoteCountRow {
            value: VoteValue,
            count: u64,
        }

        let counts = PageVote::find()
            .column(page_vote::Column::Value)
            .column_as(page_vote::Column::Value.count(), "count")
            .filter(condition)
            .group_by(page_vote::Column::Value)
            .into_model::<VoteCountRow>()
            .all(txn)
            .await
            .or_raise(|| Error::new("failed to collect votes", ErrorType::PageVote))?;

        // Gather results into map
        let mut map = VoteMap::new();

        for VoteCountRow { value, count } in counts {
            map.insert(value, count);
        }

        Ok(map)
    }

    async fn imported_rating(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<Option<i64>> {
        #[derive(FromQueryResult, Debug)]
        struct TableExistsRow {
            exists: bool,
        }

        #[derive(FromQueryResult, Debug)]
        struct ImportedRatingRow {
            imported_rating: i64,
        }

        let txn = ctx.transaction();
        let table_exists_statement = Statement::from_string(
            txn.get_database_backend(),
            "SELECT to_regclass('wikidot_page_snapshot') IS NOT NULL AS exists",
        );
        let table_exists = TableExistsRow::find_by_statement(table_exists_statement)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check imported page rating table",
                    ErrorType::PageVote,
                )
            })?
            .map(|row| row.exists)
            .unwrap_or(false);

        if !table_exists {
            return Ok(None);
        }

        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT imported_rating FROM wikidot_page_snapshot WHERE page_id = {}",
                page_id,
            ),
        );

        let row = ImportedRatingRow::find_by_statement(statement)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new("failed to load imported page rating", ErrorType::PageVote)
            })?;

        Ok(row.map(|row| row.imported_rating))
    }

    fn combine_imported_rating(
        imported_rating: i64,
        local_score: ScoreValue,
    ) -> Result<ScoreValue> {
        match local_score {
            ScoreValue::Integer(value) => {
                let combined = imported_rating.checked_add(value).ok_or_else(|| {
                    Error::new(
                        "imported page rating overflowed while combining local votes",
                        ErrorType::PageVote,
                    )
                })?;
                Ok(ScoreValue::Integer(combined))
            }
            ScoreValue::Float(value) => {
                Ok(ScoreValue::Float(imported_rating as f64 + value))
            }
        }
    }

    fn build_condition(page_id: i64, local_votes_only: bool) -> Condition {
        let mut condition = Condition::all()
            .add(page_vote::Column::PageId.eq(page_id))
            .add(page_vote::Column::DeletedAt.is_null())
            .add(page_vote::Column::DisabledAt.is_null());

        if local_votes_only {
            condition = condition.add(page_vote::Column::FromWikidot.eq(false));
        }

        condition
    }
}
