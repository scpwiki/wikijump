/*
 * services/import/service.rs
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

//! Importer service, for ingesting data from Wikidot.
//!
//! This does not perform checks such as name / slug correspodnence,
//! uniqueness (this will get blocked by the database probably),
//! inconsistency, or perform filter validation.
//!
//! It is for limited use during initial setup only.

use super::structs::{
    ImportPage, ImportPageOutput, ImportPageRevision, ImportPageRevisionOutput,
    ImportSite, ImportSiteOutput, ImportUser, ImportUserOutput, ImportedUserType,
};
use crate::constants::SYSTEM_USER_ID;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::Model as PageCategoryModel;
use crate::models::page_revision::{self, Entity as PageRevision};
use crate::models::site::{self, Entity as Site};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::blob::{BlobService, FinalizeBlobUploadOutput};
use crate::services::page_lock::{CreatePageLockInput, PageLockService};
use crate::services::page_revision::{
    CreateFirstPageRevision, CreateFirstPageRevisionOutput, CreatePageRevision,
    CreatePageRevisionBody, PageRevisionService,
};
use crate::services::{CategoryService, UserService};
use crate::types::{Maybe, PageId, PageLockType, Reference};
use crate::utils::{get_category_name, now};
use ftml::layout::Layout;
use sea_orm::DatabaseTransaction;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, UpdateResult};
use sea_query::Expr;

const SYNC_PAGE_REVISION_SEQUENCE_SQL: &str = "SELECT setval('page_revision_revision_id_seq', GREATEST(COALESCE((SELECT MAX(revision_id) FROM page_revision), 1), CASE WHEN seq.is_called THEN seq.last_value ELSE 1 END), COALESCE((SELECT MAX(revision_id) FROM page_revision), 0) > 0 OR seq.is_called) FROM page_revision_revision_id_seq AS seq";

async fn sync_page_revision_sequence(txn: &DatabaseTransaction) -> Result<()> {
    txn.execute_raw(Statement::from_string(
        DatabaseBackend::Postgres,
        SYNC_PAGE_REVISION_SEQUENCE_SQL,
    ))
    .await
    .map(|_| ())
    .or_raise(|| {
        Error::new(
            "failed to synchronize the page revision ID sequence",
            ErrorType::DatabaseImport,
        )
    })
}

#[derive(Debug)]
pub struct ImportService;

impl ImportService {
    pub async fn add_user(
        ctx: &ServiceContext<'_>,
        ImportUser {
            user_id,
            created_at,
            fetched_at,
            wikidot_user_type,
            avatar_uploaded_blob_id,
            real_name,
            gender,
            birthday,
            location,
            biography,
            website,
            karma,
            is_pro,
            importing_user_id,
            ip_address,
        }: ImportUser,
    ) -> Result<ImportUserOutput> {
        info!(
            "Importing Wikidot user (user ID {}, created {}, karma {})",
            user_id,
            created_at,
            karma.value(),
        );

        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!("failed to import wikidot user (user ID {user_id})"),
                ErrorType::DatabaseImport,
            )
        };

        if user_id <= 0 {
            return Err(make_error().into());
        }

        let (
            is_deleted,
            name,
            slug,
            avatar_uploaded_blob_id,
            real_name,
            gender,
            birthday,
            location,
            biography,
            website,
            karma,
            is_pro,
        ) = match wikidot_user_type {
            ImportedUserType::Extant { name, slug } => (
                false,
                Some(name),
                Some(slug),
                avatar_uploaded_blob_id,
                real_name,
                gender,
                birthday,
                location,
                biography,
                website,
                karma,
                is_pro,
            ),
            // A deleted source account is a tombstone. Discard its public
            // profile and avatar fields so stale/private data cannot survive
            // a deletion refresh.
            ImportedUserType::Deleted => (
                true,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                ftml::data::KarmaLevel::Zero,
                false,
            ),
        };

        let existing = WikidotUser::find_by_id(user_id)
            .one(txn)
            .await
            .or_raise(make_error)?;

        if let Some(existing) = existing.as_ref() {
            if existing.created_at != created_at {
                return Err(make_error().into());
            }

            let unchanged = existing.fetched_at == fetched_at
                && existing.is_deleted == is_deleted
                && existing.name == name
                && existing.slug == slug
                && existing.real_name == real_name
                && existing.gender == gender
                && existing.birthday == birthday
                && existing.location == location
                && existing.biography == biography
                && existing.website == website
                && existing.karma == i16::from(karma.value())
                && existing.is_pro == is_pro
                && (avatar_uploaded_blob_id.is_none() || is_deleted)
                && (!is_deleted || existing.avatar_s3_hash.is_none());

            if unchanged {
                return Ok(ImportUserOutput { user_id });
            }

            // Stable source ID is authoritative. Accept a changed snapshot
            // only when it is strictly newer; same-time contradictions and
            // stale replays are not identity evidence.
            if fetched_at <= existing.fetched_at {
                return Err(make_error().into());
            }
        }

        if !is_deleted {
            let conflicting_identity = WikidotUser::find()
                .filter(wikidot_user::Column::UserId.ne(user_id))
                .filter(
                    Condition::any()
                        .add(wikidot_user::Column::Name.eq(name.as_deref().unwrap()))
                        .add(wikidot_user::Column::Slug.eq(slug.as_deref().unwrap())),
                )
                .one(txn)
                .await
                .or_raise(make_error)?;
            if conflicting_identity.is_some() {
                return Err(make_error().into());
            }
        }

        let uploaded_avatar_s3_hash = match avatar_uploaded_blob_id {
            None => None,
            Some(uploaded_blob_id) => {
                let FinalizeBlobUploadOutput { s3_hash, .. } =
                    BlobService::finish_unscoped_upload(
                        ctx,
                        importing_user_id,
                        &uploaded_blob_id,
                    )
                    .await
                    .or_raise(make_error)?;

                // We don't check the avatar size, just keep whatever it was for Wikidot
                // which will have a limited size anyways, so it's probably fine.

                Some(s3_hash.to_vec())
            }
        };

        // Add to audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::ImportUser {
                user_id,
                user_slug: slug.as_deref(),
                user_name: name.as_deref(),
            },
        )
        .await
        .or_raise(make_error)?;

        // Add known user ID
        UserService::insert_known_user_id(ctx, i64::from(user_id))
            .await
            .or_raise(make_error)?;

        // Now add the actual Wikidot record itself
        let avatar_s3_hash = if is_deleted {
            None
        } else {
            uploaded_avatar_s3_hash.or_else(|| {
                existing
                    .as_ref()
                    .and_then(|user| user.avatar_s3_hash.clone())
            })
        };

        if let Some(existing) = existing {
            let mut model = existing.into_active_model();
            model.fetched_at = Set(fetched_at);
            model.is_deleted = Set(is_deleted);
            model.name = Set(name);
            model.slug = Set(slug);
            model.avatar_s3_hash = Set(avatar_s3_hash);
            model.real_name = Set(real_name);
            model.gender = Set(gender);
            model.birthday = Set(birthday);
            model.location = Set(location);
            model.biography = Set(biography);
            model.website = Set(website);
            model.karma = Set(i16::from(karma.value()));
            model.is_pro = Set(is_pro);
            model.update(txn).await.or_raise(make_error)?;
        } else {
            let model = wikidot_user::ActiveModel {
                user_id: Set(user_id),
                created_at: Set(created_at),
                fetched_at: Set(fetched_at),
                is_deleted: Set(is_deleted),
                name: Set(name),
                slug: Set(slug),
                avatar_s3_hash: Set(avatar_s3_hash),
                real_name: Set(real_name),
                gender: Set(gender),
                birthday: Set(birthday),
                location: Set(location),
                biography: Set(biography),
                website: Set(website),
                karma: Set(i16::from(karma.value())),
                is_pro: Set(is_pro),
            };
            WikidotUser::insert(model)
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        Ok(ImportUserOutput { user_id })
    }

    pub async fn add_site(
        ctx: &ServiceContext<'_>,
        ImportSite {
            site_id,
            created_at,
            name,
            slug,
            locale,
            ip_address,
        }: ImportSite,
    ) -> Result<ImportSiteOutput> {
        info!("Importing site (name '{name}', slug '{slug}', locale '{locale}')");

        let make_error = || {
            Error::new(
                format!(
                    "failed to import site (name '{}', slug '{}', ID {})",
                    name, slug, site_id,
                ),
                ErrorType::DatabaseImport,
            )
        };

        // Insert site row
        let txn = ctx.transaction();
        let site = site::ActiveModel {
            site_id: Set(site_id),
            created_at: Set(created_at),
            from_wikidot: Set(true),
            name: Set(name.clone()),
            slug: Set(slug.clone()),
            locale: Set(locale),
            ..Default::default()
        };
        Site::insert(site).exec(txn).await.or_raise(make_error)?;

        // Add to audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::ImportSite {
                site_id,
                site_slug: &slug,
                site_name: &name,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(ImportSiteOutput { site_id })
    }

    pub async fn add_page(
        ctx: &ServiceContext<'_>,
        ImportPage {
            page_id,
            site_id,
            created_at,
            slug,
            locked,
            discussion_thread_id,
            ip_address,
        }: ImportPage,
    ) -> Result<ImportPageOutput> {
        info!("Creating page '{slug}' in site ID {site_id}");

        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to import page (slug '{}', ID {} in site ID {})",
                    slug, page_id, site_id,
                ),
                ErrorType::DatabaseImport,
            )
        };

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await
                .or_raise(make_error)?;

        // Insert page row into table
        let page = page::ActiveModel {
            page_id: Set(page_id),
            site_id: Set(site_id),
            created_at: Set(created_at),
            from_wikidot: Set(true),
            slug: Set(slug.clone()),
            page_category_id: Set(category_id),
            discussion_thread_id: Set(discussion_thread_id),
            ..Default::default()
        };
        Page::insert(page).exec(txn).await.or_raise(make_error)?;

        // If locked, add that too
        if locked {
            PageLockService::create(
                ctx,
                site_id,
                SYSTEM_USER_ID,
                Reference::Id(page_id),
                CreatePageLockInput {
                    page: Reference::Id(page_id),
                    from_wikidot: true,
                    lock_type: PageLockType::Wikidot,
                    reason: None,
                    expires_at: None,
                    override_existing: false,
                    ip_address,
                },
            )
            .await
            .or_raise(make_error)?;
        }

        // Add to audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::ImportPage {
                site_id,
                page_id,
                page_slug: &slug,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(ImportPageOutput { site_id, page_id })
    }

    pub async fn add_page_revision(
        ctx: &ServiceContext<'_>,
        ImportPageRevision {
            revision_id,
            revision_type,
            created_at,
            updated_at,
            revision_number,
            page_id,
            site_id,
            user_id,
            wikitext,
            comments,
            title,
            slug,
            tags,
        }: ImportPageRevision,
    ) -> Result<ImportPageRevisionOutput> {
        info!(
            "Creating page revision ID {} (number {}) on page ID {} on site ID {}",
            revision_id, revision_number, page_id, site_id,
        );

        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to import page revision ID {} (number {}) on page ID {} in site ID {}",
                    revision_id, revision_number, page_id, site_id,
                ),
                ErrorType::DatabaseImport,
            )
        };

        // Get page category
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await
                .or_raise(make_error)?;

        // Get prior revision
        //
        // Import operations don't require an initial page revision on page import,
        // so it's possible that this is None.
        //
        // Then we check that the revision_number being inserted is one more than the
        // prior revision (or 0 if None, i.e. this is the first).
        let prev_revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id)),
            )
            .order_by_desc(page_revision::Column::RevisionNumber)
            .one(txn)
            .await
            .or_raise(make_error)?;

        let mut first_followups = None;
        let mut persisted_revision = None;
        let output_revision_id = match prev_revision {
            // No prior revisions
            // First revision for the page
            None => {
                if revision_number != 0 {
                    bail!(Error::new(
                        format!(
                            "failed to import page revision ID {} (number {}), because there are no prior revisions (should've been 0)",
                            revision_id, revision_number,
                        ),
                        ErrorType::DatabaseImport,
                    ));
                }

                let CreateFirstPageRevisionOutput {
                    revision_id: output_revision_id,
                    parser_errors: _,
                    followups,
                } = PageRevisionService::create_first(
                    ctx,
                    PageId {
                        site_id,
                        category_id,
                        page_id,
                    },
                    CreateFirstPageRevision {
                        user_id,
                        comments,
                        wikitext,
                        title,
                        alt_title: None,
                        tags: tags.clone(),
                        slug,
                        layout: Some(Layout::Wikidot),
                    },
                )
                .await
                .or_raise(make_error)?;

                first_followups = Some(followups);
                output_revision_id
            }

            // Prior revisions
            // Second revision or later
            Some(prev_revision) => {
                if revision_number != prev_revision.revision_number + 1 {
                    bail!(Error::new(
                        format!(
                            "failed to import page revision ID {} (number {}), because the prior revision was {} (should've been {})",
                            revision_id,
                            revision_number,
                            prev_revision.revision_number,
                            prev_revision.revision_number + 1,
                        ),
                        ErrorType::DatabaseImport
                    ));
                }

                let output = PageRevisionService::create(
                    ctx,
                    PageId {
                        site_id,
                        category_id,
                        page_id,
                    },
                    CreatePageRevision {
                        user_id,
                        comments,
                        revision_type,
                        body: CreatePageRevisionBody {
                            wikitext: Maybe::Set(wikitext),
                            title: Maybe::Set(title),
                            alt_title: Maybe::Unset,
                            slug: Maybe::Set(slug),
                            // NOTE: We set tags here so the "changes" value
                            //       is correct for this revision.
                            tags: Maybe::Set(tags.clone()),
                            parent_changed: false,
                        },
                    },
                    prev_revision,
                )
                .await
                .or_raise(make_error)?;

                let output = match output {
                    // There should always be an output, if not, raise an error
                    Some(output) => output,
                    None => bail!(Error::new(
                        format!(
                            "failed to import page revision ID {} (number {}), because no data was changed from the last revision",
                            revision_id, revision_number,
                        ),
                        ErrorType::DatabaseImport
                    )),
                };
                let output_revision_number = output.output.revision_number;
                let output_revision_id = output.output.revision_id;
                assert_eq!(
                    revision_number, output_revision_number,
                    "Newly-created revision has a revision number of {}, which is not {} as expected",
                    output_revision_number, revision_number,
                );
                persisted_revision = Some(output);
                output_revision_id
            }
        };

        // Update database fields that we can't set via PageRevisionService
        //
        // We need to use update_many since this changes the primary key,
        // but only one row should be modified by this.

        let UpdateResult { rows_affected, .. } = PageRevision::update_many()
            .col_expr(page_revision::Column::RevisionId, Expr::value(revision_id))
            .col_expr(page_revision::Column::CreatedAt, Expr::value(created_at))
            .col_expr(page_revision::Column::UpdatedAt, Expr::value(updated_at))
            // not settable on page create
            .col_expr(page_revision::Column::Tags, Expr::value(tags))
            .filter(page_revision::Column::RevisionId.eq(output_revision_id))
            .exec(txn)
            .await
            .or_raise(make_error)?;

        assert_eq!(
            rows_affected, 1,
            "More than one row updated in page_revision after revision creation",
        );

        // Imported IDs replace generated BIGSERIAL values; keep the next normal insert above them.
        sync_page_revision_sequence(txn)
            .await
            .or_raise(make_error)?;

        let model = page::ActiveModel {
            page_id: Set(page_id),
            latest_revision_id: Set(Some(revision_id)),
            updated_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await.or_raise(make_error)?;

        let id = PageId {
            site_id,
            category_id,
            page_id,
        };
        if let Some(followups) = first_followups {
            PageRevisionService::apply_first_followups(ctx, id, followups)
                .await
                .or_raise(make_error)?;
        }
        if let Some(revision) = persisted_revision {
            PageRevisionService::apply_followups(ctx, id, revision)
                .await
                .or_raise(make_error)?;
        }

        // Return output

        Ok(ImportPageRevisionOutput {
            site_id,
            page_id,
            page_revision_id: revision_id,
            page_revision_number: revision_number,
        })
    }

    // TODO page_vote

    // TODO file
    // TODO forum
}

#[cfg(test)]
mod tests {
    use super::SYNC_PAGE_REVISION_SEQUENCE_SQL;

    #[test]
    fn revision_sequence_sync_sql_uses_imported_max_and_current_sequence_state() {
        assert!(
            SYNC_PAGE_REVISION_SEQUENCE_SQL
                .contains("setval('page_revision_revision_id_seq'")
        );
        assert!(SYNC_PAGE_REVISION_SEQUENCE_SQL.contains("MAX(revision_id)"));
        assert!(SYNC_PAGE_REVISION_SEQUENCE_SQL.contains("seq.last_value"));
        assert!(SYNC_PAGE_REVISION_SEQUENCE_SQL.contains("seq.is_called"));
    }
}
