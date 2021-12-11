/*
 * methods/user.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use crate::utils::replace_in_place;
use self::users::Entity as User;
use super::prelude::*;
use wikidot_normalize::normalize;

pub async fn user_get(req: ApiRequest) -> ApiResponse {
    let reference = ItemReference::try_from(&req)?;
    let db = &req.state().database;

    let user = match reference {
        ItemReference::Id(id) => User::find_by_id(id).one(db).await?,
        ItemReference::Slug(slug) => {
            User::find()
                .filter(users::Column::Slug.eq(slug))
                .one(db)
                .await?
        }
    };

    match user {
        Some(user) => {
            // This includes fields like the password hash.
            //
            // For now this is fine, but depending on what
            // we want the usage of the API to be, we may
            // want to filter out fields.
            let body = Body::from_json(&user)?;
            Ok(body.into())
        }
        None => Err(Error::from_str(StatusCode::NotFound, "")),
    }
}

pub async fn user_post(_req: ApiRequest) -> ApiResponse {
    // returns ()
    todo!()
}

pub async fn user_put(_req: ApiRequest) -> ApiResponse {
    // returns ()
    todo!()
}

pub async fn user_delete(_req: ApiRequest) -> ApiResponse {
    // returns ()
    todo!()
}

fn get_user_slug(username: &str) -> String {
    let mut slug = str!(username);
    normalize(&mut slug);
    replace_in_place(&mut slug, ":", "");
    slug
}
