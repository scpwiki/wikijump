/*
 * endpoints/site_member.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::models::interaction::Model as InteractionModel;
use crate::services::interaction::{CreateSiteMember, GetSiteMember, RemoveSiteMember};

pub async fn membership_get(
    state: ServerState,
    params: Params<'static>,
) -> Result<InteractionModel> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let input: GetSiteMember = params.parse()?;
    let output = InteractionService::get_site_member(&ctx, input).await?;
    txn.commit().await?;
    Ok(output)
}

pub async fn membership_set(state: ServerState, params: Params<'static>) -> Result<()> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let input: CreateSiteMember = params.parse()?;
    InteractionService::create_site_member(&ctx, input).await?;
    txn.commit().await?;
    Ok(())
}

pub async fn membership_delete(
    state: ServerState,
    params: Params<'static>,
) -> Result<InteractionModel> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let input: RemoveSiteMember = params.parse()?;
    let output = InteractionService::remove_site_member(&ctx, input).await?;
    txn.commit().await?;
    Ok(output)
}
