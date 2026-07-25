/*
 * tests/session_token_logging.rs
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

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::ADMIN_USER_ID;
use deepwell::services::SessionService;
use deepwell::services::session::{CreateSession, RenewSession};
use log::{LevelFilter, Log, Metadata, Record};
use serde_json::json;
use std::sync::Mutex;

#[derive(Debug)]
struct CaptureLogger {
    messages: Mutex<Vec<String>>,
}

impl Log for CaptureLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        self.messages
            .lock()
            .expect("capture logger lock should not be poisoned")
            .push(format!(
                "{} {} {}",
                record.level(),
                record.target(),
                record.args(),
            ));
    }

    fn flush(&self) {}
}

static LOGGER: CaptureLogger = CaptureLogger {
    messages: Mutex::new(Vec::new()),
};

#[tokio::test]
async fn session_lifecycle_does_not_log_bearer_values() {
    log::set_logger(&LOGGER).expect("test logger should initialize once");

    let runner = TestRunner::setup().await;
    let mut trace_config = runner.config().clone();
    trace_config.logger_level = LevelFilter::Trace;
    let logger_level = trace_config.operational_logger_level();
    assert_eq!(logger_level, LevelFilter::Debug);
    log::set_max_level(logger_level);

    LOGGER
        .messages
        .lock()
        .expect("capture logger lock should not be poisoned")
        .clear();

    log::debug!("session debug control");
    log::trace!("session trace control");
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "session-token-logging-test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("test session should be created");
    let other_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "session-token-logging-test-other".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("other test session should be created");
    assert_eq!(
        SessionService::invalidate_others(
            runner.context(),
            &session_token,
            ADMIN_USER_ID,
        )
        .await
        .expect("other test session should be invalidated"),
        1,
    );
    let renewed_session_token = SessionService::renew(
        runner.context(),
        RenewSession {
            old_session_token: session_token.clone(),
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "session-token-logging-test-renewed".to_owned(),
        },
    )
    .await
    .expect("test session should be renewed");
    SessionService::get(runner.context(), &renewed_session_token)
        .await
        .expect("renewed test session should be found");

    let restricted_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "session-token-logging-test-mfa".to_owned(),
            restricted: true,
        },
    )
    .await
    .expect("restricted test session should be created");
    run_endpoint_err!(
        runner,
        auth_mfa_verify,
        json!({
            "session_token": restricted_session_token,
            "totp_or_code": "000000",
            "ip_address": common::IP_ADDRESS,
            "user_agent": "session-token-logging-test-mfa",
        }),
    );

    SessionService::invalidate(runner.context(), restricted_session_token.clone())
        .await
        .expect("restricted test session should be invalidated");
    SessionService::invalidate(runner.context(), renewed_session_token.clone())
        .await
        .expect("renewed test session should be invalidated");

    let messages = LOGGER
        .messages
        .lock()
        .expect("capture logger lock should not be poisoned")
        .join("\n");

    assert!(
        messages.contains("session debug control")
            && messages.contains("Creating new session")
            && messages.contains("Looking up session by token")
            && messages.contains("Renewing session")
            && messages.contains("Verifying user's MFA for login")
            && messages.contains("Invalidating session"),
        "debug logs and useful session events should remain enabled",
    );
    assert!(
        !messages.contains("session trace control"),
        "trace logs should be rejected by the operational logging policy",
    );
    let bearers = [
        &session_token,
        &other_session_token,
        &renewed_session_token,
        &restricted_session_token,
    ];
    let redacted_messages = bearers.iter().fold(messages.clone(), |logs, bearer| {
        logs.replace(*bearer, "<redacted bearer>")
    });
    for bearer in bearers {
        assert!(
            !messages.contains(bearer),
            "session lifecycle logs must not contain bearer values:\n{}",
            redacted_messages,
        );
    }

    runner.teardown().await;
}
