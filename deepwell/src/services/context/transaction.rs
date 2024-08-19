/*
 * services/context/transaction.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use crate::api::ServerState;
use crate::database::SqlxTransaction;
use crate::services::Result;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard, RawMutex};
use std::sync::Arc;

#[derive(Debug)]
pub struct ServiceTransaction {
    mutex: Mutex<Option<SqlxTransaction<'static>>>,
}

impl ServiceTransaction {
    pub fn new() -> Self {
        ServiceTransaction {
            mutex: Mutex::new(None),
        }
    }

    pub async fn get(
        &self,
        state: &ServerState,
    ) -> Result<MappedMutexGuard<SqlxTransaction<'static>>> {
        let mut guard = self.mutex.lock();

        // If a transaction hasn't been created yet, then start it
        if guard.is_none() {
            let txn = state.database_sqlx.begin().await?;
            *guard = Some(txn);
        }

        // At this point, the field must be Some(_)
        Ok(MutexGuard::map(guard, |inner| {
            inner
                .as_mut()
                .expect("No transaction present despite check")
        }))
    }

    pub async fn commit(&mut self) -> Result<()> {
        let mut guard = self.mutex.lock();

        if let Some(txn) = guard.take() {
            txn.commit().await?;
        }

        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<()> {
        let mut guard = self.mutex.lock();

        if let Some(txn) = guard.take() {
            txn.rollback().await?;
        }

        Ok(())
    }
}
