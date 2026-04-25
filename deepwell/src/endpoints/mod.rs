/*
 * endpoints/mod.rs
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

//! Definitions of endpoints invoked by different API routes.
//!
//! The structure of the internal API is specified in `api.rs`, not here.
//! This module contains implementations of those endpoints only.
//!
//! The module should not contain core business logic of its own, which should
//! instead live in `services`. Endpoint definitions should ideally be wrappers
//! around service calls, or possibly perform modest data conversion for HTTP.

#[macro_use]
mod macros;

#[allow(unused_imports)]
mod prelude {
    pub use crate::{
        api::ServerState,
        error::prelude::*,
        services::{
            AliasService, BlobService, CaddyService, CategoryService, DomainService,
            FileRevisionService, FileService, LinkService, MessageReportService,
            MessageService, MfaService, PageRevisionService, PageService, ParentService,
            RelationService, RenderService, ScoreService, ServiceContext, SessionService,
            SettingsService, SiteService, TextBlockService, TextService, UserService,
            ViewService, VoteService,
        },
    };
    pub use jsonrpsee::types::params::Params;
    pub use std::convert::TryFrom;
}

pub mod all {
    pub use super::{
        auth::*, basic_error::*, blob::*, category::*, domain::*, email::*, file::*,
        file_revision::*, health::*, info::*, link::*, locale::*, message::*, misc::*,
        page::*, page_attribution::*, page_revision::*, parent::*, routing::*, site::*,
        site_member::*, text::*, text_block::*, user::*, user_bot::*, view::*, vote::*,
    };
}

pub mod auth;
pub mod basic_error;
pub mod blob;
pub mod category;
pub mod domain;
pub mod email;
pub mod file;
pub mod file_revision;
pub mod health;
pub mod info;
pub mod link;
pub mod locale;
pub mod message;
pub mod misc;
pub mod page;
pub mod page_attribution;
pub mod page_revision;
pub mod parent;
pub mod routing;
pub mod site;
pub mod site_member;
pub mod text;
pub mod text_block;
pub mod user;
pub mod user_bot;
pub mod view;
pub mod vote;
