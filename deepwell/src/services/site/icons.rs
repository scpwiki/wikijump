/*
 * services/site/icons.rs
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

//! Site-level icon byte storage (favicon, iOS touch icons, Windows tile).
//!
//! Icon sources (`favicon_source` and siblings) record the configured
//! location, but the compatibility routes (`/local--favicon/*`,
//! `/local--iosicon/*`, `/local--wp8icon/*`) must serve bytes without
//! depending on Wikidot at request time. Imported icon bytes live here, keyed
//! by site and kind, outside the page model (file rows require a page).

use super::service::{SiteIconSourceKind, validate_site_icon_source};
use super::structs::{GetSiteIconOutput, SetSiteIconOutput};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::site::{self, Model as SiteModel};
use crate::models::site_icon::{self, Entity as SiteIcon};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService, SiteFields};
use crate::types::{Maybe, Reference};
use crate::utils::now;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::net::IpAddr;

/// Largest accepted icon upload, in bytes. Icons are tiny page-head assets;
/// anything larger is not a site icon.
pub const MAX_SITE_ICON_BYTES: usize = 524288;

fn allowed_mime_types(kind: SiteIconSourceKind) -> &'static [&'static str] {
    match kind {
        // Wikidot declares the favicon route as image/gif regardless of the
        // uploaded bytes, so legacy mismatches are tolerated here as well.
        SiteIconSourceKind::Favicon => {
            &["image/gif", "image/png", "image/x-icon", "image/jpeg"]
        }
        // The derived iOS and Windows routes are PNGs on Wikidot.
        SiteIconSourceKind::Ios | SiteIconSourceKind::Windows => {
            &["image/png", "image/jpeg"]
        }
    }
}

fn magic_matches_mime(mime: &str, bytes: &[u8]) -> bool {
    match mime {
        "image/png" => {
            bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
        }
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/x-icon" => bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]),
        _ => false,
    }
}

/// Whether bytes are any recognized image, regardless of declared type.
///
/// Wikidot serves its favicon route as `image/gif` even when the underlying
/// bytes are a PNG (scp-wiki, 2026-10-12 capture), so the favicon slot
/// tolerates that legacy mismatch instead of refusing the import.
fn magic_matches_any_image(bytes: &[u8]) -> bool {
    ["image/png", "image/gif", "image/jpeg", "image/x-icon"]
        .iter()
        .any(|mime| magic_matches_mime(mime, bytes))
}

fn validate_site_icon_bytes(
    kind: SiteIconSourceKind,
    mime: &str,
    bytes: &[u8],
) -> Result<()> {
    let make_error = |message: String| Error::new(message, ErrorType::BadRequest);

    if !allowed_mime_types(kind).contains(&mime) {
        return Err(make_error(format!(
            "{} does not accept '{mime}' uploads",
            kind.as_str(),
        ))
        .into());
    }

    if bytes.is_empty() || bytes.len() > MAX_SITE_ICON_BYTES {
        return Err(make_error(format!(
            "{} must be between 1 and {MAX_SITE_ICON_BYTES} bytes",
            kind.as_str(),
        ))
        .into());
    }

    // The favicon slot keeps Wikidot's leniency: any image bytes are
    // accepted under any allowed declared type. The derived iOS and Windows
    // slots have no such evidence, so they stay strict.
    let magic_ok = match kind {
        SiteIconSourceKind::Favicon => magic_matches_any_image(bytes),
        SiteIconSourceKind::Ios | SiteIconSourceKind::Windows => {
            magic_matches_mime(mime, bytes)
        }
    };
    if !magic_ok {
        return Err(make_error(format!(
            "{} bytes do not match the declared '{mime}' type",
            kind.as_str(),
        ))
        .into());
    }

    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

impl super::service::SiteService {
    /// Stores (or clears) one site icon's configured source and/or bytes.
    ///
    /// Each of `source` and `data` is tri-state: absent leaves that half
    /// untouched, null clears it, and a value sets it. Setting bytes
    /// requires a MIME type. Stored bytes always require a configured
    /// source, so a refresh and a removal stay consistent in one call.
    #[allow(clippy::too_many_arguments)]
    pub async fn set_site_icon(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        kind: &str,
        source: Option<Option<String>>,
        mime: Option<String>,
        data: Option<Option<Vec<u8>>>,
        actor_user_id: i64,
        ip_address: IpAddr,
    ) -> Result<SetSiteIconOutput> {
        let txn = ctx.transaction();
        let kind = SiteIconSourceKind::parse(kind).ok_or_else(|| {
            Error::new(
                format!("unknown site icon kind '{kind}'"),
                ErrorType::BadRequest,
            )
        })?;
        let make_error = || Error::new("failed to set site icon", ErrorType::Site);

        let site = Self::get_for_update(ctx, reference)
            .await
            .or_raise(make_error)?;

        // Resolve the resulting source, validating any new value the same
        // way site settings updates do.
        let mut new_source = site_source(&site, kind);
        if let Some(ref validated) = source {
            validate_site_icon_source(&site.slug, site.from_wikidot, validated, kind)?;
            new_source = validated.clone();
        }

        // Resolve the resulting bytes.
        let existing = SiteIcon::find_by_id((site.site_id, kind.as_str().to_string()))
            .one(txn)
            .await
            .or_raise(make_error)?;
        enum BytesUpdate {
            Leave,
            Clear,
            Set { mime: String, bytes: Vec<u8> },
        }
        let bytes_update = match data {
            None => BytesUpdate::Leave,
            Some(None) => BytesUpdate::Clear,
            Some(Some(bytes)) => {
                let mime = mime.ok_or_else(|| {
                    Error::new(
                        "site icon bytes require a MIME type",
                        ErrorType::BadRequest,
                    )
                })?;
                validate_site_icon_bytes(kind, &mime, &bytes)?;
                BytesUpdate::Set { mime, bytes }
            }
        };

        let bytes_changed = !matches!(bytes_update, BytesUpdate::Leave);
        let source_changed = source.is_some();
        if !bytes_changed && !source_changed {
            return Err(Error::new(
                "site icon update changes neither the source nor the bytes",
                ErrorType::BadRequest,
            )
            .into());
        }

        // Stored bytes without a configured source would serve from routes
        // the page head never declares, so refuse the inconsistent state
        // before any write keeps rejected calls side-effect free.
        let bytes_present = match &bytes_update {
            BytesUpdate::Leave => existing.is_some(),
            BytesUpdate::Clear => false,
            BytesUpdate::Set { .. } => true,
        };
        if bytes_present && new_source.is_none() {
            return Err(Error::new(
                "site icon bytes require a configured icon source",
                ErrorType::BadRequest,
            )
            .into());
        }

        match &bytes_update {
            BytesUpdate::Leave => {}
            BytesUpdate::Clear => {
                SiteIcon::delete_by_id((site.site_id, kind.as_str().to_string()))
                    .exec(txn)
                    .await
                    .or_raise(make_error)?;
            }
            BytesUpdate::Set { mime, bytes } => {
                let model = site_icon::ActiveModel {
                    site_id: Set(site.site_id),
                    kind: Set(kind.as_str().to_string()),
                    mime: Set(mime.clone()),
                    bytes: Set(bytes.clone()),
                    sha256_hex: Set(sha256_hex(bytes)),
                    updated_at: Set(now()),
                };
                SiteIcon::insert(model)
                    .on_conflict(
                        OnConflict::columns([
                            site_icon::Column::SiteId,
                            site_icon::Column::Kind,
                        ])
                        .update_columns([
                            site_icon::Column::Mime,
                            site_icon::Column::Bytes,
                            site_icon::Column::Sha256Hex,
                            site_icon::Column::UpdatedAt,
                        ])
                        .to_owned(),
                    )
                    .exec_without_returning(txn)
                    .await
                    .or_raise(make_error)?;
            }
        }

        if source_changed {
            // Audit before writing, mirroring site settings updates.
            let mut previous_fields = SiteFields::default();
            let mut changed_fields = SiteFields::default();
            match kind {
                SiteIconSourceKind::Favicon => {
                    previous_fields.favicon_source =
                        Maybe::Set(site.favicon_source.as_deref());
                    changed_fields.favicon_source = Maybe::Set(new_source.as_deref());
                }
                SiteIconSourceKind::Ios => {
                    previous_fields.ios_icon_source =
                        Maybe::Set(site.ios_icon_source.as_deref());
                    changed_fields.ios_icon_source = Maybe::Set(new_source.as_deref());
                }
                SiteIconSourceKind::Windows => {
                    previous_fields.windows_tile_source =
                        Maybe::Set(site.windows_tile_source.as_deref());
                    changed_fields.windows_tile_source =
                        Maybe::Set(new_source.as_deref());
                }
            }
            AuditService::log(
                ctx,
                ip_address,
                AuditEvent::SiteUpdate {
                    site_id: site.site_id,
                    user_id: actor_user_id,
                    previous_fields,
                    changed_fields,
                },
            )
            .await?;

            let mut model = site::ActiveModel {
                site_id: Set(site.site_id),
                ..Default::default()
            };
            match kind {
                SiteIconSourceKind::Favicon => {
                    model.favicon_source = Set(new_source.clone());
                }
                SiteIconSourceKind::Ios => {
                    model.ios_icon_source = Set(new_source.clone());
                }
                SiteIconSourceKind::Windows => {
                    model.windows_tile_source = Set(new_source.clone());
                }
            }
            model.updated_at = Set(Some(now()));
            model.settings_revision = Set(site.settings_revision + 1);
            model.update(txn).await.or_raise(make_error)?;
        } else if bytes_changed {
            // Bytes-only refreshes still invalidate cached site output.
            let model = site::ActiveModel {
                site_id: Set(site.site_id),
                updated_at: Set(Some(now())),
                settings_revision: Set(site.settings_revision + 1),
                ..Default::default()
            };
            model.update(txn).await.or_raise(make_error)?;
        }

        ctx.defer_public_content_cache_invalidate_site(site.site_id)
            .or_raise(make_error)?;

        let row = SiteIcon::find_by_id((site.site_id, kind.as_str().to_string()))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(SetSiteIconOutput {
            site_id: site.site_id,
            kind: kind.as_str().to_string(),
            source: new_source,
            mime: row.as_ref().map(|row| row.mime.clone()),
            sha256_hex: row.map(|row| row.sha256_hex),
        })
    }

    /// Fetches one site icon's stored bytes, if any.
    pub async fn get_site_icon(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        kind: &str,
    ) -> Result<Option<GetSiteIconOutput>> {
        let txn = ctx.transaction();
        let kind = SiteIconSourceKind::parse(kind).ok_or_else(|| {
            Error::new(
                format!("unknown site icon kind '{kind}'"),
                ErrorType::BadRequest,
            )
        })?;
        let make_error = || Error::new("failed to get site icon", ErrorType::Site);

        let site_id = Self::get_id(ctx, reference).await.or_raise(make_error)?;
        let row = SiteIcon::find_by_id((site_id, kind.as_str().to_string()))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(row.map(|row| GetSiteIconOutput {
            site_id,
            kind: row.kind,
            mime: row.mime,
            sha256_hex: row.sha256_hex,
            data: row.bytes,
        }))
    }
}

fn site_source(site: &SiteModel, kind: SiteIconSourceKind) -> Option<String> {
    match kind {
        SiteIconSourceKind::Favicon => site.favicon_source.clone(),
        SiteIconSourceKind::Ios => site.ios_icon_source.clone(),
        SiteIconSourceKind::Windows => site.windows_tile_source.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::service::SiteIconSourceKind;
    use super::allowed_mime_types;
    use super::{magic_matches_mime, validate_site_icon_bytes};

    #[test]
    fn icon_kinds_parse_exactly() {
        assert_eq!(
            SiteIconSourceKind::parse("favicon"),
            Some(SiteIconSourceKind::Favicon),
        );
        assert_eq!(
            SiteIconSourceKind::parse("ios"),
            Some(SiteIconSourceKind::Ios),
        );
        assert_eq!(
            SiteIconSourceKind::parse("windows"),
            Some(SiteIconSourceKind::Windows),
        );
        for unknown in ["", "Favicon", "favicon.gif", "touch", "wp8icon", "icon"] {
            assert_eq!(SiteIconSourceKind::parse(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn mime_allowlist_is_tight_per_kind() {
        assert!(allowed_mime_types(SiteIconSourceKind::Favicon).contains(&"image/gif"));
        assert!(
            allowed_mime_types(SiteIconSourceKind::Favicon).contains(&"image/x-icon")
        );
        assert!(!allowed_mime_types(SiteIconSourceKind::Ios).contains(&"image/gif"));
        assert!(!allowed_mime_types(SiteIconSourceKind::Ios).contains(&"image/x-icon"));
        assert!(!allowed_mime_types(SiteIconSourceKind::Windows).contains(&"image/gif"));
        for kind in [
            SiteIconSourceKind::Favicon,
            SiteIconSourceKind::Ios,
            SiteIconSourceKind::Windows,
        ] {
            assert!(!allowed_mime_types(kind).contains(&"image/svg+xml"));
            assert!(!allowed_mime_types(kind).contains(&"text/html"));
        }
    }

    #[test]
    fn magic_must_match_declared_mime() {
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(magic_matches_mime("image/png", &png));
        assert!(!magic_matches_mime("image/gif", &png));
        assert!(!magic_matches_mime("image/png", b"GIF89a..."));
        assert!(magic_matches_mime("image/gif", b"GIF89a..."));
        assert!(magic_matches_mime("image/jpeg", &[0xFF, 0xD8, 0xFF, 0x00]));
        assert!(magic_matches_mime(
            "image/x-icon",
            &[0x00, 0x00, 0x01, 0x00, 0x01]
        ));
    }

    #[test]
    fn mismatched_or_empty_bytes_are_rejected() {
        let png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(
            validate_site_icon_bytes(SiteIconSourceKind::Ios, "image/png", &png).is_ok(),
        );
        assert!(
            validate_site_icon_bytes(SiteIconSourceKind::Ios, "image/gif", &png).is_err(),
        );
        // The favicon slot tolerates Wikidot's legacy gif-declared PNG bytes.
        assert!(
            validate_site_icon_bytes(SiteIconSourceKind::Favicon, "image/gif", &png)
                .is_ok(),
        );
        // Non-image bytes are still rejected everywhere.
        assert!(
            validate_site_icon_bytes(SiteIconSourceKind::Favicon, "image/gif", b"nope")
                .is_err(),
        );
        assert!(
            validate_site_icon_bytes(SiteIconSourceKind::Favicon, "image/png", &[])
                .is_err(),
        );
        assert!(
            validate_site_icon_bytes(
                SiteIconSourceKind::Favicon,
                "image/png",
                &vec![0u8; super::MAX_SITE_ICON_BYTES + 1],
            )
            .is_err(),
        );
    }
}
