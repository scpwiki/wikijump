/*
 * error.rs
 *
 * wikidot-html - Library to convert Wikidot syntax into HTML
 * Copyright (c) 2019 Ammon Smith for Project Foundation
 *
 * wikidot-html is available free of charge under the terms of the MIT
 * License. You are free to redistribute and/or modify it under those
 * terms. It is distributed in the hopes that it will be useful, but
 * WITHOUT ANY WARRANTY. See the LICENSE file for more details.
 *
 */

use std::error::Error as StdError;
use std::{io, fmt::{self, Write}};
use std::str::Utf8Error;

#[must_use]
#[derive(Debug)]
pub enum Error {
    StaticMsg(&'static str),
    Msg(String),
    Io(io::Error),
    Utf8(Utf8Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            StaticMsg(s) => s,
            Msg(ref s) => s,
            Io(ref e) => e.description(),
            Utf8(ref e) => e.description(),
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::Error::*;

        match *self {
            StaticMsg(_) | Msg(_) => None,
            Io(ref e) => Some(e),
            Utf8(ref e) => Some(e),
        }
    }
}

impl Into<String> for Error {
    fn into(self) -> String {
        if let Error::Msg(string) = self {
            string
        } else {
            let mut string = String::new();
            write!(&mut string, "{}", &self).unwrap();
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
