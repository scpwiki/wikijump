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

use super::prelude::*;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::TextService;
use crate::web::ProvidedValue;

/// A representation of the updating tasks to do for a revision.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct RevisionTasks {
    pub render_and_update_links: bool,
    pub rename: bool,
    pub rerender_incoming_links: bool,
    pub rerender_outgoing_includes: bool,
    pub rerender_templates: bool,
}

impl RevisionTasks {
    pub fn determine(revision: &PageRevisionModel, changes: &CreateRevisionBody) -> Self {
        let mut tasks = RevisionTasks::default();

        if let ProvidedValue::Set(ref wikitext) = changes.wikitext {
            if revision.wikitext_hash.as_slice() != TextService::hash(wikitext).as_slice()
            {
                tasks.render_and_update_links = true;
                tasks.rerender_outgoing_includes = true;
                tasks.rerender_templates = true;
            }
        }

        // Don't need to check changes.hidden

        if let ProvidedValue::Set(ref title) = changes.title {
            if &revision.title != title {
                tasks.render_and_update_links = true;
                tasks.rerender_incoming_links = true;
            }
        }

        if let ProvidedValue::Set(ref alt_title) = changes.alt_title {
            if &revision.alt_title != alt_title {
                tasks.render_and_update_links = true;
                tasks.rerender_incoming_links = true;
            }
        }

        if let ProvidedValue::Set(ref slug) = changes.slug {
            if &revision.slug != slug {
                tasks.render_and_update_links = true;
                tasks.rename = true;
                tasks.rerender_incoming_links = true;
                tasks.rerender_outgoing_includes = true;
                tasks.rerender_templates = true;
            }
        }

        if let ProvidedValue::Set(ref tags) = changes.tags {
            if !string_list_equals_json(&revision.tags, tags) {
                tasks.render_and_update_links = true;
                tasks.rerender_outgoing_includes = true;
                tasks.rerender_templates = true;
            }
        }

        if let ProvidedValue::Set(ref metadata) = changes.metadata {
            if &revision.metadata != metadata {
                // TODO
                tasks.render_and_update_links = true;
            }
        }

        tasks
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        !self.render_and_update_links
            && !self.rename
            && !self.rerender_incoming_links
            && !self.rerender_outgoing_includes
            && !self.rerender_templates
    }
}

/*
 * TODO: Tasks for other page changes:
 *
 * page file change:
 * - render
 * - rerender_outgoing_includes
 * - outdate page cache
 */
