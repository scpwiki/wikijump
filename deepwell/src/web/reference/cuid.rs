/*
 * web/reference/file.rs
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

use crate::api::ApiRequest;
use std::convert::TryFrom;
use tide::{Error, StatusCode};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CuidReference<'a> {
    Id(&'a str),
    Name(&'a str),
}

impl<'a> CuidReference<'a> {
    pub fn try_from_fields_key(
        req: &'a ApiRequest,
        value_type_key: &str,
        value_key: &str,
    ) -> Result<Self, Error> {
        let value_type = req.param(value_type_key)?;
        let value = req.param(value_key)?;

        CuidReference::try_from_fields(value_type, value)
    }

    pub fn try_from_fields(value_type: &str, value: &'a str) -> Result<Self, Error> {
        // How long a cuid string is
        const CUID_LENGTH: usize = 25;

        match value_type {
            "name" => {
                tide::log::debug!("Reference via name, {value}");
                Ok(CuidReference::Name(value))
            }
            "id" if value.len() == CUID_LENGTH => Err(Error::from_str(
                StatusCode::BadRequest,
                "CUID string is of an incorrect length",
            )),
            "id" => {
                tide::log::debug!("Reference via ID, {value}");
                Ok(CuidReference::Id(value))
            }
            _ => Err(Error::from_str(
                StatusCode::BadRequest,
                "May only specify object by 'id' or 'slug'",
            )),
        }
    }
}

impl<'a> TryFrom<&'a ApiRequest> for CuidReference<'a> {
    type Error = Error;

    #[inline]
    fn try_from(req: &'a ApiRequest) -> Result<CuidReference<'a>, Error> {
        CuidReference::try_from_fields_key(req, "type", "id_or_slug")
    }
}
