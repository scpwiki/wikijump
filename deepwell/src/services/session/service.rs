/*
 * services/session/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::models::session::{self, Entity as Session, Model as SessionModel};
use crate::models::user::{Entity as User, Model as UserModel};
use crate::utils::assert_is_csprng;
use chrono::Duration;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;

/// Fixed prefix for all session tokens.
const SESSION_TOKEN_PREFIX: &str = "wj:";

/// Length of each session token.
const SESSION_TOKEN_LENGTH: usize = 64;

/// Number of minutes that normal sessions last before expiring.
const NORMAL_SESSION_MINUTES: i64 = 30;

/// Number of minutes that restricted sessions last.
///
/// This is essentially the time the user has to finish logging in
/// before they must begin again.
const RESTRICTED_SESSION_MINUTES: i64 = 5;

#[derive(Debug)]
pub struct SessionService;

impl SessionService {
    /// Creates a new session with the given parameters.
    ///
    /// # Returns
    /// The generated session token.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateSession {
            user_id,
            ip_address,
            user_agent,
            restricted,
        }: CreateSession,
    ) -> Result<String> {
        tide::log::info!(
            "Creating new session for user ID {user_id} (restricted: {restricted})",
        );

        let txn = ctx.transaction();
        let token = Self::new_token();
        let expiry = if restricted {
            now() + Duration::minutes(RESTRICTED_SESSION_MINUTES)
        } else {
            now() + Duration::minutes(NORMAL_SESSION_MINUTES)
        };

        let model = session::ActiveModel {
            session_token: Set(token),
            user_id: Set(user_id),
            created_at: Set(now()),
            expires_at: Set(expiry),
            ip_address: Set(str!(ip_address)), // TODO inet type?
            user_agent: Set(user_agent),
            restricted: Set(restricted),
        };

        let SessionModel { session_token, .. } = model.insert(txn).await?;
        tide::log::info!("Created new session token");
        Ok(session_token)
    }

    /// Securely generates a new session token.
    ///
    /// Example generated token: `wj:T9iF6vfjoYYE20QzrybV2C1V4K0LchHXsNVipX8G1GZ9vSJf0rvQpJ4YC8c8MAQ3`.
    fn new_token() -> String {
        tide::log::debug!("Generating a new session token");
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        let mut token = Alphanumeric.sample_string(&mut rng, SESSION_TOKEN_LENGTH);
        token.insert_str(0, SESSION_TOKEN_PREFIX);

        token
    }

    /// Gets a session model from its token.
    /// Yields an error if the given session token does not exist or is expired.
    pub async fn get(
        ctx: &ServiceContext<'_>,
        session_token: &str,
    ) -> Result<SessionModel> {
        tide::log::info!("Looking up session with token {session_token}");
        Self::get_optional(ctx, session_token)
            .await?
            .ok_or(Error::NotFound)
    }

    async fn get_optional(
        ctx: &ServiceContext<'_>,
        session_token: &str,
    ) -> Result<Option<SessionModel>> {
        let txn = ctx.transaction();
        let session = Session::find()
            .filter(
                Condition::all()
                    .add(session::Column::SessionToken.eq(session_token))
                    .add(session::Column::ExpiresAt.gt(now())),
            )
            .one(txn)
            .await?;

        Ok(session)
    }

    /// Gets the associated `UserModel` from an active session.
    ///
    /// Performs a join rather than two separate fetches.
    /// Yields an error if the given session token does not exist or is expired.
    ///
    /// The `restricted` status must match the argument passed.
    pub async fn get_user(
        ctx: &ServiceContext<'_>,
        session_token: &str,
        restricted: bool,
    ) -> Result<UserModel> {
        tide::log::info!("Looking up user for session token");

        let txn = ctx.transaction();
        let user = User::find()
            .join(JoinType::Join, session::Relation::User.def())
            .filter(
                Condition::all()
                    .add(session::Column::SessionToken.eq(session_token))
                    .add(session::Column::ExpiresAt.gt(now()))
                    .add(session::Column::Restricted.eq(restricted)),
            )
            .one(txn)
            .await?
            .ok_or(Error::NotFound)?;

        Ok(user)
    }

    /// Gets all active sessions for a user.
    /// For instance, useful for listing all sessions and their information.
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        user_id: i64,
    ) -> Result<Vec<SessionModel>> {
        tide::log::info!("Getting all sessions for user ID {user_id}");

        let txn = ctx.transaction();
        let sessions = Session::find()
            .filter(
                Condition::all()
                    .add(session::Column::UserId.eq(user_id))
                    .add(session::Column::ExpiresAt.gt(now())),
            )
            .all(txn)
            .await?;

        Ok(sessions)
    }

    /// Renews a session, invalidating the old one and creating a new one.
    ///
    /// # Returns
    /// The new session token.
    /// After this point, the previous session token will be invalid.
    pub async fn renew(
        ctx: &ServiceContext<'_>,
        RenewSession {
            old_session_token,
            user_id,
            ip_address,
            user_agent,
        }: RenewSession,
    ) -> Result<String> {
        tide::log::info!("Renewing session ID {old_session_token}");

        // Get existing session to ensure the token matches the passed user ID.
        let old_session = Self::get(ctx, &old_session_token).await?;
        if old_session.user_id != user_id {
            tide::log::error!(
                "Requested session renewal, user IDs do not match! (current: {}, request: {})",
                old_session.user_id,
                user_id,
            );

            return Err(Error::BadRequest);
        }

        // Invalid and recreate
        let (_, session_token) = try_join!(
            Self::invalidate(ctx, old_session_token),
            Self::create(
                ctx,
                CreateSession {
                    user_id,
                    ip_address,
                    user_agent,
                    restricted: false,
                }
            ),
        )?;

        Ok(session_token)
    }

    /// Invalidates the given session, causing it to be deleted.
    pub async fn invalidate(
        ctx: &ServiceContext<'_>,
        session_token: String,
    ) -> Result<()> {
        tide::log::info!("Invalidating session ID {session_token}");

        let txn = ctx.transaction();
        let DeleteResult { rows_affected } =
            Session::delete_by_id(session_token).exec(txn).await?;

        if rows_affected != 1 {
            tide::log::error!("This session was already deleted or does not exist");
            return Err(Error::NotFound);
        }

        Ok(())
    }

    /// Invalidates all others sessions _except_ the one listed.
    /// This enables a user to "log out all other sessions",
    /// a useful security feature. See [WJ-364].
    ///
    /// # Returns
    /// The number of invalidated sessions.
    ///
    /// [WJ-364]: https://scuttle.atlassian.net/browse/WJ-364
    pub async fn invalidate_others(
        ctx: &ServiceContext<'_>,
        session_token: &str,
        user_id: i64,
    ) -> Result<u64> {
        tide::log::info!("Invalidation all other session IDs for user ID {user_id}");

        let txn = ctx.transaction();
        let session = Self::get(ctx, session_token).await?;
        if session.user_id != user_id {
            tide::log::error!(
                "Requested invalidation of other sessions, user IDs do not match! (current: {}, request: {})",
                session.user_id,
                user_id,
            );

            return Err(Error::BadRequest);
        }

        // Delete all sessions from user_id, except if it's this session_token
        let DeleteResult { rows_affected } = Session::delete_many()
            .filter(
                Condition::all()
                    .add(session::Column::SessionToken.ne(session_token))
                    .add(session::Column::UserId.eq(user_id)),
            )
            .exec(txn)
            .await?;

        tide::log::debug!(
            "User ID {user_id}: {rows_affected} other sessions were invalidated",
        );
        Ok(rows_affected)
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
    assert!(token.starts_with(SESSION_TOKEN_PREFIX));
    assert_eq!(
        token.len(),
        SESSION_TOKEN_LENGTH + SESSION_TOKEN_PREFIX.len(),
    );
}
