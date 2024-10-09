/*
 * database/seeder/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use self::data::SeedData;
use crate::api::ServerState;
use crate::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use crate::models::sea_orm_active_enums::AliasType;
use crate::services::alias::{AliasService, CreateAlias};
use crate::services::domain::{CreateCustomDomain, DomainService};
use crate::services::filter::{CreateFilter, FilterService};
use crate::services::page::{CreatePage, PageService};
use crate::services::site::{CreateSite, CreateSiteOutput, SiteService};
use crate::services::user::{CreateUser, CreateUserOutput, UpdateUserBody, UserService};
use crate::services::ServiceContext;
use crate::types::{ProvidedValue, Reference};
use anyhow::Result;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement, TransactionTrait,
};
use std::borrow::Cow;
use std::collections::HashMap;

pub async fn seed(state: &ServerState) -> Result<()> {
    info!("Running seeder...");

    // Set up context
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(state, &txn);

    // Ensure seeding has not already been done
    if UserService::exists(&ctx, Reference::from(ADMIN_USER_ID)).await? {
        info!("Seeding has already been done");
        return Ok(());
    }

    // Reset sequences so IDs are consistent
    restart_sequence(&txn, "user_user_id_seq").await?;
    restart_sequence(&txn, "page_page_id_seq").await?;
    restart_sequence(&txn, "site_site_id_seq").await?;

    // Load seed data
    info!(
        "Loading seed data from {}",
        state.config.seeder_path.display(),
    );

    let SeedData {
        users,
        sites,
        pages,
        filters,
    } = SeedData::load(&state.config.seeder_path)?;

    let mut user_aliases = Vec::new();

    // Seed user data
    for user in users {
        info!("Creating seed user '{}' (ID {})", user.name, user.id);

        // Create users
        let CreateUserOutput { user_id, slug } = UserService::create(
            &ctx,
            CreateUser {
                user_type: user.user_type,
                name: user.name,
                email: user.email,
                password: user.password.unwrap_or_default(),
                locales: user.locales,
                bypass_filter: true,
                bypass_email_verification: true,
            },
        )
        .await?;

        UserService::update(
            &ctx,
            Reference::Id(user_id),
            UpdateUserBody {
                email_verified: ProvidedValue::Set(true),
                real_name: ProvidedValue::Set(user.real_name),
                gender: ProvidedValue::Set(user.gender),
                birthday: ProvidedValue::Set(user.birthday),
                location: ProvidedValue::Set(user.location),
                biography: ProvidedValue::Set(user.biography),
                user_page: ProvidedValue::Set(user.user_page),
                ..Default::default()
            },
        )
        .await?;

        // Queue up aliases to add
        //
        // This has to be a separate list, since the alias is "added"
        // by the "system" user, which may not have been created yet.
        user_aliases.push((user_id, user.aliases));

        debug!("User created with slug '{}'", slug);
        assert_eq!(user_id, user.id, "Specified user ID doesn't match created");
        assert_eq!(slug, user.slug, "Specified user slug doesn't match created");
    }

    // Seed user alias data
    for (user_id, aliases) in user_aliases {
        info!("Creating aliases for user ID {user_id}");

        for alias in aliases {
            info!("Creating user alias '{alias}'");

            AliasService::create(
                &ctx,
                CreateAlias {
                    slug: alias,
                    alias_type: AliasType::User,
                    target_id: user_id,
                    created_by: SYSTEM_USER_ID,
                    bypass_filter: true,
                },
            )
            .await?;
        }
    }

    // Seed site data
    let mut site_ids = HashMap::new();
    for site in sites {
        info!("Creating seed site '{}' (slug {})", site.name, site.slug);

        let CreateSiteOutput { site_id, slug, .. } = SiteService::create(
            &ctx,
            CreateSite {
                slug: site.slug,
                name: site.name,
                tagline: site.tagline,
                description: site.description,
                default_page: site.default_page,
                layout: site.layout,
                locale: site.locale,
            },
        )
        .await?;

        for site_alias in site.aliases {
            info!("Creating site alias '{site_alias}'");

            AliasService::create(
                &ctx,
                CreateAlias {
                    slug: site_alias,
                    alias_type: AliasType::Site,
                    target_id: site_id,
                    created_by: SYSTEM_USER_ID,
                    bypass_filter: true,
                },
            )
            .await?;
        }

        for domain in site.domains {
            info!("Creating site domain '{domain}'");

            DomainService::create_custom(&ctx, CreateCustomDomain { site_id, domain })
                .await?;
        }

        site_ids.insert(slug, site_id);
    }

    // Seed page data
    for (site_slug, pages) in pages {
        info!("Creating pages in site {site_slug}");
        let site_id = site_ids[&site_slug];

        for page in pages {
            info!("Creating page '{}' (slug {})", page.title, page.slug);

            let model = PageService::create(
                &ctx,
                CreatePage {
                    site_id,
                    wikitext: page.wikitext,
                    title: page.title,
                    alt_title: page.alt_title,
                    slug: page.slug,
                    layout: None,
                    revision_comments: str!(""),
                    user_id: SYSTEM_USER_ID,
                    bypass_filter: true,
                },
            )
            .await?;

            // TODO add attribution with site_user as author
            let _ = model;
        }
    }

    // Seed filters
    for filter in filters {
        // Get site (if any)
        // Also do logging
        let site_id = match filter.site_slug {
            Some(slug) => {
                let site = {
                    let slug: Cow<str> = Cow::Borrowed(&slug);
                    SiteService::get(&ctx, Reference::Slug(slug)).await?
                };

                info!(
                    "Creating site filter '{}' ('{}') for site '{}' (ID {})",
                    filter.regex, filter.description, slug, site.site_id,
                );

                Some(site.site_id)
            }
            None => {
                info!(
                    "Creating platform filter '{}' ('{}')",
                    filter.regex, filter.description,
                );

                None
            }
        };

        FilterService::create(
            &ctx,
            site_id,
            CreateFilter {
                affects_user: filter.user,
                affects_email: filter.email,
                affects_page: filter.page,
                affects_file: filter.file,
                affects_forum: filter.forum,
                case_sensitive: filter.case_sensitive,
                regex: filter.regex,
                description: filter.description,
            },
        )
        .await?;
    }

    // After all seeding, modify ID sequences so that they exhibit Wikidot compatibility.
    //
    // This property means that no valid Wikidot ID for a class of object
    // can ever also be a valid Wikijump ID for that same class of object.
    // We do this by putting the start ID for new Wikijump IDs well above
    // what the Wikidot value is likely to reach by the time the project
    // hits production.
    //
    // Some classes of object are not assigned compatibility IDs, either
    // because the ID value does not matter, is unused, or is not exposed.
    //
    // See https://scuttle.atlassian.net/browse/WJ-964

    restart_sequence_with(&txn, "user_user_id_seq", 10000000).await?;
    restart_sequence_with(&txn, "site_site_id_seq", 6000000).await?;
    restart_sequence_with(&txn, "page_page_id_seq", 3000000000).await?;
    restart_sequence_with(&txn, "page_revision_revision_id_seq", 3000000000).await?;
    restart_sequence_with(&txn, "page_category_category_id_seq", 100000000).await?;

    /*
     * TODO: tables which don't exist yet:
     * restart_sequence_with(&txn, < forum category seq >, 9000000).await?;
     * restart_sequence_with(&txn, < forum thread seq >, 30000000).await?;
     * restart_sequence_with(&txn, < forum post seq >, 7000000).await?;
     */

    txn.commit().await?;
    info!("Finished running seeder.");
    Ok(())
}

