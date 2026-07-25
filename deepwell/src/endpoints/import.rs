/*
 * endpoints/import.rs
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
use crate::services::import::{
    ImportPage, ImportPageOutput, ImportService, ImportSite, ImportSiteOutput,
    ImportUser, ImportUserOutput,
};

pub async fn import_wikidot_user(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<ImportUserOutput> {
    let input: ImportUser = parse!(params, DatabaseImport);
    info!("Importing Wikidot user ID {}", input.user_id);
    ImportService::add_user(ctx, input).await.or_raise(|| {
        Error::new("failed to import wikidot user", ErrorType::DatabaseImport)
    })
}

pub async fn import_wikidot_site(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<ImportSiteOutput> {
    let input: ImportSite = parse!(params, DatabaseImport);
    info!("Importing Wikidot site ID {}", input.site_id);
    ImportService::add_site(ctx, input).await.or_raise(|| {
        Error::new("failed to import wikidot site", ErrorType::DatabaseImport)
    })
}

pub async fn import_wikidot_page(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<ImportPageOutput> {
    let input: ImportPage = parse!(params, DatabaseImport);
    info!("Importing Wikidot page ID {}", input.page_id);
    ImportService::add_page(ctx, input).await.or_raise(|| {
        Error::new("failed to import wikidot page", ErrorType::DatabaseImport)
    })
}
