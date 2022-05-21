/*
 * services/context.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

#[derive(Debug)]
pub struct ServiceContext<'txn> {
    state: ApiServerState,
    transaction: &'txn DatabaseTransaction,
}

impl<'txn> ServiceContext<'txn> {
    #[inline]
    pub fn new(req: &ApiRequest, transaction: &'txn DatabaseTransaction) -> Self {
        Self::from_raw(req.state(), transaction)
    }

    pub fn from_raw(
        state: &ApiServerState,
        transaction: &'txn DatabaseTransaction,
    ) -> Self {
        ServiceContext {
            state: Arc::clone(state),
            transaction,
        }
    }

    // Getters
    #[inline]
    pub fn transaction(&self) -> &'txn DatabaseTransaction {
        self.transaction
    }
}