async fn restart_sequence(
    txn: &DatabaseTransaction,
    sequence_name: &'static str,
) -> Result<()> {
    debug!("Restarting sequence {sequence_name}");

    // SAFETY: We cannot parameterize the sequence name here, so we have to use format!()
    //         However, by requiring that sequence_name be &'static str, we ensure that it
    //         is only applied to hardcoded values and never used for runtime values
    //         (such as ones entered by an external, untrusted user).
    run_query(txn, format!("ALTER SEQUENCE {sequence_name} RESTART")).await
}

async fn restart_sequence_with(
    txn: &DatabaseTransaction,
    sequence_name: &'static str,
    new_start_value: i64,
) -> Result<()> {
    debug!("Restarting sequence {sequence_name} to start with {new_start_value}",);
    assert!(
        new_start_value > 0,
        "New sequence start value {new_start_value} is not positive",
    );

    // SAFETY: Like the above, except we have to bake in the integer value too because
    //         I cannot figure out Sea-ORM's raw query parameterization.
    //
    //         This is unfortunate, but no positive integer value can result in a SQL injection,
    //         and like the sequence name, this is a hardcoded value.
    run_query(
        txn,
        format!("ALTER SEQUENCE {sequence_name} RESTART WITH {new_start_value}"),
    )
    .await
}

async fn run_query(txn: &DatabaseTransaction, sql: String) -> Result<()> {
    txn.execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await?;

    Ok(())
}
