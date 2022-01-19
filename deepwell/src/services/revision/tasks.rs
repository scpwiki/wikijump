/*
 * services/revision/tasks.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use super::structs::CreateRevisionBody;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::TextService;
use crate::web::ProvidedValue;

/// A representation of the updating tasks to do for a revision.
#[derive(Debug, Copy, Clone)]
pub struct RevisionTasks {
    wikitext: bool,
    links_incoming: bool,
    links_outgoing: bool,
    title: bool,
    slug: bool,
}

impl RevisionTasks {
    #[inline]
    pub fn created_page() -> Self {
        RevisionTasks {
            wikitext: true,
            links_incoming: true,
            links_outgoing: true,
            title: true,
            slug: true,
        }
    }

    pub fn determine(revision: &PageRevisionModel, changes: &CreateRevisionBody) -> Self {
        let mut tasks = RevisionTasks {
            wikitext: false,
            links_incoming: false,
            links_outgoing: false,
            title: false,
            slug: false,
        };

        if let ProvidedValue::Set(ref wikitext) = changes.wikitext {
            if revision.wikitext_hash.as_slice() != TextService::hash(wikitext).as_slice()
            {
                tasks.wikitext = true;
            }
        }

        // TODO check hidden

        if let ProvidedValue::Set(ref title) = changes.title {
            if &revision.title != title {
                tasks.title = true;
            }
        }

        if let ProvidedValue::Set(ref alt_title) = changes.alt_title {
            if &revision.alt_title != alt_title {
                tasks.title = true;
            }
        }

        if let ProvidedValue::Set(ref slug) = changes.slug {
            if &revision.slug != slug {
                tasks.slug = true;
            }
        }

        // TODO check tags

        // TODO check metadata

        tasks
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        !self.wikitext && !self.title && !self.slug
    }
}
