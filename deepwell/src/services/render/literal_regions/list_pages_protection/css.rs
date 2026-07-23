/*
 * services/render/literal_regions/list_pages_protection/css.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

mod candidates;
mod ranges;
mod syntax;

pub(super) use candidates::{
    collect_pinned_css_module_candidates, collect_pinned_css_module_candidates_with_heads,
};
pub(super) use ranges::{
    collect_downstream_css_module_ranges,
    collect_downstream_css_module_ranges_with_heads, collect_projected_css_module_ranges,
    collect_projected_css_module_ranges_with_heads,
};
