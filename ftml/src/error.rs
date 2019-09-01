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
use std::str::Utf8Error;
use std::ops::Deref;
use std::{
    fmt::{self, Write},
    io,
};

#[must_use = "should handle errors"]
#[derive(Debug)]
pub enum Error {
    StaticMsg(&'static str),
    Msg(String),
    Io(io::Error),
    Utf8(Utf8Error),
    Parse(ParseError),
    Remote(RemoteError),
    Fmt(fmt::Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            StaticMsg(s) => s,
            Msg(ref s) => s,
            Io(ref e) => e.description(),
            Utf8(ref e) => e.description(),
            Parse(ref e) => e.description(),
            Remote(ref s) => s,
            Fmt(ref e) => e.description(),
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::Error::*;

        match *self {
            StaticMsg(_) | Msg(_) => None,
            Io(ref e) => Some(e),
            Utf8(ref e) => Some(e),
            Parse(ref e) => Some(e),
            Remote(_) => None,
            Fmt(ref e) => Some(e),
        }
    }
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", StdError::description(self))?;

        if let Some(source) = StdError::source(self) {
            write!(f, ": {}", source)?;
        }

        Ok(())
    }
}

// Auto-conversion impls
impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Msg(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::Parse(error)
    }
}

impl From<RemoteError> for Error {
    fn from(error: RemoteError) -> Self {
        Error::Remote(error)
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error::Fmt(error)
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
