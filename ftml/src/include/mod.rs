/*
 * include/mod.rs
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

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[grammar = "include/grammar.pest"]
struct Parser;

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct PageRef<'t> {
    site: Option<Cow<'t, str>>,
    page: Cow<'t, str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncludeRef<'t> {
    page: PageRef<'t>,
    variables: HashMap<Cow<'t, str>, Cow<'t, str>>,
}

pub trait Includer<'t> {
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> HashMap<PageRef<'t>, Cow<'t, str>>;

    fn no_such_include(&mut self) -> Cow<'t, str>;
}

pub fn include<'t>(
    log: &slog::Logger,
    text: &'t mut String,
    includer: &mut dyn Includer<'t>,
) {
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "include",
        "text" => str!(text),
    ));

    info!(
        log,
        "Finding and replacing all instances of include blocks in text"
    );

    todo!()
}
