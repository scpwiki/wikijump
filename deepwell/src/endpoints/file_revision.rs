/*
 * endpoints/file_revision.rs
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

use super::file::ensure_parent_page_view_permission;
use super::prelude::*;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::MutationAuthorization;
use crate::services::file::GetFile;
use crate::services::file_revision::{
    CountFileRevisions, FileRevisionCountOutput, FileRevisionModelFiltered,
    GetFileRevision, GetFileRevisionRange, GetPageFileRevisionHistory,
    PageFileRevisionHistoryModelFiltered, PageFileRevisionHistoryOutput,
    UpdateFileRevision,
};
use crate::types::{Action, Permission, Reference, Resource};
use ftml::data::UserInfo;
use std::collections::{BTreeMap, BTreeSet};

pub async fn file_revision_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionCountOutput> {
    let input: GetFile<'_> = parse!(params, FileRevision);
    let site_id = input.site_id;
    let page_id = input.page_id;

    let make_error = || {
        Error::new(
            "failed to get count of file revisions",
            ErrorType::FileRevision,
        )
    };

    ensure_parent_page_view_permission(ctx, site_id, page_id, None)
        .await
        .or_raise(make_error)?;

    let file_id = FileService::get_id(ctx, input).await.or_raise(make_error)?;

    let revision_count = FileRevisionService::count(
        ctx,
        CountFileRevisions {
            site_id,
            page_id,
            file_id,
        },
    )
    .await
    .or_raise(make_error)?;

    Ok(FileRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    })
}

pub async fn file_revision_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<FileRevisionModelFiltered>> {
    let input: GetFileRevision = parse!(params, FileRevision);
    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id, None)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check file revision parent-page visibility",
                ErrorType::FileRevision,
            )
        })?;

    let revision = FileRevisionService::get_optional(ctx, input)
        .await
        .or_raise(|| {
            Error::new("failed to get file revision", ErrorType::FileRevision)
        })?;

    revision.map(filter_file_revision).transpose()
}

pub async fn file_revision_range(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageFileRevisionHistoryModelFiltered>> {
    let input: GetFileRevisionRange = parse!(params, FileRevision);
    let make_error = || {
        Error::new(
            "failed to get range of file revisions",
            ErrorType::FileRevision,
        )
    };
    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id, None)
        .await
        .or_raise(make_error)?;

    let revisions = FileRevisionService::get_range(ctx, input)
        .await
        .or_raise(make_error)?;

    // Like the page timeline's file history, each revision carries the
    // author's public identity (issue #2199), so the Files pane never has
    // to render raw user IDs.
    let authors = resolve_file_revision_authors(ctx, &revisions)
        .await
        .or_raise(make_error)?;

    revisions
        .into_iter()
        .map(|revision| attach_file_revision_author(revision, &authors))
        .collect()
}

async fn resolve_file_revision_authors(
    ctx: &ServiceContext<'_>,
    revisions: &[FileRevisionModel],
) -> Result<BTreeMap<i64, UserInfo<'static>>> {
    let user_ids = revisions
        .iter()
        .map(|revision| revision.user_id)
        .collect::<BTreeSet<_>>();
    let mut authors = BTreeMap::new();
    for user_id in user_ids {
        let Some(user) = UserService::get_optional(ctx, Reference::Id(user_id)).await?
        else {
            continue;
        };
        if let Some(author) = user.into_public_identity() {
            authors.insert(user_id, author);
        }
    }
    Ok(authors)
}

fn attach_file_revision_author(
    revision: FileRevisionModel,
    authors: &BTreeMap<i64, UserInfo<'static>>,
) -> Result<PageFileRevisionHistoryModelFiltered> {
    let author = authors.get(&revision.user_id).cloned();
    Ok(PageFileRevisionHistoryModelFiltered {
        revision: filter_file_revision(revision)?,
        author,
    })
}

pub async fn file_revision_page_history(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageFileRevisionHistoryOutput> {
    let input: GetPageFileRevisionHistory = parse!(params, FileRevision);
    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id, None)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check file history parent-page visibility",
                ErrorType::FileRevision,
            )
        })?;

    let history = FileRevisionService::get_page_history(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to get file revision history for page",
                ErrorType::FileRevision,
            )
        })?;

    let authors = resolve_file_revision_authors(ctx, &history.revisions)
        .await
        .or_raise(|| {
            Error::new(
                "failed to get file revision history for page",
                ErrorType::FileRevision,
            )
        })?;

    let revisions = history
        .revisions
        .into_iter()
        .map(|revision| attach_file_revision_author(revision, &authors))
        .collect::<Result<Vec<_>>>()?;

    Ok(PageFileRevisionHistoryOutput {
        revision_count: history.revision_count,
        revisions,
    })
}

pub async fn file_revision_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionModel> {
    let input: UpdateFileRevision = parse!(params, FileRevision);
    MutationAuthorization::require_matching_actor(
        ctx,
        input.user_id,
        "edit file revision visibility",
    )?;
    MutationAuthorization::require_permission(
        ctx,
        input.site_id,
        Some(Reference::Id(input.page_id)),
        Permission {
            resource_type: Resource::Page,
            resource_category: None,
            action: Action::Edit,
        },
        "edit file revision visibility",
    )
    .await?;

    info!(
        "Editing file revision ID {} for file ID {} on page {}",
        input.revision_id, input.file_id, input.page_id,
    );

    FileRevisionService::update(ctx, input)
        .await
        .or_raise(|| Error::new("failed to edit file revision", ErrorType::FileRevision))
}

fn filter_file_revision(model: FileRevisionModel) -> Result<FileRevisionModelFiltered> {
    let FileRevisionModel {
        revision_id,
        revision_type,
        created_at,
        revision_number,
        file_id,
        page_id,
        site_id,
        user_id,
        name,
        s3_hash,
        mime,
        size,
        content_type_label: _,
        content_type_description: _,
        changes,
        comments,
        hidden,
    } = model;

    let mut name = Some(name);
    let mut s3_hash = Some(s3_hash);
    let mut mime = Some(mime);
    let mut size = Some(size);
    let mut comments = Some(comments);

    for field in &hidden {
        match field.as_str() {
            "name" => name = None,
            "s3_hash" => s3_hash = None,
            "mime" => mime = None,
            "size" => size = None,
            "comments" => comments = None,
            _ => bail!(Error::new(
                format!("unknown hidden file revision field: {field}"),
                ErrorType::FileRevision,
            )),
        }
    }

    Ok(FileRevisionModelFiltered {
        revision_id,
        revision_type,
        created_at,
        revision_number,
        file_id,
        page_id,
        site_id,
        user_id,
        name,
        s3_hash,
        mime,
        size,
        changes,
        comments,
        hidden,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileRevisionType;
    use serde_json::Value;
    use time::OffsetDateTime;

    fn revision(hidden: Vec<String>) -> FileRevisionModel {
        FileRevisionModel {
            revision_id: 17,
            revision_type: FileRevisionType::Regular,
            created_at: OffsetDateTime::UNIX_EPOCH,
            revision_number: 3,
            file_id: 23,
            page_id: 29,
            site_id: 31,
            user_id: 37,
            name: "moderated-name.txt".to_owned(),
            s3_hash: vec![41; 64],
            mime: "text/plain".to_owned(),
            size: 43,
            content_type_label: None,
            content_type_description: None,
            changes: vec!["name".to_owned()],
            comments: "moderated comments".to_owned(),
            hidden,
        }
    }

    #[test]
    fn filters_hidden_file_revision_fields_before_serialization() {
        let hidden = ["name", "s3_hash", "mime", "size", "comments"]
            .map(str::to_owned)
            .to_vec();
        let filtered = filter_file_revision(revision(hidden.clone()))
            .expect("known hidden fields should be filtered");

        assert_eq!(filtered.revision_id, 17);
        assert_eq!(filtered.revision_number, 3);
        assert_eq!(filtered.hidden, hidden);
        assert!(filtered.name.is_none());
        assert!(filtered.s3_hash.is_none());
        assert!(filtered.mime.is_none());
        assert!(filtered.size.is_none());
        assert!(filtered.comments.is_none());

        let serialized =
            serde_json::to_value(filtered).expect("filtered revision should serialize");
        for field in ["name", "s3_hash", "mime", "size", "comments"] {
            assert_eq!(serialized[field], Value::Null, "field {field} must be null");
        }
    }

    #[test]
    fn preserves_visible_file_revision_fields() {
        let filtered = filter_file_revision(revision(Vec::new()))
            .expect("visible revision should be returned");

        assert_eq!(filtered.name.as_deref(), Some("moderated-name.txt"));
        assert_eq!(filtered.s3_hash, Some(vec![41; 64]));
        assert_eq!(filtered.mime.as_deref(), Some("text/plain"));
        assert_eq!(filtered.size, Some(43));
        assert_eq!(filtered.comments.as_deref(), Some("moderated comments"));
    }

    #[test]
    fn rejects_unknown_hidden_file_revision_fields() {
        let result = filter_file_revision(revision(vec!["revision_id".to_owned()]));

        assert!(result.is_err(), "unknown hidden fields must fail closed");
    }

    #[test]
    fn attaches_author_by_user_id_not_position() {
        let authors = BTreeMap::from([(37, UserInfo::dummy())]);
        let attached = attach_file_revision_author(revision(Vec::new()), &authors)
            .expect("known revision should attach");

        assert_eq!(attached.revision.file_id, 23);
        let author = attached
            .author
            .expect("matching user ID should attach its author");
        assert_eq!(author, UserInfo::dummy());

        let others = BTreeMap::from([(999, UserInfo::dummy())]);
        let unattached = attach_file_revision_author(revision(Vec::new()), &others)
            .expect("known revision should attach");
        assert!(
            unattached.author.is_none(),
            "an unrelated user ID must not attach",
        );
    }

    #[test]
    fn leaves_author_empty_without_a_public_identity() {
        let attached =
            attach_file_revision_author(revision(Vec::new()), &BTreeMap::new())
                .expect("known revision should attach");

        // Unknown, deleted, and synthetic import actors (e.g. user -1/-2)
        // resolve to no public identity, so the pane must fall back to its
        // deleted-user marker instead of a raw numeric ID (issue #2199).
        assert!(attached.author.is_none());
        assert_eq!(attached.revision.user_id, 37);
    }

    #[test]
    fn serializes_author_flat_alongside_revision_fields() {
        let authors = BTreeMap::from([(37, UserInfo::dummy())]);
        let attached = attach_file_revision_author(revision(Vec::new()), &authors)
            .expect("known revision should attach");

        let serialized =
            serde_json::to_value(attached).expect("attached revision should serialize");
        assert_eq!(serialized["file_id"], 23);
        assert!(serialized.get("author").is_some());
        assert!(serialized.get("revision").is_none());
    }
}
