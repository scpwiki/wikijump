/*
 * web/reference.rs
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

use crate::api::ApiRequest;
use std::convert::TryFrom;
use tide::{Error, StatusCode};

#[derive(Debug, Copy, Clone)]
pub enum ItemReference<'a> {
    Id(i64),
    Slug(&'a str),
}

impl From<i64> for ItemReference<'static> {
    #[inline]
    fn from(id: i64) -> ItemReference<'static> {
        ItemReference::Id(id)
    }
}

impl<'a> From<&'a str> for ItemReference<'a> {
    #[inline]
    fn from(slug: &'a str) -> ItemReference<'a> {
        ItemReference::Slug(slug)
    }
}

impl<'a> TryFrom<&'a ApiRequest> for ItemReference<'a> {
    type Error = Error;

    fn try_from(req: &'a ApiRequest) -> Result<ItemReference<'a>, Error> {
        let value_type = req.param("type")?;
        let value = req.param("id_or_slug")?;

        match value_type {
            "slug" => {
                tide::log::debug!("Reference via slug, {}", value);

                Ok(ItemReference::Slug(value))
            }
            "id" => {
                tide::log::debug!("Reference via ID, {}", value);

                let id = value.parse()?;
                Ok(ItemReference::Id(id))
            }
            _ => Err(Error::from_str(
                StatusCode::BadRequest,
                "May only specify object by 'id' or 'slug'",
            )),
        }
    }
}
