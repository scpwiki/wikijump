/*
 * services/audit/structs.rs
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

use super::prelude::*;
use crate::license::License;
use crate::types::id::{PageId, SiteId, UserId, PageCategoryId, PageRevisionId};
use ftml::layout::Layout;
use std::borrow::Cow;
use std::net::IpAddr;
use time::Date;

// Main structs

/// An event on the audit log.
///
/// Each type of event has a different set of fields that it provides
#[derive(Debug, Clone)]
pub enum AuditEvent<'a> {
    UserCreate {
        user_id: UserId,
    },
    UserUpdate {
        user_id: UserId,
        previous_fields: UserFields<'a>,
        changed_fields: UserFields<'a>,
    },
    UserUpdateMfa {
        user_id: UserId,
        operation: UpdateMfaOperation,
    },
    SiteCreate {
        site_id: SiteId,
    },
    SiteUpdate {
        site_id: SiteId,
        user_id: UserId,
        previous_fields: SiteFields<'a>,
        changed_fields: SiteFields<'a>,
    },
    PageCreate {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: PageRevisionId,
        category_id: PageCategoryId,
    },
    PageEdit {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: Option<PageRevisionId>,
    },
    PageMove {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: PageRevisionId,
        old_slug: &'a str,
        new_slug: &'a str,
    },
    PageDelete {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: PageRevisionId,
        page_slug: &'a str,
    },
    PageUndelete {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: PageRevisionId,
        category_id: PageCategoryId,
        page_slug: &'a str,
    },
    PageRollback {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        revision_id: Option<PageRevisionId>,
        revision_number: i32,
    },
    PageLayoutUpdate {
        user_id: UserId,
        site_id: SiteId,
        page_id: PageId,
        layout: Option<Layout>,
    },
}

impl<'a> AuditEvent<'a> {
    pub fn extract(&self, ip_address: IpAddr) -> Result<RawAuditEvent<'a>> {
        let raw_event = match *self {
            AuditEvent::UserCreate { user_id } => RawAuditEvent {
                event_type: "user.create",
                ip_address,
                user_id: Some(user_id),
                site_id: None,
                page_id: None,
                extra_id_1: None,
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::UserUpdate {
                user_id,
                ref previous_fields,
                ref changed_fields,
            } => {
                let previous_fields_json = serde_json::to_string(previous_fields)?;
                let changed_fields_json = serde_json::to_string(changed_fields)?;

                RawAuditEvent {
                    event_type: "user.update",
                    ip_address,
                    user_id: Some(user_id),
                    site_id: None,
                    page_id: None,
                    extra_id_1: None,
                    extra_id_2: None,
                    extra_string_1: Some(Cow::Owned(previous_fields_json)),
                    extra_string_2: Some(Cow::Owned(changed_fields_json)),
                    extra_number: None,
                }
            }
            AuditEvent::UserUpdateMfa { user_id, operation } => RawAuditEvent {
                event_type: "user.update_mfa",
                ip_address,
                user_id: Some(user_id),
                site_id: None,
                page_id: None,
                extra_id_1: None,
                extra_id_2: None,
                extra_string_1: Some(cow!(operation.value())),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::SiteCreate { site_id } => RawAuditEvent {
                event_type: "site.create",
                ip_address,
                user_id: None,
                site_id: Some(site_id),
                page_id: None,
                extra_id_1: None,
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::SiteUpdate {
                user_id,
                site_id,
                ref previous_fields,
                ref changed_fields,
            } => {
                let previous_fields_json = serde_json::to_string(previous_fields)?;
                let changed_fields_json = serde_json::to_string(changed_fields)?;

                RawAuditEvent {
                    event_type: "site.update",
                    ip_address,
                    user_id: Some(user_id),
                    site_id: Some(site_id),
                    page_id: None,
                    extra_id_1: None,
                    extra_id_2: None,
                    extra_string_1: Some(Cow::Owned(previous_fields_json)),
                    extra_string_2: Some(Cow::Owned(changed_fields_json)),
                    extra_number: None,
                }
            }
            AuditEvent::PageCreate {
                user_id,
                site_id,
                page_id,
                revision_id,
                category_id,
            } => RawAuditEvent {
                event_type: "page.create",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: Some(revision_id),
                extra_id_2: Some(category_id),
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::PageEdit {
                user_id,
                site_id,
                page_id,
                revision_id,
            } => RawAuditEvent {
                event_type: "page.edit",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: revision_id,
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::PageMove {
                user_id,
                site_id,
                page_id,
                revision_id,
                old_slug,
                new_slug,
            } => RawAuditEvent {
                event_type: "page.move",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: Some(revision_id),
                extra_id_2: None,
                extra_string_1: Some(cow!(old_slug)),
                extra_string_2: Some(cow!(new_slug)),
                extra_number: None,
            },
            AuditEvent::PageDelete {
                user_id,
                site_id,
                page_id,
                revision_id,
                page_slug,
            } => RawAuditEvent {
                event_type: "page.delete",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: Some(revision_id),
                extra_id_2: None,
                extra_string_1: Some(cow!(page_slug)),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::PageUndelete {
                user_id,
                site_id,
                page_id,
                revision_id,
                category_id,
                page_slug,
            } => RawAuditEvent {
                event_type: "page.undelete",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: Some(revision_id),
                extra_id_2: Some(category_id),
                extra_string_1: Some(cow!(page_slug)),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::PageRollback {
                user_id,
                site_id,
                page_id,
                revision_id,
                revision_number,
            } => RawAuditEvent {
                event_type: "page.rollback",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: revision_id,
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: Some(revision_number),
            },
            AuditEvent::PageLayoutUpdate {
                user_id,
                site_id,
                page_id,
                layout,
            } => RawAuditEvent {
                event_type: "page_layout.update",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: None,
                extra_id_2: None,
                extra_string_1: layout.map(|l| cow!(l.value())),
                extra_string_2: None,
                extra_number: None,
            },
        };

        Ok(raw_event)
    }
}

/// The raw data fields to be inserted into the `audit_log` table.
#[derive(Debug, Clone)]
pub struct RawAuditEvent<'a> {
    pub event_type: &'static str,
    pub ip_address: IpAddr,
    pub user_id: Option<UserId>,
    pub site_id: Option<SiteId>,
    pub page_id: Option<PageId>,
    pub extra_id_1: Option<i64>,
    pub extra_id_2: Option<i64>,
    pub extra_string_1: Option<Cow<'a, str>>,
    pub extra_string_2: Option<Cow<'a, str>>,
    pub extra_number: Option<i32>,
}

// Ancillary structures

#[derive(Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UserFields<'a> {
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub name: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub slug: Maybe<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub email: Maybe<&'a str>,
    // NOTE: We don't log the password value, hash or otherwise,
    //       instead we record whether a password value is *present*
    //       or not. See DISABLED_PASSWORD_HASH in UserService.
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub password: Maybe<bool>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub locales: Maybe<&'a [String]>,
    // NOTE: This is simply whether an avatar is set or not, not its value.
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub avatar: Maybe<bool>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub real_name: Maybe<Option<&'a str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub gender: Maybe<Option<&'a str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub birthday: Maybe<Option<Date>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub location: Maybe<Option<&'a str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub biography: Maybe<Option<&'a str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub user_page: Maybe<Option<&'a str>>,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SiteFields<'a> {
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub name: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub slug: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub tagline: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub description: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub license: Maybe<License>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub locale: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub default_page: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub top_bar_page: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub side_bar_page: Maybe<&'a str>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub preferred_domain: Maybe<Option<&'a str>>,
    #[serde(skip_serializing_if = "Maybe::is_unset")]
    pub layout: Maybe<Option<Layout>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UpdateMfaOperation {
    Setup,
    ResetRecoveryCodes,
    Disable,
}

impl UpdateMfaOperation {
    pub fn value(self) -> &'static str {
        match self {
            UpdateMfaOperation::Setup => "setup",
            UpdateMfaOperation::ResetRecoveryCodes => "reset_recovery_codes",
            UpdateMfaOperation::Disable => "disable",
        }
    }
}
