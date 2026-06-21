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
use crate::services::authorization_token::AuthorizedObject;
use crate::services::filter::FilterSummary;
use crate::types::{PageLockType, Permission};
use ftml::layout::Layout;
use sea_orm::prelude::TimeDateTimeWithTimeZone;
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
        user_id: i64,
    },
    UserUpdate {
        user_id: i64,
        previous_fields: UserFields<'a>,
        changed_fields: UserFields<'a>,
    },
    UserUpdateMfa {
        user_id: i64,
        operation: UpdateMfaOperation,
    },
    SiteCreate {
        site_id: i64,
    },
    SiteUpdate {
        site_id: i64,
        user_id: i64,
        previous_fields: SiteFields<'a>,
        changed_fields: SiteFields<'a>,
    },
    PageCreate {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: i64,
        category_id: i64,
    },
    PageEdit {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: Option<i64>,
    },
    PageMove {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: i64,
        old_slug: &'a str,
        new_slug: &'a str,
    },
    PageDelete {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: i64,
        page_slug: &'a str,
    },
    PageUndelete {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: i64,
        category_id: i64,
        page_slug: &'a str,
    },
    PageRollback {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        revision_id: Option<i64>,
        revision_number: i32,
    },
    PageLayoutUpdate {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        layout: Option<Layout>,
    },
    FilterViolation {
        object: ObjectScope,
        info: &'a FilterSummary,
        field: &'a str,
        value: &'a str,
    },
    RoleCreate {
        site_id: i64,
        role_id: i64,
        creating_user_id: i64,
    },
    RoleUpdate {
        role_id: i64,
        name: Option<&'a str>,
        description: Option<&'a str>,
        updating_user_id: i64,
    },
    #[allow(dead_code)]
    RoleDelete {
        role_id: i64,
        deleting_user_id: i64,
    },
    RoleReparent {
        role_id: i64,
        new_parent_id: Option<i64>,
        reparenting_user_id: i64,
    },
    GrantUserRole {
        user_id: i64,
        role_id: i64,
        assigning_user_id: i64,
        expires_at: Option<TimeDateTimeWithTimeZone>,
    },
    RevokeUserRole {
        user_id: i64,
        role_id: i64,
        revoking_user_id: i64,
    },
    UpdatePermissions {
        role_id: i64,
        updating_user_id: i64,
        old_permissions: Vec<Permission<'static>>,
        new_permissions: Vec<Permission<'static>>,
    },
    PageLockCreate {
        user_id: i64,
        site_id: i64,
        page_id: i64,
        page_lock_id: i64,
        lock_type: PageLockType,
    },
    PageLockRemove {
        user_id: i64,
        page_id: i64,
        page_lock_id: i64,
        lock_type: PageLockType,
    },
    AuthorizationTokenCreate {
        user_id: i64,
        object_type: AuthorizedObject,
        description: &'a str,
    },
    AuthorizationTokenVerify {
        token: &'a str,
        token_id: i32,
        object_type: AuthorizedObject,
    },
}

impl<'a> AuditEvent<'a> {
    pub fn extract(&self, ip_address: IpAddr) -> Result<RawAuditEvent<'a>> {
        let make_error = || {
            Error::new(
                format!("failed to extract raw audit event from {:#?}", self),
                ErrorType::AuditLog,
            )
        };

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
                let previous_fields_json =
                    serde_json::to_string(previous_fields).or_raise(make_error)?;

                let changed_fields_json =
                    serde_json::to_string(changed_fields).or_raise(make_error)?;

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
                let previous_fields_json =
                    serde_json::to_string(previous_fields).or_raise(make_error)?;

                let changed_fields_json =
                    serde_json::to_string(changed_fields).or_raise(make_error)?;

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
            AuditEvent::FilterViolation {
                object,
                info,
                field,
                value,
            } => {
                #[derive(Serialize, Debug)]
                struct Metadata<'a> {
                    #[serde(flatten)]
                    info: &'a FilterSummary,
                    field: &'a str,
                    value: &'a str,
                }

                let mut user_id = None;
                let mut site_id = None;
                let mut page_id = None;
                let mut extra_id_1 = None;

                match object {
                    ObjectScope::User(id) => user_id = Some(id),
                    ObjectScope::Site(id) => site_id = Some(id),
                    ObjectScope::Page(id) => page_id = Some(id),
                    ObjectScope::File(id) => extra_id_1 = Some(id),
                    ObjectScope::Other => (),
                }

                let metadata_json =
                    serde_json::to_string(&Metadata { info, field, value })
                        .or_raise(make_error)?;

