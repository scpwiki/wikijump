/*
 * macros.rs
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

/// Like `try!()` or `?`, except it returns a `Response` for the error case.
/// This is for functions which return `Response` rather than `Result<T, E>`.
macro_rules! try_response {
    ($future:expr $(,)?) => {
        match $future.await {
            Ok(data) => data,
            Err(response) => return response,
        }
    };
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};

    async fn ok_path() -> Response {
        let value: u16 = try_response!(async { Ok::<_, Response>(204) });
        (StatusCode::OK, value.to_string()).into_response()
    }

    async fn error_path() -> Response {
        let _value: u16 = try_response!(async {
            Err::<u16, _>(StatusCode::BAD_REQUEST.into_response())
        });
        StatusCode::OK.into_response()
    }

    #[tokio::test]
    async fn try_response_unwraps_ok_value() {
        let response = ok_path().await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"204");
    }

    #[tokio::test]
    async fn try_response_returns_error_response() {
        let response = error_path().await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
