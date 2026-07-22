/*
 * tests/locale.rs
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
use deepwell::error::prelude::*;
use serde_json::json;

#[tokio::test]
async fn locale_info() {
    let runner = TestRunner::setup().await;

    let info = run_endpoint!(runner, locale_info, json!(["en-gb"]));
    assert_eq!(info.language, "en");
    assert_str_eq!(info.region, Some("GB"));
    assert_eq!(info.script, None);
    assert!(info.variants.is_empty());

    let info = run_endpoint!(runner, locale_info, json!(["fr_Latn-FR-MACOS"]));
    assert_eq!(info.language, "fr");
    assert_str_eq!(info.region, Some("FR"));
    assert_str_eq!(info.script, Some("Latn"));
    assert_eq!(info.variants.len(), 1);
    assert_eq!(info.variants[0], "macos");

    let info = run_endpoint!(runner, locale_info, json!(["ja-corrections"]));
    assert_eq!(info.language, "ja");
    assert_eq!(info.region, None);
    assert_eq!(info.script, None);
    assert!(info.variants.is_empty());
}

#[tokio::test]
async fn translate_strings() {
    let runner = TestRunner::setup().await;

    // Error cases

    // No locales
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({
            "locales": [],
            "messages": {
                "license": {},
                "base-title": {"title": "foo"},
            },
        }),
    );
    assert_contains_error!(error, ErrorType::NoLocalesSpecified);
    assert_no_error!(
        error,
        ErrorType::BadRequest
            | ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    // Key in strip_message_keys but not messages
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({
            "locales": ["fr_FR"],
            "messages": {
                "license": {},
            },
            "strip_message_keys": ["base-title"],
        }),
    );
    assert_contains_error!(error, ErrorType::BadRequest);
    assert_no_error!(
        error,
        ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({
            "locales": ["fr_FR"],
            "messages": {
                "license": {},
                "base-title": {"title": "foo"},
            },
            "strip_message_keys": ["xyz-invalid-key"],
        }),
    );
    assert_contains_error!(error, ErrorType::BadRequest);
    assert_no_error!(
        error,
        ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    // No locale
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({"locales": ["xyz_US"], "messages": {"license": {}}}),
    );
    assert_contains_error!(
        error,
        // it first tries xyz_US, which doesn't exist
        // it then tries xyz, which also doesn't exist
        // and it has no more fallbacks so that is what it returns
        ErrorType::LocaleMissing { locale } if locale == "xyz",
    );
    assert_no_error!(
        error,
        ErrorType::BadRequest
            | ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    // No message key
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({"locales": ["en"], "messages": {"xyz-invalid-key": {}}}),
    );
    assert_contains_error!(
        error,
        ErrorType::LocaleMessageMissing { message_key } if message_key == "xyz-invalid-key",
    );
    assert_no_error!(
        error,
        ErrorType::BadRequest
            | ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    // No message value
    // Only attributes exist
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "error-404": {},
                "error-404.page": {},
            },
        }),
    );
    assert_contains_error!(
        error,
        ErrorType::LocaleMessageValueMissing { message_key } if message_key == "error-404",
    );
    assert_no_error!(
        error,
        ErrorType::BadRequest
            | ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageAttributeMissing { .. }
    );

    // No message attribute
    let error = run_endpoint_err!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "license.cc0": {},
                "license.xyz": {},
            },
        }),
    );
    assert_contains_error!(
        error,
        ErrorType::LocaleMessageAttributeMissing { message_key, attribute }
            if message_key == "license" && attribute == "xyz",
    );
    assert_no_error!(
        error,
        ErrorType::BadRequest
            | ErrorType::LocaleInvalid { .. }
            | ErrorType::LocaleMissing { .. }
            | ErrorType::LocaleMessageMissing { .. }
            | ErrorType::LocaleMessageValueMissing { .. }
    );

    // Success cases

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "license": {},
                "license.cc0": {},
                "base-title": {"title": "foo"},
            },
        }),
    );
    assert_eq!(output.len(), 3);
    assert_str_eq!(output["license"], Some("License"));
    assert_str_eq!(output["license.cc0"], Some("Public Domain (CC0)"));
    assert_str_eq!(output["base-title"], Some("\u{2068}foo\u{2069} | Wikijump"));

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["ja-corrections", "en"],
            "messages": {
                "license": {},
            },
        }),
    );
    assert_str_eq!(output["license"], Some("License"));

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["zh", "zh_Hans", "ko"],
            "messages": {
                "alt-title": {},
                "close": {},
            },
        }),
    );
    assert_eq!(output.len(), 2);
    assert!(output.contains_key("alt-title"));
    assert!(output.contains_key("close"));

    // Testing strip_message_keys

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "base-title": {"title": "foo"},
            },
            "strip_message_keys": ["base-title"],
        }),
    );
    assert_eq!(output.len(), 1);
    assert_str_eq!(output["base-title"], Some("foo | Wikijump"));

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "base-title": {"title": "foo"},
            },
            "strip_message_keys": [],
        }),
    );
    assert_eq!(output.len(), 1);
    assert_str_eq!(output["base-title"], Some("\u{2068}foo\u{2069} | Wikijump"));

    let output = run_endpoint!(
        runner,
        translate_strings,
        json!({
            "locales": ["en"],
            "messages": {
                "license.cc0": {},
                "license": {},
            },
            "strip_message_keys": ["license", "license.cc0"],
        }),
    );
    assert_eq!(output.len(), 2);
    assert_str_eq!(output["license"], Some("License"));
    assert_str_eq!(output["license.cc0"], Some("Public Domain (CC0)"));
}
