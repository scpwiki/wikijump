/*
 * services/page_revision/tasks.rs
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

/// A representation of the updating tasks to do for a revision.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct PageRevisionTasks {
    pub render_and_update_links: bool,
    pub rerender_incoming_links: bool,
    pub rerender_outgoing_includes: bool,
    pub rerender_templates: bool,
}

impl PageRevisionTasks {
    /// Determine what tasks need to be performed based on the found changes.
    ///
    pub fn determine(changes: &[PageRevisionChange]) -> Self {
        let mut tasks = PageRevisionTasks::default();

        for change in changes {
            match change {
                PageRevisionChange::Wikitext => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
                PageRevisionChange::Title | PageRevisionChange::AltTitle => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_incoming_links = true;
                }
                PageRevisionChange::Slug => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_incoming_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
                PageRevisionChange::Tags => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
            }
        }

        tasks
    }
}
use crate::types::PageRevisionChange;