                RawAuditEvent {
                    event_type: "filter.violation",
                    ip_address,
                    user_id,
                    site_id,
                    page_id,
                    extra_id_1,
                    extra_id_2: None,
                    extra_string_1: Some(Cow::Owned(metadata_json)),
                    extra_string_2: None,
                    extra_number: None,
                }
            }
            AuditEvent::RoleCreate {
                site_id,
                role_id,
                creating_user_id,
            } => RawAuditEvent {
                event_type: "role.create",
                ip_address,
                user_id: Some(creating_user_id),
                site_id: Some(site_id),
                page_id: None,
                extra_id_1: Some(role_id),
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::RoleUpdate {
                role_id,
                name,
                description,
                updating_user_id,
            } => RawAuditEvent {
                event_type: "role.update",
                ip_address,
                user_id: Some(updating_user_id),
                site_id: None,
                page_id: None,
                extra_id_1: Some(role_id),
                extra_id_2: None,
                extra_string_1: name.map(Cow::Borrowed),
                extra_string_2: description.map(Cow::Borrowed),
                extra_number: None,
            },
            AuditEvent::RoleDelete {
                role_id,
                deleting_user_id,
            } => RawAuditEvent {
                event_type: "role.delete",
                ip_address,
                user_id: Some(deleting_user_id),
                site_id: None,
                page_id: None,
                extra_id_1: Some(role_id),
                extra_id_2: None,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::RoleReparent {
                role_id,
                new_parent_id,
                reparenting_user_id,
            } => RawAuditEvent {
                event_type: "role.reparent",
                ip_address,
                user_id: Some(reparenting_user_id),
                site_id: None,
                page_id: None,
                extra_id_1: Some(role_id),
                extra_id_2: new_parent_id,
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::GrantUserRole {
                user_id,
                role_id,
                assigning_user_id,
                expires_at,
            } => RawAuditEvent {
                event_type: "user_role.grant",
                ip_address,
                user_id: Some(assigning_user_id),
                site_id: None,
                page_id: None,
                extra_id_1: Some(user_id),
                extra_id_2: Some(role_id),
                extra_string_1: expires_at.map(|dt| Cow::Owned(dt.to_string())),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::RevokeUserRole {
                user_id,
                role_id,
                revoking_user_id,
            } => RawAuditEvent {
                event_type: "user_role.revoke",
                ip_address,
                user_id: Some(revoking_user_id),
                site_id: None,
                page_id: None,
                extra_id_1: Some(user_id),
                extra_id_2: Some(role_id),
                extra_string_1: None,
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::UpdatePermissions {
                role_id,
                updating_user_id,
                ref old_permissions,
                ref new_permissions,
            } => {
                let old_perms_json =
                    serde_json::to_string(old_permissions).or_raise(make_error)?;
                let new_perms_json =
                    serde_json::to_string(new_permissions).or_raise(make_error)?;

                RawAuditEvent {
                    event_type: "role.update_permissions",
                    ip_address,
                    user_id: Some(updating_user_id),
                    site_id: None,
                    page_id: None,
                    extra_id_1: Some(role_id),
                    extra_id_2: None,
                    extra_string_1: Some(Cow::Owned(old_perms_json)),
                    extra_string_2: Some(Cow::Owned(new_perms_json)),
                    extra_number: None,
                }
            }
            AuditEvent::PageLockCreate {
                user_id,
                site_id,
                page_id,
                page_lock_id,
                lock_type,
            } => RawAuditEvent {
                event_type: "page_lock.create",
                ip_address,
                user_id: Some(user_id),
                site_id: Some(site_id),
                page_id: Some(page_id),
                extra_id_1: Some(page_lock_id),
                extra_id_2: None,
                extra_string_1: Some(Cow::Owned(str!(lock_type))),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::PageLockRemove {
                user_id,
                page_id,
                page_lock_id,
                lock_type,
            } => RawAuditEvent {
                event_type: "page_lock.remove",
                ip_address,
                user_id: Some(user_id),
                site_id: None,
                page_id: Some(page_id),
                extra_id_1: Some(page_lock_id),
                extra_id_2: None,
                extra_string_1: Some(Cow::Owned(str!(lock_type))),
                extra_string_2: None,
                extra_number: None,
            },
            AuditEvent::AuthorizationTokenCreate {
                user_id,
                object_type,
                description,
            } => {
                #[derive(Serialize, Debug)]
                struct Metadata<'a> {
                    description: &'a str,
                }

                let metadata_json = serde_json::to_string(&Metadata { description })
                    .or_raise(make_error)?;

                RawAuditEvent {
                    event_type: "authorization_token.create",
                    ip_address,
                    user_id: Some(user_id),
                    site_id: None,
                    page_id: None,
                    extra_id_1: None,
                    extra_id_2: None,
                    extra_string_1: Some(Cow::Borrowed(object_type.name())),
                    extra_string_2: Some(Cow::Owned(metadata_json)),
                    extra_number: None,
                }
            }
            AuditEvent::AuthorizationTokenVerify {
                token,
                token_id,
                object_type,
            } => RawAuditEvent {
                event_type: "authorization_token.verify",
                ip_address,
                user_id: None,
                site_id: None,
                page_id: None,
                extra_id_1: Some(i64::from(token_id)),
                extra_id_2: None,
                extra_string_1: Some(Cow::Borrowed(object_type.name())),
                extra_string_2: Some(Cow::Owned(str!(token))),
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
    pub user_id: Option<i64>,
    pub site_id: Option<i64>,
    pub page_id: Option<i64>,
    pub extra_id_1: Option<i64>,
    pub extra_id_2: Option<i64>,
    pub extra_string_1: Option<Cow<'a, str>>,
    pub extra_string_2: Option<Cow<'a, str>>,
    pub extra_number: Option<i32>,
}

// Ancillary structures

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectScope {
    User(i64),
    Site(i64),
    Page(i64),
    File(i64),
    Other,
}

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
    pub website: Maybe<Option<&'a str>>,
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
