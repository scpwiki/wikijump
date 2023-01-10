/*
 * services/revision/tasks.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
pub struct RevisionTasks {
    pub render_and_update_links: bool,
    pub rerender_incoming_links: bool,
    pub rerender_outgoing_includes: bool,
    pub rerender_templates: bool,
}

impl RevisionTasks {
    /// Determine what tasks need to be performed based on the found changes.
    ///
    /// # Panics
    /// This takes a list of string changes, which is the same pattern as stored
    /// in the database. Invalid values will cause the function to panic.
    ///
    /// This should be thought of as a list of enums, but because we want to avoid
    /// the extra conversion step before this goes to the database, we're using strings.
    /// Eventually we can use the native database enum when SeaORM supports Postgres arrays.
    pub fn determine(changes: &[String]) -> Self {
        let mut tasks = RevisionTasks::default();

        for change in changes {
            match change.as_str() {
                "wikitext" => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
                "title" | "alt_title" => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_incoming_links = true;
                }
                "slug" => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_incoming_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
                "tags" => {
                    tasks.render_and_update_links = true;
                    tasks.rerender_outgoing_includes = true;
                    tasks.rerender_templates = true;
                }
                _ => panic!("Unknown change string enum value: {change}"),
            }
        }

        tasks
    }
}
