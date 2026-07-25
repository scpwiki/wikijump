use super::settings::{RenderReplaySettings, states_sql};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::runtime::ServerState;
use crate::services::context::ServiceContext;
use crate::services::render::{CorpusReplayExpandedWikitext, RenderService};
use crate::services::{
    PageRevisionService, ScoreService, SettingsService, SiteService, TextService,
};
use crate::types::{PageId, Reference};
use crate::utils::{locale_for_ftml, split_category};
use ftml::data::PageInfo;
use ftml::settings::{WikitextMode, WikitextSettings};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait, Value};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(super) struct ReplayCandidate {
    pub source_fullname: String,
    pub page_id: i64,
    pub site_id: i64,
    pub category_id: i64,
    pub state: String,
}

pub(super) async fn expand_candidate(
    state: &ServerState,
    candidate: &ReplayCandidate,
) -> Result<CorpusReplayExpandedWikitext> {
    let make_error = || {
        Error::new(
            format!(
                "failed to prepare replay page {}",
                candidate.source_fullname
            ),
            ErrorType::Render,
        )
    };
    let transaction = state.database.begin().await.or_raise(make_error)?;
    transaction
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "SET TRANSACTION READ ONLY",
        ))
        .await
        .or_raise(make_error)?;
    let result = async {
        let ctx = ServiceContext::new(state, &transaction);
        let revision =
            PageRevisionService::get_latest(&ctx, candidate.site_id, candidate.page_id)
                .await
                .or_raise(make_error)?;
        let wikitext = TextService::get(&ctx, &revision.wikitext_hash)
            .await
            .or_raise(make_error)?;
        let score = ScoreService::score(&ctx, candidate.page_id)
            .await
            .or_raise(make_error)?;
        let layout =
            SettingsService::get_layout(&ctx, candidate.site_id, Some(candidate.page_id))
                .await
                .or_raise(make_error)?;
        let site = SiteService::get(&ctx, Reference::Id(candidate.site_id))
            .await
            .or_raise(make_error)?;
        let (category, page) = split_category(&revision.slug);
        let page_info = PageInfo {
            page: Cow::Owned(page.to_owned()),
            category: category.map(|value| Cow::Owned(value.to_owned())),
            site: Cow::Owned(site.slug),
            title: Cow::Owned(revision.title),
            alt_title: revision.alt_title.map(Cow::Owned),
            score,
            tags: revision.tags.into_iter().map(Cow::Owned).collect(),
            language: Cow::Owned(locale_for_ftml(&site.locale).to_owned()),
        };
        let settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        RenderService::expand_corpus_replay_wikitext(
            &ctx,
            wikitext,
            page_info,
            settings,
            PageId {
                site_id: candidate.site_id,
                category_id: candidate.category_id,
                page_id: candidate.page_id,
            },
        )
        .await
        .or_raise(make_error)
    }
    .await;
    transaction.rollback().await.or_raise(make_error)?;
    result
}

pub(super) async fn select_import_run(
    state: &ServerState,
    settings: &RenderReplaySettings,
) -> Result<Option<i64>> {
    let make_error = || {
        Error::new(
            "failed to select render-replay import run",
            ErrorType::DatabaseQuery,
        )
    };
    if let Some(import_run_id) = settings.import_run_id {
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT 1 AS present FROM wikidot_corpus_import_run WHERE import_run_id = $1",
            [Value::from(import_run_id)],
        );
        if state
            .database
            .query_one(statement)
            .await
            .or_raise(make_error)?
            .is_none()
        {
            bail!(Error::new(
                format!("render-replay import run {import_run_id} does not exist"),
                ErrorType::DatabaseQuery,
            ));
        }
        return Ok(Some(import_run_id));
    }
    let state_sql = states_sql(&settings.states);
    let statement = Statement::from_string(
        DatabaseBackend::Postgres,
        format!(
            "SELECT run.import_run_id \
             FROM wikidot_corpus_import_run AS run \
             WHERE EXISTS (SELECT 1 FROM wikidot_corpus_import_item AS item \
                 WHERE item.import_run_id = run.import_run_id \
                 AND item.state IN ({state_sql})) \
             ORDER BY run.started_at DESC, run.import_run_id DESC LIMIT 1"
        ),
    );
    state
        .database
        .query_one(statement)
        .await
        .or_raise(make_error)?
        .map(|row| row.try_get("", "import_run_id").or_raise(make_error))
        .transpose()
}

pub(super) async fn list_candidates(
    state: &ServerState,
    import_run_id: i64,
    settings: &RenderReplaySettings,
) -> Result<Vec<ReplayCandidate>> {
    let sql = format!(
        "SELECT item.source_fullname, item.page_id, page.site_id, \
             page.page_category_id, item.state \
         FROM wikidot_corpus_import_item AS item \
         JOIN page ON page.page_id = item.page_id AND page.deleted_at IS NULL \
         WHERE item.import_run_id = $1 AND item.state IN ({}) \
         ORDER BY item.source_fullname ASC",
        states_sql(&settings.states),
    );
    let statement = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        [Value::from(import_run_id)],
    );
    let make_error = || {
        Error::new(
            "failed to list render-replay candidates",
            ErrorType::DatabaseQuery,
        )
    };
    state
        .database
        .query_all(statement)
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|row| {
            Ok(ReplayCandidate {
                source_fullname: row
                    .try_get("", "source_fullname")
                    .or_raise(make_error)?,
                page_id: row.try_get("", "page_id").or_raise(make_error)?,
                site_id: row.try_get("", "site_id").or_raise(make_error)?,
                category_id: row.try_get("", "page_category_id").or_raise(make_error)?,
                state: row.try_get("", "state").or_raise(make_error)?,
            })
        })
        .collect()
}
