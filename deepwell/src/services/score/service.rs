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

use super::impls::{MeanScorer, SumScorer};
use super::scorer::Scorer;
use super::structs::{ScoreType, VoteMap, VoteType, VoteValue};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page_vote::{self, Entity as PageVote};
use crate::services::ServiceContext;
use crate::services::settings::PageRatingType;
use crate::services::{PageService, SettingsService};
use ftml::data::ScoreValue;
use sea_orm::Statement;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    FromQueryResult, QueryFilter, QuerySelect,
};

#[derive(Debug)]
pub struct ScoreService;

#[derive(Debug)]
enum ConfiguredScorer {
    Sum(SumScorer),
    Mean(MeanScorer),
}

impl ConfiguredScorer {
    fn vote_store_key(&self) -> &'static str {
        match self {
            Self::Sum(_) => "points",
            Self::Mean(_) => "stars",
        }
    }
}

impl Scorer for ConfiguredScorer {
    fn score_type(&self) -> ScoreType {
        match self {
            Self::Sum(scorer) => scorer.score_type(),
            Self::Mean(scorer) => scorer.score_type(),
        }
    }

    fn accepts_vote_type(&self, vote_type: VoteType) -> bool {
        match self {
            Self::Sum(scorer) => scorer.accepts_vote_type(vote_type),
            Self::Mean(scorer) => scorer.accepts_vote_type(vote_type),
        }
    }

