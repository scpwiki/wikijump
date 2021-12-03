/*
 * web/ratelimit.rs
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

//! Rate-limiter implementation using the `governor` crate.
//!
//! Based on the implementation found in the [`tide-governor`](https://github.com/ohmree/tide-governor)
//! trait, which cannot be used here because we need to expose a hole for privileged access
//! (that is, the web server backend, as opposed to external API consumers).

use governor::state::keyed::DefaultKeyedStateStore;
use governor::{
    clock::{Clock, DefaultClock},
    Quota, RateLimiter,
};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use tide::utils::async_trait;
use tide::{Error, Middleware, Next, Request, Response, StatusCode};

lazy_static! {
    static ref CLOCK: DefaultClock = DefaultClock::default();
}

/// Tide middleware to rate-limit new requests.
///
/// Once the rate-limit has been reached, all further
/// requests will be responded to HTTP 429 (Too Many Requests)
/// and a `Retry-After` header with the amount of time
/// until the next request will be permitted.
#[derive(Debug, Clone)]
pub struct GovernorMiddleware {
    limiter: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>,
    secret: String,
}

impl GovernorMiddleware {
    pub fn per_minute(times: NonZeroU32, secret: &str) -> Self {
        GovernorMiddleware {
            limiter: Arc::new(RateLimiter::<IpAddr, _, _>::keyed(Quota::per_minute(
                times,
            ))),
            secret: secret.to_owned(),
        }
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for GovernorMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        macro_rules! next {
            () => {
                next.run(req).await
            };
        }

        // Check for privileged exemption
        if let Some(values) = req.header("X-Exempt-RateLimit") {
            if let Some(value) = values.get(0) {
                if value.as_str() == &self.secret {
                    tide::log::debug!("Skipping rate-limit due to exemption");
                    return Ok(next!());
                }
            }

            tide::log::warn!("Invalid X-Exempt-RateLimit header found! {:?}", values);
        }

        // Get IP address
        let remote = {
            let remote_str = req.remote().ok_or_else(|| {
                Error::from_str(
                    StatusCode::InternalServerError,
                    "Unable to get remote address of request",
                )
            })?;

            // Try parsing as a socket, then as an IP address only
            match remote_str.parse::<SocketAddr>() {
                Ok(addr) => addr.ip(),
                Err(_) => remote_str.parse()?,
            }
        };

        // Check rate-limit bucket by IP address
        tide::log::trace!("Checking rate-limit for IP address {}", remote);
        match self.limiter.check_key(&remote) {
            Ok(_) => {
                tide::log::debug!("Allowing IP address {}", remote);
                Ok(next!())
            }
            Err(negative) => {
                let wait_time = negative.wait_time_from(CLOCK.now());
                let res = Response::builder(StatusCode::TooManyRequests)
                    .header(
                        tide::http::headers::RETRY_AFTER,
                        wait_time.as_secs().to_string(),
                    )
                    .build();

                tide::log::debug!(
                    "Denying IP address {} for {} seconds",
                    remote,
                    wait_time.as_secs()
                );
                Ok(res)
            }
        }
    }
}
