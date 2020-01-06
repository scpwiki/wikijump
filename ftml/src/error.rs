/*
 * error.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use crate::parse::ParseError;
use std::error::Error as StdError;
use std::ops::Deref;
use std::str::Utf8Error;
use std::{
    fmt::{self, Display, Write},
    io,
};

#[must_use = "should handle errors"]
#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown error: {0}")]
    StaticMsg(&'static str),

    #[error("unknown error: {0}")]
    Msg(String),

    #[error("general I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("UTF-8 parsing error: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("parsing error: {0}")]
    Parse(#[from] ParseError),

    #[error("remote error in consumer code: {0}")]
    Remote(#[from] RemoteError),

    #[error("formatting error: {0}")]
    Fmt(#[from] fmt::Error),
}

impl Into<String> for Error {
    fn into(self) -> String {
        if let Error::Msg(string) = self {
            string
        } else {
            let mut string = String::new();
            write!(&mut string, "{}", &self).expect("Formatted write to string failed");
            string
        }
    }
}

// Remote error wrapper
#[must_use = "should handle errors"]
#[derive(Debug)]
pub struct RemoteError(String);

impl RemoteError {
    #[inline]
    pub fn new(message: String) -> Self {
        RemoteError(message)
    }
}

impl StdError for RemoteError {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RemoteError {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for RemoteError {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl Into<String> for RemoteError {
    #[inline]
    fn into(self) -> String {
        self.0
    }
}
