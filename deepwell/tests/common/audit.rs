/*
 * tests/common/audit.rs
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

use super::TestRunner;
use deepwell::models::audit_log::{self, Entity as AuditLog};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

#[allow(dead_code)]
pub async fn latest_audit_event(
    runner: &TestRunner,
    event_type: &str,
    site_id: i64,
    target_user_id: i64,
) -> audit_log::Model {
    AuditLog::find()
        .filter(audit_log::Column::EventType.eq(event_type))
        .filter(audit_log::Column::SiteId.eq(site_id))
        .filter(audit_log::Column::UserId.eq(target_user_id))
        .order_by_desc(audit_log::Column::EventId)
        .one(runner.context().transaction())
        .await
        .expect("Unable to query audit log")
        .expect("Expected audit event was not found")
}
