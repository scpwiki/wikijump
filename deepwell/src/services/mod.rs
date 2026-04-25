/*
 * services/mod.rs
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

#![allow(unused_imports)]

//! The "services" module, providing low-level logical operations.
//!
//! Each service is named for a particular object or concept, and
//! provides several low-level methods for interacting with it.
//! This may be CRUD, or small operations which should be composed
//! into larger ones.
//!
//! As such, **all methods here are _not_ contained in transactions,**
//! the expectation is that the caller will use transactions when needed.
//! For methods which make multiple calls, they will assert that they
//! are currently in a transaction, if you are not then they will raise
//! an error.
//!
//! These methods are called as component operations either by other
//! services or by route implementations found in the `methods` module.

mod prelude {
    pub use super::context::ServiceContext;
    pub use crate::{
        config::Config,
        error::prelude::*,
        types::{Maybe, Reference},
        utils::{
            ConvertToI16, ConvertToI32, ConvertToI64, ConvertToU64, ConvertToUsize, now,
        },
    };
    pub use paste::paste;
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DeleteResult,
        EntityTrait, IntoActiveModel, JoinType, ModelTrait, PaginatorTrait, QueryFilter,
        QueryOrder, QuerySelect, RelationTrait, Set,
    };
}

#[macro_use]
mod macros;

mod context;

pub mod alias;
pub mod audit;
pub mod authentication;
pub mod authorization_token;
pub mod basic_error;
pub mod blob;
pub mod blueprint;
pub mod caddy;
pub mod category;
pub mod domain;
pub mod email;
pub mod file;
pub mod file_revision;
pub mod filter;
pub mod forum;
pub mod forum_post;
pub mod forum_post_revision;
pub mod forum_thread;
pub mod import;
pub mod job;
pub mod link;
pub mod message;
pub mod message_report;
pub mod mfa;
pub mod outdate;
pub mod page;
pub mod page_query;
pub mod page_revision;
pub mod parent;
pub mod password;
pub mod permission;
pub mod relation;
pub mod render;
pub mod role;
pub mod score;
pub mod session;
pub mod settings;
pub mod site;
pub mod text;
pub mod text_block;
pub mod user;
pub mod view;
pub mod vote;

pub use self::{
    alias::AliasService, authentication::AuthenticationService,
    authorization_token::AuthorizationTokenService, basic_error::BasicErrorService,
    blob::BlobService, blueprint::BlueprintPageService, caddy::CaddyService,
    category::CategoryService, context::ServiceContext, domain::DomainService,
    file::FileService, file_revision::FileRevisionService, filter::FilterService,
    forum::ForumService, forum_post::ForumPostService,
    forum_post_revision::ForumPostRevisionService, forum_thread::ForumThreadService,
    job::JobService, link::LinkService, message::MessageService,
    message_report::MessageReportService, mfa::MfaService, outdate::OutdateService,
    page::PageService, page_query::PageQueryService, page_revision::PageRevisionService,
    parent::ParentService, password::PasswordService, relation::RelationService,
    render::RenderService, score::ScoreService, session::SessionService,
    settings::SettingsService, site::SiteService, text::TextService,
    text_block::TextBlockService, user::UserService, view::ViewService,
    vote::VoteService,
};
