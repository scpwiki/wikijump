/*
 * handler/misc.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

use crate::state::ServerState;
use axum::body::Body;
use axum::extract::State;
use axum::http::header;
use axum::http::status::StatusCode;
use axum::response::Response;

fn text_response(body: &'static str, status: StatusCode) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body))
        .expect("Unable to convert response data")
}

pub async fn handle_health_check(State(state): State<ServerState>) -> Response {
    // DEEPWELL's ping ensures both Postgres and Redis are connected
    match state.deepwell.ping().await {
        Ok(()) => text_response("✅", StatusCode::OK),
        Err(error) => {
            error!("Unable to perform health check: {error}");
            text_response("❌", StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

pub async fn handle_invalid_method() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body;

    #[tokio::test]
    async fn text_response_sets_status_content_type_and_body() {
        let response = text_response("ready", StatusCode::OK);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8",
        );
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"ready");
    }

    #[tokio::test]
    async fn invalid_methods_are_rejected() {
        assert_eq!(
            handle_invalid_method().await,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
