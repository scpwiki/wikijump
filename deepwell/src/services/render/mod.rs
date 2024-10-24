/*
 * services/render/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

#[allow(unused_imports)]
mod prelude {
    pub use super::super::prelude::*;
    pub use super::structs::*;
    pub use ftml::{
        self,
        data::PageInfo,
        info::VERSION as FTML_VERSION,
        parsing::ParseError,
        render::html::{HtmlOutput, HtmlRender},
        render::Render,
        settings::WikitextSettings,
    };
}

mod includer;
mod service;
mod structs;

pub use self::service::RenderService;
pub use self::structs::*;
