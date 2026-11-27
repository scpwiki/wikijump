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
            let data = Some(json!({
                "call_trace": format!("{exn_error:?}"),
                "code_trace": code_trace,
                "extra": extra_data,
            }));
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