    async fn score(
        &self,
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<ScoreValue> {
        match self {
            Self::Sum(scorer) => scorer.score(txn, condition).await,
            Self::Mean(scorer) => scorer.score(txn, condition).await,
        }
    }
}

impl ScoreService {
    pub async fn scores_bulk(
        ctx: &ServiceContext<'_>,
        page_ids: &[i64],
    ) -> Result<Vec<(i64, ScoreValue)>> {
        if page_ids.is_empty() {
            return Ok(Vec::new());
        }

        for &page_id in page_ids {
            let scorer = Self::get_scorer(ctx, page_id).await?;
            if scorer.score_type() != ScoreType::Sum {
                return Self::scores_sequential(ctx, page_ids).await;
            }
        }

        #[derive(FromQueryResult, Debug)]
        struct BulkScoreRow {
            page_id: i64,
            imported_rating: Option<i64>,
            local_score: Option<i64>,
        }

        let txn = ctx.transaction();
        let values = page_ids
            .iter()
            .enumerate()
            .map(|(index, page_id)| format!("({}, {})", page_id, index))
            .collect::<Vec<_>>()
            .join(", ");
        let statement = if Self::imported_rating_table_exists(ctx).await? {
            Statement::from_string(
                txn.get_database_backend(),
                format!(
                    "WITH input(page_id, ordinal) AS (VALUES {values}) \
                     SELECT input.page_id, snapshot.imported_rating, COALESCE(SUM(vote.value), 0) AS local_score \
                     FROM input \
                     LEFT JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = input.page_id \
                     LEFT JOIN page_vote vote ON vote.page_id = input.page_id \
                       AND vote.rating_system = 'points' \
                       AND vote.deleted_at IS NULL \
                       AND vote.disabled_at IS NULL \
                       AND (snapshot.imported_rating IS NULL OR vote.from_wikidot = false) \
                     GROUP BY input.page_id, input.ordinal, snapshot.imported_rating \
                     ORDER BY input.ordinal",
                ),
            )
        } else {
            Statement::from_string(
                txn.get_database_backend(),
                format!(
                    "WITH input(page_id, ordinal) AS (VALUES {values}) \
                     SELECT input.page_id, NULL::BIGINT AS imported_rating, COALESCE(SUM(vote.value), 0) AS local_score \
                     FROM input \
                     LEFT JOIN page_vote vote ON vote.page_id = input.page_id \
                       AND vote.rating_system = 'points' \
                       AND vote.deleted_at IS NULL \
                       AND vote.disabled_at IS NULL \
                     GROUP BY input.page_id, input.ordinal \
                     ORDER BY input.ordinal",
                ),
            )
        };

        let rows = BulkScoreRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to evaluate page scores in bulk",
                    ErrorType::PageVote,
                )
            })?;
        let mut scores = Vec::with_capacity(rows.len());
        for BulkScoreRow {
            page_id,
            imported_rating,
            local_score,
        } in rows
        {
            let local_score = ScoreValue::Integer(local_score.unwrap_or(0));
            let score = match imported_rating {
                Some(imported_rating) => {
                    Self::combine_imported_rating(imported_rating, local_score)?
                }
                None => local_score,
            };
            scores.push((page_id, score));
        }
        Ok(scores)
    }

    async fn scores_sequential(
        ctx: &ServiceContext<'_>,
        page_ids: &[i64],
    ) -> Result<Vec<(i64, ScoreValue)>> {
        let mut scores = Vec::with_capacity(page_ids.len());
        for &page_id in page_ids {
            scores.push((page_id, Self::score(ctx, page_id).await?));
        }
        Ok(scores)
    }

    pub async fn score(ctx: &ServiceContext<'_>, page_id: i64) -> Result<ScoreValue> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!("failed to evaluate score for page ID {}", page_id),
                ErrorType::PageVote,
            )
        };

        let scorer = Self::get_scorer(ctx, page_id).await.or_raise(make_error)?;
        let imported_rating = if scorer.score_type() == ScoreType::Mean {
            None
        } else {
            Self::imported_rating(ctx, page_id)
                .await
                .or_raise(make_error)?
        };
        let condition = Self::build_condition(
            page_id,
            imported_rating.is_some(),
            scorer.vote_store_key(),
        );
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
    async fn get_scorer(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<ConfiguredScorer> {
        let page = PageService::get_direct(ctx, page_id, true).await?;
        let settings = SettingsService::get_page_rating_settings(
            ctx,
            page.site_id,
            page.page_category_id,
        )
        .await?;
        Ok(match settings.rating_type {
            PageRatingType::Stars => ConfiguredScorer::Mean(MeanScorer),
            PageRatingType::Plus | PageRatingType::PlusMinus => {
                ConfiguredScorer::Sum(SumScorer)
            }
        })
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
        struct ImportedRatingRow {
            imported_rating: i64,
        }

        let txn = ctx.transaction();
        if !Self::imported_rating_table_exists(ctx).await? {
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

    async fn imported_rating_table_exists(ctx: &ServiceContext<'_>) -> Result<bool> {
        #[derive(FromQueryResult, Debug)]
        struct TableExistsRow {
            exists: bool,
        }

        let txn = ctx.transaction();
        let table_exists_statement = Statement::from_string(
            txn.get_database_backend(),
            "SELECT to_regclass('wikidot_page_snapshot') IS NOT NULL AS exists",
        );
        Ok(TableExistsRow::find_by_statement(table_exists_statement)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check imported page rating table",
                    ErrorType::PageVote,
                )
            })?
            .map(|row| row.exists)
            .unwrap_or(false))
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

    fn build_condition(
        page_id: i64,
        local_votes_only: bool,
        rating_system: &str,
    ) -> Condition {
        let mut condition = Condition::all()
            .add(page_vote::Column::PageId.eq(page_id))
            .add(page_vote::Column::RatingSystem.eq(rating_system))
            .add(page_vote::Column::DeletedAt.is_null())
            .add(page_vote::Column::DisabledAt.is_null());

        if local_votes_only {
            condition = condition.add(page_vote::Column::FromWikidot.eq(false));
        }

        condition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_imported_rating_with_local_scores() {
        assert_eq!(
            ScoreService::combine_imported_rating(10, ScoreValue::Integer(5),).unwrap(),
            ScoreValue::Integer(15),
        );
        assert_eq!(
            ScoreService::combine_imported_rating(10, ScoreValue::Float(2.5),).unwrap(),
            ScoreValue::Float(12.5),
        );
    }

    #[test]
    fn rejects_imported_rating_integer_overflow() {
        let error =
            ScoreService::combine_imported_rating(i64::MAX, ScoreValue::Integer(1))
                .unwrap_err();

        assert!(error.to_string().contains("overflowed"));
    }
}
