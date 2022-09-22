/*
 * database/seeder/mod.rs
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

mod data;

use self::data::{SeedData, SitePages};
use crate::api::ApiServerState;
use crate::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use crate::services::page::{CreatePage, PageService};
use crate::services::site::{CreateSite, CreateSiteOutput, SiteService};
use crate::services::user::{CreateUser, CreateUserOutput, UserService};
use crate::services::ServiceContext;
use crate::web::Reference;
use anyhow::Result;
use sea_orm::TransactionTrait;

pub async fn seed(state: &ApiServerState) -> Result<()> {
    tide::log::info!("Running seeder...");

    // Set up context
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::from_raw(state, &txn);

    // Ensure seeding has not already been done
    if UserService::exists(&ctx, Reference::from(ADMIN_USER_ID)).await? {
        tide::log::info!("Seeding has already been done");
        return Ok(());
    }

    // Load seed data
    tide::log::debug!(
        "Loading seed data from {}",
        state.config.seeder_path.display(),
    );

    let SeedData { users, site_pages } = SeedData::load(&state.config.seeder_path)?;

    // Seed user data
    for user in users {
        tide::log::info!("Creating seed user '{}' (ID {})", user.name, user.id);

        // Hash password
        // TODO
        let password = user.password;

        // TODO Create user aliases
        let _ = user.aliases;

        // TODO modify during user refactor
        let CreateUserOutput { user_id, slug } = UserService::create(
            &ctx,
            CreateUser {
                username: user.name,
                email: user.email,
                password,
                language: Some(user.locale),
            },
        )
        .await?;

        tide::log::debug!("User created with slug '{}'", slug);
        assert_eq!(user_id, user.id, "Specified user ID doesn't match created");
    }

    // Seed site data
    for SitePages { site, pages } in site_pages {
        tide::log::info!("Creating seed site '{}' (slug {})", site.name, site.slug);

        let CreateSiteOutput { site_id, slug: _ } = SiteService::create(
            &ctx,
            CreateSite {
                slug: site.slug,
                name: site.name,
                tagline: site.tagline,
                description: site.description,
                locale: site.locale,
            },
        )
        .await?;

        for page in pages {
            tide::log::info!("Creating page '{}' (slug {})", page.title, page.slug);

            PageService::create(
                &ctx,
                site_id,
                CreatePage {
                    wikitext: page.wikitext,
                    title: page.title,
                    alt_title: page.alt_title,
                    slug: page.slug,
                    revision_comments: str!(""),
                    user_id: SYSTEM_USER_ID,
                },
            )
            .await?;
        }
    }

    txn.commit().await?;
    Ok(())
}
