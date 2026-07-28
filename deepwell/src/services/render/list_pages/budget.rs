/*
 * services/render/list_pages/budget.rs
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

//! Shared ListPages expansion budget state.

use super::super::service::{
    MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER, MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER,
};

#[derive(Debug)]
pub(in crate::services::render) struct ListPagesExpansionBudget {
    pub(in crate::services::render) remaining_content_modules: usize,
    pub(in crate::services::render) remaining_content_rows: usize,
}

impl ListPagesExpansionBudget {
    pub(in crate::services::render) fn new() -> Self {
        Self {
            remaining_content_modules: MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER,
            remaining_content_rows: MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER,
        }
    }

    pub(in crate::services::render) fn try_start_content_module(&mut self) -> bool {
        if self.remaining_content_modules == 0 {
            return false;
        }
        self.remaining_content_modules -= 1;
        true
    }

    pub(in crate::services::render) fn remaining_content_rows(&self) -> usize {
        self.remaining_content_rows
    }

    pub(in crate::services::render) fn can_expand_content_rows(
        &self,
        rows: usize,
    ) -> bool {
        rows <= self.remaining_content_rows
    }

    pub(in crate::services::render) fn consume_content_rows(&mut self, rows: usize) {
        debug_assert!(self.can_expand_content_rows(rows));
        self.remaining_content_rows = self.remaining_content_rows.saturating_sub(rows);
    }
}
