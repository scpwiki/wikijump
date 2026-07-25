/*
 * middleware.rs
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

use http::Request;
use std::borrow::Cow;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::error::StdResult;
use crate::types::Reference;

#[derive(Debug, Clone)]
pub struct RequestContextHeaders {
    pub session_token: Option<String>,
    pub site_id: Option<i64>,
    pub page_ref: Option<Reference<'static>>,
}

/// tower middleware layer to extract relevant headers from the request
/// and store them in the request extensions for later use in the handlers.
#[derive(Debug, Clone)]
pub struct RequestContextLayer;

impl<S> Layer<S> for RequestContextLayer {
    type Service = RequestContextService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestContextService { service }
    }
}

// Service that does the interception of the request.
#[derive(Debug, Clone)]
pub struct RequestContextService<S> {
    service: S,
}

impl<S, Body> Service<Request<Body>> for RequestContextService<S>
where
    S: Service<Request<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<StdResult<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let context = request_context_headers(&request);
        request.extensions_mut().insert(context);
        self.service.call(request)
    }
}

fn request_context_headers<Body>(request: &Request<Body>) -> RequestContextHeaders {
    let session_token = request
        .headers()
        .get("X-Deepwell-Session-Token")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let site_id = request
        .headers()
        .get("X-Deepwell-Site-Id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    let page_ref = request
        .headers()
        .get("X-Deepwell-Page")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .parse::<i64>()
                .map(Reference::Id)
                .unwrap_or_else(|_| Reference::Slug(Cow::Owned(value.to_owned())))
        });

    RequestContextHeaders {
        session_token,
        site_id,
        page_ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_request_context_headers() {
        let request = Request::builder()
            .header("X-Deepwell-Session-Token", "session-token")
            .header("X-Deepwell-Site-Id", "42")
            .header("X-Deepwell-Page", "category:page")
            .body(())
            .expect("request should build");

        let headers = request_context_headers(&request);
        assert_eq!(headers.session_token.as_deref(), Some("session-token"));
        assert_eq!(headers.site_id, Some(42));
        assert_eq!(
            headers.page_ref,
            Some(Reference::Slug(Cow::Borrowed("category:page")))
        );
    }

    #[test]
    fn numeric_page_header_uses_id_reference() {
        let request = Request::builder()
            .header("X-Deepwell-Page", "123")
            .body(())
            .expect("request should build");

        assert_eq!(
            request_context_headers(&request).page_ref,
            Some(Reference::Id(123))
        );
    }
}
