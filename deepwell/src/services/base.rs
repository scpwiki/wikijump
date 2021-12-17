/*
 * services/base.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use crate::api::{ApiRequest, ApiServerState};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;

/// The base service, with common data and helpers for other services.
#[derive(Debug)]
pub struct BaseService<'txn> {
    _state: ApiServerState,
    transaction: &'txn DatabaseTransaction,
}

impl<'txn> BaseService<'txn> {
    pub fn new(req: &ApiRequest, transaction: &'txn DatabaseTransaction) -> Self {
        BaseService {
            _state: Arc::clone(req.state()),
            transaction,
        }
    }

    #[inline]
    pub fn transaction(&self) -> &DatabaseTransaction {
        &self.transaction
    }
}
