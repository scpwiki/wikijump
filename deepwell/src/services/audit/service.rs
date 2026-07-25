/*
 * services/audit/service.rs
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

use super::structs::{AuditEvent, RawAuditEvent};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::audit_log::{self, Entity as AuditLog};
use crate::services::ServiceContext;
use sea_orm::{EntityTrait, Set};
use std::net::IpAddr;

#[derive(Debug)]
pub struct AuditService;

impl AuditService {
    /// Write a new event to the audit log.
    pub async fn log(
        ctx: &ServiceContext<'_>,
        ip_address: IpAddr,
        event: AuditEvent<'_>,
    ) -> Result<i64> {
        let make_error = || Error::new("failed to write audit log", ErrorType::AuditLog);

        let RawAuditEvent {
            event_type,
            ip_address,
            user_id,
            site_id,
            page_id,
            extra_id_1,
            extra_id_2,
            extra_string_1,
            extra_string_2,
            extra_number,
        } = event.extract(ip_address).or_raise(make_error)?;

        let model = audit_log::ActiveModel {
            event_type: Set(str!(event_type)),
            ip_address: Set(str!(ip_address)),
            user_id: Set(user_id),
            site_id: Set(site_id),
            page_id: Set(page_id),
            extra_id_1: Set(extra_id_1),
            extra_id_2: Set(extra_id_2),
            extra_string_1: Set(extra_string_1.map(|s| str!(s))),
            extra_string_2: Set(extra_string_2.map(|s| str!(s))),
            extra_number: Set(extra_number),
            ..Default::default()
        };

        let txn = ctx.transaction();
        let event_id = AuditLog::insert(model)
            .exec(txn)
            .await
            .or_raise(make_error)?
            .last_insert_id;

        info!("Adding audit log event '{event_type}' (ID {event_id})");
        Ok(event_id)
    }
}
