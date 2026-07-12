/*
 * error/convert.rs
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

//! This module concerns conversion methods for constrainted error types.
//!
//! JSONRPC expects a very particular kind of error structure, which our
//! error type does account for, but it needs some conversion to take
//! the stack of `exn::Frame`s and convert it.

use super::{Error, ErrorType};
use exn::{ErrorExt, Exn, Frame};
use jsonrpsee::types::error::ErrorObjectOwned;
use sea_orm::TransactionError;
use serde_json::{Value as JsonValue, json};

/// Unwraps Sea-ORM transaction error into a standard crate error.
///
/// Sea-ORM wraps all results from transactions in this enum.
/// This function either passes through the real error or makes
/// a new layer if it's a database connectivity issue.
pub fn unwrap_transaction_error(txn_error: TransactionError<Exn<Error>>) -> Exn<Error> {
    match txn_error {
        TransactionError::Transaction(error) => error,
        TransactionError::Connection(error) => error.raise().raise(Error::new(
            "database transaction failed",
            ErrorType::DatabaseQuery,
        )),
    }
}

/// Converts an `Exn<deepwell::error::Error>` to a JSONRPC object type.
///
/// This is not a `From` implementation since, technically, `Exn<T>` is a
/// foreign type. 🙁
pub fn exn_error_to_rpc_error(exn_error: Exn<Error>) -> ErrorObjectOwned {
    // Traverse the tree until we hit the highest-level Error
    // that is not a high-level error type, or an owned error
    // from JSONRPC itself.

    #[derive(Debug)]
    enum TopError<'e> {
        CrateError(&'e Error),
        JsonrpcError(&'e ErrorObjectOwned),
    }

    fn walk<'e>(
        frame: &'e Frame,
        code_trace: &mut Vec<i32>,
        extra_data: &mut Vec<JsonValue>,
    ) -> Option<TopError<'e>> {
        let crate_error = frame.error().downcast_ref::<Error>();
        if let Some(error) = crate_error {
            // Add error code and data for history
            code_trace.push(error.code());
            extra_data.push(error.data());

            // If acceptable, return
            if !error.error_type.is_high_level() {
                return Some(TopError::CrateError(error));
            }
        }

        let jsonrpc_error = frame.error().downcast_ref::<ErrorObjectOwned>();
        match jsonrpc_error {
            Some(error) => Some(TopError::JsonrpcError(error)),
            _ => frame
                .children()
                .iter()
                .find_map(|frame| walk(frame, code_trace, extra_data)),
        }
    }

    let mut code_trace = Vec::new();
    let mut extra_data = Vec::new();
    let top_error = walk(exn_error.frame(), &mut code_trace, &mut extra_data);

    // Special case
    // If all the extra data fields are null, then just replace the list with that.
    let extra_data = if extra_data.iter().all(JsonValue::is_null) {
        None
    } else {
        Some(extra_data)
    };

    match top_error {
        // Get the top non-request crate error
        Some(TopError::CrateError(error)) => {
            let error_code = error.code();
            let message = error.summary();
            let data = match error.error_type {
                // Special case, if authentication then don't include call trace
                // See comment in auth_login in endpoints/auth.rs
                ErrorType::InvalidAuthentication => None,

                // Normal case, provide error context
                _ => Some(json!({
                    "call_trace": format!("{exn_error:?}"),
                    "code_trace": code_trace,
                    "extra": extra_data,
                })),
            };
            ErrorObjectOwned::owned(error_code, message, data)
        }

        // For JSONRPC errors, just pass them as-is
        Some(TopError::JsonrpcError(error)) => error.clone(),

        // No crate or jsonrpsee Error exists in chain,
        // just return as string.
        None => {
            let message = str!(exn_error);
            let data = json!({
                "call_trace": format!("{exn_error:?}"),
                "code_trace": code_trace,
                "extra": extra_data,
            });
            ErrorObjectOwned::owned(0, message, Some(data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::filter::FilterSummary;
    use exn::ErrorExt;
    use sea_orm::{DbErr, TransactionError};
    use serde_json::json;

    #[test]
    fn unwraps_transaction_errors_without_rewriting_inner_error() {
        let error = Error::new("page missing", ErrorType::PageNotFound).raise();
        let unwrapped = unwrap_transaction_error(TransactionError::Transaction(error));

        assert_eq!(unwrapped.code(), ErrorType::PageNotFound.code());
        assert_eq!(unwrapped.message, "page missing");
    }

    #[test]
    fn wraps_transaction_connection_errors() {
        let unwrapped = unwrap_transaction_error(TransactionError::Connection(
            DbErr::Custom(str!("offline")),
        ));

        assert_eq!(unwrapped.code(), ErrorType::DatabaseQuery.code());
        assert!(format!("{unwrapped:?}").contains("offline"));
    }

    #[test]
    fn converts_low_level_crate_error_to_rpc_error() {
        let rpc = exn_error_to_rpc_error(
            Error::new("page missing", ErrorType::PageNotFound).raise(),
        );

        assert_eq!(rpc.code(), ErrorType::PageNotFound.code());
        assert_eq!(rpc.message(), ErrorType::PageNotFound.summary());
        let data: JsonValue = serde_json::from_str(rpc.data().unwrap().get()).unwrap();
        assert_eq!(data["code_trace"], json!([ErrorType::PageNotFound.code()]),);
        assert_eq!(data["extra"], json!(null));
        assert!(
            data["call_trace"]
                .as_str()
                .unwrap()
                .contains("page missing")
        );
    }

    #[test]
    fn converts_high_level_chain_to_low_level_rpc_error() {
        let inner = Error::new("page missing", ErrorType::PageNotFound).raise();
        let exn = inner.raise(Error::new("page operation failed", ErrorType::Page));
        let rpc = exn_error_to_rpc_error(exn);

        assert_eq!(rpc.code(), ErrorType::PageNotFound.code());
        let data: JsonValue = serde_json::from_str(rpc.data().unwrap().get()).unwrap();
        assert_eq!(
            data["code_trace"],
            json!([ErrorType::Page.code(), ErrorType::PageNotFound.code()]),
        );
    }

    #[test]
    fn filter_violation_rpc_payload_and_call_trace_redact_filter_details() {
        let regex = "PRIVATE_FILTER_REGEX_472";
        let description = "ADMIN_ONLY_FILTER_DESCRIPTION_472";
        let rpc = exn_error_to_rpc_error(
            Error::new(
                "filter rejected title",
                ErrorType::FilterViolation {
                    field: "title".to_owned(),
                    value: "submitted title".to_owned(),
                    failed: vec![FilterSummary {
                        filter_id: 472,
                        regex: regex.to_owned(),
                        description: description.to_owned(),
                    }],
                },
            )
            .raise(),
        );

        let serialized = rpc.to_string();
        let data: JsonValue = serde_json::from_str(rpc.data().unwrap().get()).unwrap();

        assert_eq!(data["extra"][0]["failed_count"], 1);
        assert_eq!(data["extra"][0]["field"], "title");
        assert!(!serialized.contains(regex));
        assert!(!serialized.contains(description));
        assert!(!data["call_trace"].as_str().unwrap().contains(regex));
        assert!(!data["call_trace"].as_str().unwrap().contains(description));
    }

    #[test]
    fn omits_rpc_data_for_invalid_authentication() {
        let rpc = exn_error_to_rpc_error(
            Error::new("invalid login", ErrorType::InvalidAuthentication).raise(),
        );

        assert_eq!(rpc.code(), ErrorType::InvalidAuthentication.code());
        assert!(rpc.data().is_none());
    }

    #[test]
    fn passes_jsonrpc_error_through_high_level_chain() {
        let jsonrpc = ErrorObjectOwned::owned(1234, "jsonrpc failure", Some("data"));
        let exn = Exn::<Error>::raise_all(
            Error::new("request failed", ErrorType::Request),
            [jsonrpc.clone().raise()],
        );
        let rpc = exn_error_to_rpc_error(exn);

        assert_eq!(rpc.code(), 1234);
        assert_eq!(rpc.message(), "jsonrpc failure");
        assert_eq!(rpc.data().unwrap().get(), r#""data""#);
    }

    #[test]
    fn converts_high_level_only_chain_to_generic_rpc_error() {
        let rpc = exn_error_to_rpc_error(
            Error::new("request failed", ErrorType::Request).raise(),
        );

        assert_eq!(rpc.code(), 0);
        assert!(rpc.message().contains("request failed"));
        let data: JsonValue = serde_json::from_str(rpc.data().unwrap().get()).unwrap();
        assert_eq!(data["code_trace"], json!([ErrorType::Request.code()]));
    }
}
