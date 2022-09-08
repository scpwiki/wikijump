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
use crate::constants::AUTOMATIC_USER_ID;
use crate::services::page::{CreatePage, PageService};
use crate::services::site::{CreateSite, CreateSiteOutput, SiteService};
use crate::services::user::{CreateUser, CreateUserOutput, UserService};
use crate::services::ServiceContext;
use anyhow::Result;
use sea_orm::TransactionTrait;

pub async fn seed(state: &ApiServerState) -> Result<()> {
    tide::log::info!("Running seeder...");

    // Set up context
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::from_raw(state, &txn);

    // Load seed data
    let SeedData { users, site_pages } = SeedData::load(&state.config.seeder_path)?;

    // Seed user data
    for user in users {
        // Hash password
        // TODO
        let password = user.password;

        // TODO Create with slug

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

        assert_eq!(user_id, user.id, "Specified user ID doesn't match created");
    }

    // Seed site data
    for SitePages { site, pages } in site_pages {
        let CreateSiteOutput { site_id, slug } = SiteService::create(
            &ctx,
            CreateSite {
                slug: site.slug,
                name: site.name,
                subtitle: site.tagline, // TODO
                description: site.description,
                locale: site.locale,
            },
        )
        .await?;

        for page in pages {
            PageService::create(
                &ctx,
                site_id,
                CreatePage {
                    wikitext: page.wikitext,
                    title: page.title,
                    alt_title: page.alt_title,
                    slug: page.slug,
                    revision_comments: str!(""),
                    user_id: AUTOMATIC_USER_ID,
                },
            )
            .await?;
        }
    }

    txn.commit().await?;
    Ok(())
}
