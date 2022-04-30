/*
 * test/locale.rs
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

use super::prelude::*;

#[async_std::test]
async fn locale_name() -> Result<()> {
    let env = TestEnvironment::setup().await?;

    macro_rules! check {
        ($locale:expr, $($json:tt)+ $(,)?) => {{
            let path = concat!("/locale/", $locale);
            let (output, status) = env.get(path)?.recv_json().await?;
            assert_eq!(status, StatusCode::Ok);
            assert_eq!(output, json!($($json)+));
        }};
    }

    check!("EN", {
        "language": "en",
        "region": null,
        "script": null,
        "variants": [],
    });

    check!("en-us", {
        "language": "en",
        "region": "US",
        "script": null,
        "variants": [],
    });

    check!("en_US", {
        "language": "en",
        "region": "US",
        "script": null,
        "variants": [],
    });

    check!("en-gb", {
        "language": "en",
        "region": "GB",
        "script": null,
        "variants": [],
    });

    check!("en-in", {
        "language": "en",
        "region": "IN",
        "script": null,
        "variants": [],
    });

    check!("fR", {
        "language": "fr",
        "region": null,
        "script": null,
        "variants": [],
    });

    check!("fr_ca", {
        "language": "fr",
        "region": "CA",
        "script": null,
        "variants": [],
    });

    check!("de-AT", {
        "language": "de",
        "region": "AT",
        "script": null,
        "variants": [],
    });

    check!("Pl-Latn-PL", {
        "language": "pl",
        "region": "PL",
        "script": "Latn",
        "variants": [],
    });

    Ok(())
}

#[async_std::test]
async fn message() -> Result<()> {
    let env = TestEnvironment::setup().await?;

    macro_rules! check {
        ($locale:expr, $message_key:expr, $translation:expr $(,)?) => {
            check!($locale, $message_key, $translation, {})
        };

        ($locale:expr, $message_key:expr, $translation:expr, $($json:tt)+ $(,)?) => {{
            let path = concat!("/message/", $locale, "/", $message_key);
            let (translation, status) = env
                .post(path)?
                .body_json(json!($($json)+))?
                .recv_string()
                .await?;

            assert_eq!(status, StatusCode::Ok);
            assert_eq!(translation, $translation);
        }};
    }

    check!("en", "goto-home", "Go to home page");
    // TODO add translation for another language

    check!("en", "navigated-to", "Navigated to \u{2068}jellybean\u{2069}", { "path": "jellybean" });
    // TODO add translation for another language

    Ok(())
}
