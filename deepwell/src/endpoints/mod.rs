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
    pub use crate::error::prelude::*;
    pub use crate::runtime::ServerState;
    pub use crate::services::{
        AliasService, BlobService, CaddyService, CategoryService, DomainService,
        FileRevisionService, FileService, LinkService, MessageReportService,
        MessageService, MfaService, PageLockService, PageRevisionService, PageService,
        ParentService, RelationService, RenderService, ScoreService, ServiceContext,
        SessionService, SettingsService, SiteService, TextBlockService, TextService,
        UserService, ViewService, VoteService,
    };
    pub use jsonrpsee::types::params::Params;
    pub use std::convert::TryFrom;
}

pub mod all {
    pub use super::auth::*;
    pub use super::basic_error::*;
    pub use super::blob::*;
    pub use super::category::*;
    pub use super::domain::*;
    pub use super::email::*;
    pub use super::file::*;
    pub use super::file_revision::*;
    pub use super::forum_post::*;
    pub use super::health::*;
    pub use super::import::*;
    pub use super::info::*;
    pub use super::link::*;
    pub use super::locale::*;
    pub use super::message::*;
    pub use super::misc::*;
    pub use super::page::*;
    pub use super::page_attribution::*;
    pub use super::page_lock::*;
    pub use super::page_revision::*;
    pub use super::parent::*;
    pub use super::role::*;
    pub use super::routing::*;
    pub use super::site::*;
    pub use super::site_ban::*;
    pub use super::site_member::*;
    pub use super::text::*;
    pub use super::text_block::*;
    pub use super::user::*;
    pub use super::user_bot::*;
    pub use super::view::*;
    pub use super::vote::*;
}

pub mod auth;
pub mod basic_error;
pub mod blob;
pub mod category;
pub mod domain;
pub mod email;
pub mod file;
pub mod file_revision;
pub mod forum_post;
pub mod health;
pub mod import;
pub mod info;
pub mod link;
pub mod locale;
pub mod message;
pub mod misc;
pub mod page;
pub mod page_attribution;
pub mod page_lock;
pub mod page_revision;
pub mod parent;
pub mod role;
pub mod routing;
pub mod site;
pub mod site_ban;
pub mod site_member;
pub mod text;
pub mod text_block;
pub mod user;
pub mod user_bot;
pub mod view;
pub mod vote;
