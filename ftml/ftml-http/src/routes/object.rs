/*
 * routes/object.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

use crate::Error;
use ftml::PageRef;

// General structs

#[derive(Deserialize, Debug)]
pub struct TextInput {
    pub text: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Response<T> {
    Result(T),
    Error(String),
}

impl<T> From<Result<T, Error>> for Response<T> {
    #[inline]
    fn from(result: Result<T, Error>) -> Response<T> {
        match result {
            Ok(item) => Response::Result(item),
            Err(error) => Response::Error(str!(error)),
        }
    }
}

// Include structs

#[derive(Serialize, Debug)]
pub struct IncludeOutput<'a> {
    pub text: String,
    pub pages: Vec<PageRef<'a>>,
}

impl<'a> From<IncludeOutput<'a>> for (String, Vec<PageRef<'a>>) {
    #[inline]
    fn from(output: IncludeOutput<'a>) -> (String, Vec<PageRef<'a>>) {
        let IncludeOutput { text, pages } = output;

        (text, pages)
    }
}
