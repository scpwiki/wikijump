/*
 * services/session/service.rs
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

//! Manages sessions for authenticated users.
//!
//! Once a user has been authenticated (password, MFA, etc)
//! then a session can be created for them, which will enable
//! them to interact with the platform.
//!
//! The session token is the only means through which a session
//! is validated. It is a unique, securely randomly generated value
//! which represents the current session. It has a somewhat short
//! expiry (30 minutes) which needs to be renewed by the client
//! periodically.

use super::prelude::*;
use crate::utils::assert_is_csprng;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;

/// Fixed prefix for all session tokens.
const SESSION_TOKEN_PREFIX: &str = "wj:";

/// Length of each session token.
const SESSION_TOKEN_LENGTH: usize = 64;

#[derive(Debug)]
pub struct SessionService;

impl SessionService {
    /// Securely generates a new session token.
    ///
    /// Example generated token: `wj:T9iF6vfjoYYE20QzrybV2C1V4K0LchHXsNVipX8G1GZ9vSJf0rvQpJ4YC8c8MAQ3`.
    fn new_token() -> String {
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        let mut token = Alphanumeric.sample_string(&mut rng, SESSION_TOKEN_LENGTH);
        token.insert_str(0, SESSION_TOKEN_PREFIX);

        token
    }

    /// Prunes all expired sessions from the database.
    ///
    /// # Returns
    /// The number of pruned sessions.
    pub async fn prune(ctx: &ServiceContext<'_>) -> Result<u64> {
        tide::log::info!("Pruning all expired sessions");

        let txn = ctx.transaction();
        let DeleteResult { rows_affected } = Session::delete_many()
            .filter(session::Column::ExpiresAt.lte(now()))
            .exec(txn)
            .await?;

        tide::log::debug!("{rows_affected} expired sessions were pruned");
        Ok(rows_affected)
    }
}

#[test]
fn new_token() {
    let token = SessionService::new_token();
    assert_eq!(token.len(), SESSION_TOKEN_LENGTH + SESSION_TOKEN_PREFIX.len());
    assert_eq!(token.starts_with(SESSION_TOKEN_PREFIX));
}
