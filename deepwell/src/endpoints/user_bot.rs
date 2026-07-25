/*
 * endpoints/user_bot.rs
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

use super::prelude::*;
use crate::constants::ADMIN_USER_ID;
use crate::services::MutationAuthorization;
use crate::services::authorization_token::{AuthorizationTokenService, AuthorizedObject};
use crate::services::relation::{
    CreateSingleUserBotOwner, RelationService, RemoveUserBotOwner, UserBotMetadata,
    UserBotOwner,
};
use crate::services::user::{CreateUser, CreateUserOutput, GetUser, UpdateUserBody};
use crate::types::{Maybe, Reference, UserType};
use std::net::IpAddr;

// Structs

/// Input structure for creating a new bot user.
#[derive(Deserialize, Debug, Clone)]
pub struct CreateBotUser {
    pub name: String,
    pub email: String,
    pub locales: Vec<String>,
    pub purpose: String,
    pub owners: Vec<i64>,
    pub authorization_token: String,
    pub created_by: i64,

    #[serde(flatten)]
    pub metadata: UserBotMetadata,

    #[serde(default)]
    pub bypass_filter: bool,

    #[serde(default)]
    pub bypass_email_verification: bool,

    pub ip_address: IpAddr,
}

/// Input structure for adding new bot owners.
#[derive(Deserialize, Debug, Clone)]
pub struct CreateBotUserOwners {
    pub bot_user_id: i64,
    pub owners: Vec<i64>,

    #[serde(flatten)]
    pub metadata: UserBotMetadata,
    pub created_by: i64,
}

async fn require_bot_owner_or_platform_staff(
    ctx: &ServiceContext<'_>,
    bot_user_id: i64,
    submitted_user_id: i64,
    action: &str,
) -> Result<i64> {
    let actor_user_id =
        MutationAuthorization::require_matching_actor(ctx, submitted_user_id, action)?;
    if actor_user_id == ADMIN_USER_ID {
        return Ok(actor_user_id);
    }

    let is_owner = RelationService::get_owners_for_bot(ctx, bot_user_id)
        .await
        .or_raise(|| {
            Error::new(
                format!("failed to authorize {action}"),
                ErrorType::UserBotOwner,
            )
        })?
        .iter()
        .any(|owner| owner.owner_user_id == actor_user_id);
    if !is_owner {
        return Err(Error::new(
            format!("user does not have permission to {action}"),
            ErrorType::PermissionDenied,
        )
        .into());
    }
    Ok(actor_user_id)
}

// Endpoints

pub async fn bot_user_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    let CreateBotUser {
        name,
        email,
        locales,
        purpose,
        mut metadata,
        owners,
        created_by,
        authorization_token,
        bypass_filter,
        bypass_email_verification,
        ip_address,
    } = parse!(params, UserBotOwner);

    MutationAuthorization::require_matching_actor(ctx, created_by, "create a bot user")?;

    info!("Creating new bot user with name '{name}'");

    let make_error = || {
        Error::new(
            "failed to create a bot user account",
            ErrorType::UserBotOwner,
        )
    };

    AuthorizationTokenService::verify(
        ctx,
        &authorization_token,
        AuthorizedObject::BotUser,
        ip_address,
    )
    .await
    .or_raise(make_error)?;

    // Create bot user
    let output = UserService::create(
        ctx,
        CreateUser {
            user_type: UserType::Bot,
            name,
            email,
            locales,
            password: String::new(), // TODO configure user-bot password
            bypass_filter,
            bypass_email_verification,
            override_user_id: None,
            ip_address,
        },
    )
    .await
    .or_raise(make_error)?;

    // Set description, get bot user
    let bot_user_id = output.user_id;
    let bot_user = UserService::update(
        ctx,
        Reference::Id(bot_user_id),
        ip_address,
        UpdateUserBody {
            biography: Maybe::Set(Some(purpose)),
            ..Default::default()
        },
    )
    .await
    .or_raise(make_error)?;

    // Normalize metadata field
    RelationService::normalize_user_bot_metadata(&mut metadata);

    // Add bot owners
    // Owners must necessarily be Wikijump users
    debug!(
        "Adding human owners for bot user '{}' ({})",
        bot_user.name, bot_user_id,
    );
    for owner_user_id in owners {
        let owner_user = UserService::get_real(ctx, Reference::Id(owner_user_id))
            .await
            .or_raise(make_error)?;

        debug!(
            "Adding human user '{}' (ID {}) as bot owner",
            owner_user.name, owner_user_id,
        );
        RelationService::create_user_bot_owner(
            ctx,
            CreateSingleUserBotOwner {
                bot_user: &bot_user,
                owner_user: &owner_user,
                created_by,
                metadata: &metadata,
            },
        )
        .await
        .or_raise(make_error)?;
    }

    Ok(output)
}

pub async fn bot_user_get_owners(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<Vec<UserBotOwner>>> {
    let GetUser { user: reference } = parse!(params, UserBotOwner);

    let make_error = || {
        Error::new(
            "failed to get owners for a bot user",
            ErrorType::UserBotOwner,
        )
    };

    // Bot users must necessarily be Wikijump users
    let user_bot = UserService::get_real_optional(ctx, reference)
        .await
        .or_raise(make_error)?;

    match user_bot {
        None => Ok(None),
        Some(bot_user) => {
            if bot_user.user_type != UserType::Bot {
                error!(
                    "Tried to get owners for non-bot user: '{}' (type {:?})",
                    bot_user.name, bot_user.user_type,
                );
                bail!(Error::new(
                    "can only operate on bot users",
                    ErrorType::UserWrongType
                ));
            }

            let owners = RelationService::get_owners_for_bot(ctx, bot_user.user_id)
                .await
                .or_raise(make_error)?;

            Ok(Some(owners))
        }
    }
}

pub async fn bot_user_get_bots(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<UserBotOwner>> {
    let GetUser { user: reference } = parse!(params, UserBotOwner);
    let make_error = || {
        Error::new(
            "failed to get bots for a owner user",
            ErrorType::UserBotOwner,
        )
    };

    let owner_user = UserService::get_real(ctx, reference)
        .await
        .or_raise(make_error)?;

    if owner_user.user_type != UserType::Regular {
        error!(
            "Tried to get bots for non-regular user: '{}' (type {:?})",
            owner_user.name, owner_user.user_type,
        );
        bail!(Error::new(
            "can only operate on regular users",
            ErrorType::UserWrongType,
        ));
    }

    let owners = RelationService::get_bots_owned_by_user(ctx, owner_user.user_id)
        .await
        .or_raise(make_error)?;

    Ok(owners)
}

pub async fn bot_user_owner_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: CreateBotUserOwners = parse!(params, UserBotOwner);
    require_bot_owner_or_platform_staff(
        ctx,
        input.bot_user_id,
        input.created_by,
        "change bot owners",
    )
    .await?;
    info!(
        "Adding or updating bot owners for {} ({} new owners)",
        input.bot_user_id,
        input.owners.len(),
    );

    let make_error =
        || Error::new("failed to add owners for bot user", ErrorType::UserBotOwner);

    let bot_user = UserService::get_real(ctx, Reference::Id(input.bot_user_id))
        .await
        .or_raise(make_error)?;

    for owner_user_id in input.owners {
        let owner_user = UserService::get_real(ctx, Reference::Id(owner_user_id))
            .await
            .or_raise(make_error)?;

        RelationService::create_user_bot_owner(
            ctx,
            CreateSingleUserBotOwner {
                bot_user: &bot_user,
                owner_user: &owner_user,
                created_by: input.created_by,
                metadata: &input.metadata,
            },
        )
        .await
        .or_raise(make_error)?;
    }

    Ok(())
}

pub async fn bot_user_owner_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: RemoveUserBotOwner = parse!(params, UserBotOwner);
    require_bot_owner_or_platform_staff(
        ctx,
        input.bot_user,
        input.removed_by,
        "remove a bot owner",
    )
    .await?;

    info!(
        "Remove bot owner ({} <- {})",
        input.bot_user, input.owner_user,
    );

    RelationService::remove_user_bot_owner(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to remove owner for bot user",
                ErrorType::UserBotOwner,
            )
        })?;

    Ok(())
}
