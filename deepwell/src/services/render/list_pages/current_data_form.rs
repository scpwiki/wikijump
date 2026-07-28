/*
 * services/render/list_pages/current_data_form.rs
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

//! Current-page data-form values used by ListPages module-head selectors.

use super::data_forms::{
    ListPagesDataFormDefinition, load_list_pages_data_form_definitions,
};
use super::substitution::substitute_list_pages_current_data_form_variables;
use crate::error::prelude::Result;
use crate::services::page_query::parse_static_wikidot_data_form_values;
use crate::services::{CategoryService, PageRevisionService, ServiceContext};
use crate::types::Reference;
use ftml::data::PageInfo;
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(in crate::services::render) struct ListPagesCurrentDataFormContext {
    values: BTreeMap<String, String>,
    definition: ListPagesDataFormDefinition,
}

pub(in crate::services::render) fn current_data_form_list_pages_head<'a>(
    head: &'a str,
    context: Option<&ListPagesCurrentDataFormContext>,
) -> Cow<'a, str> {
    context
        .and_then(|context| {
            substitute_list_pages_current_data_form_variables(
                head,
                &context.values,
                &context.definition,
            )
        })
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(head))
}

pub(in crate::services::render) async fn load_current_page_data_form_context(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    current_page_id: Option<i64>,
    page_info: &PageInfo<'_>,
) -> Result<Option<ListPagesCurrentDataFormContext>> {
    let Some(current_page_id) = current_page_id else {
        return Ok(None);
    };
    let category_slug = page_info
        .category
        .as_ref()
        .map(|category| category.as_ref())
        .unwrap_or("_default");
    let Some(category) = CategoryService::get_optional(
        ctx,
        current_site_id,
        Reference::Slug(Cow::Borrowed(category_slug)),
    )
    .await?
    else {
        return Ok(None);
    };
    if category.template_page_id.is_none() {
        return Ok(None);
    }
    let definitions =
        load_list_pages_data_form_definitions(ctx, std::slice::from_ref(&category))
            .await?;
    let Some(definition) = definitions.get(&category.category_id).cloned() else {
        return Ok(None);
    };
    let wikitext = PageRevisionService::get_wikitext_optional(
        ctx,
        current_site_id,
        Reference::Id(current_page_id),
    )
    .await?
    .unwrap_or_default();
    Ok(Some(ListPagesCurrentDataFormContext {
        values: parse_static_wikidot_data_form_values(&wikitext),
        definition,
    }))
}
