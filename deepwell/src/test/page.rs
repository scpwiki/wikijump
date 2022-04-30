/*
 * test/page.rs
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
async fn exists() -> Result<()> {
    run_test! {{
        let env = TestEnvironment::setup().await?;

        macro_rules! check {
            ($slug:expr, $exists:expr $(,)?) => {
                let path = format!("/page/{WWW_SITE_ID}/slug/{}", $slug);
                let expected_status = if $exists { StatusCode::NoContent } else { StatusCode::NotFound };
                let actual_status = env.head(path)?.recv().await?;
                assert_eq!(
                    actual_status,
                    expected_status,
                    "Actual HTTP status doesn't match expect",
                );
            };
        }

        check!("start", true);
        check!("xyz", false);
        check!("system:members", true);
        check!("system:xyz", false);

        Ok(())
    }}
}

#[async_std::test]
async fn create() -> Result<()> {
    run_test! {{
        let env = TestEnvironment::setup().await?;

        let (output, status) = env
            .post(format!("/page/{WWW_SITE_ID}"))?
            .body_json(json!({
                "wikitext": "Page contents",
                "title": "Test page!",
                "altTitle": null,
                "slug": "test",
                "revisionComments": "Create page",
                "userId": ADMIN_USER_ID,
            }))?
            .recv_json()
            .await?;

        println!("-- {:#?}", output);
        println!("-- {:#?}", status);

        Ok(())
    }}
}
