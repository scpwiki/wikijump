/*
 * services/render/list_pages/data_forms.rs
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

//! Wikidot data-form metadata used by ListPages template variables.

use std::collections::BTreeMap;

use crate::error::prelude::Result;
use crate::models::page_category;
use crate::services::ServiceContext;
use crate::services::page_revision::PageRevisionService;

#[derive(Debug, Clone, Default)]
pub(in crate::services::render) struct ListPagesDataFormDefinition {
    pub(in crate::services::render) fields:
        BTreeMap<String, ListPagesDataFormFieldDefinition>,
}

#[derive(Debug, Clone, Default)]
pub(in crate::services::render) struct ListPagesDataFormFieldDefinition {
    pub(in crate::services::render) label: String,
    pub(in crate::services::render) hint: String,
    pub(in crate::services::render) field_type: Option<String>,
    pub(in crate::services::render) values: BTreeMap<String, String>,
}

pub(in crate::services::render) async fn load_list_pages_data_form_definitions(
    ctx: &ServiceContext<'_>,
    categories: &[page_category::Model],
) -> Result<BTreeMap<i64, ListPagesDataFormDefinition>> {
    let mut templates_by_site = BTreeMap::<i64, Vec<i64>>::new();
    let mut category_templates = Vec::<(i64, i64, i64)>::new();
    for category in categories {
        let Some(template_page_id) = category.template_page_id else {
            continue;
        };
        templates_by_site
            .entry(category.site_id)
            .or_default()
            .push(template_page_id);
        category_templates.push((
            category.category_id,
            category.site_id,
            template_page_id,
        ));
    }
    if category_templates.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut template_wikitext = BTreeMap::<(i64, i64), Option<String>>::new();
    for (site_id, page_ids) in templates_by_site {
        let loaded =
            PageRevisionService::get_wikitext_optional_batch(ctx, site_id, &page_ids)
                .await?;
        template_wikitext.extend(
            loaded
                .into_iter()
                .map(|(page_id, wikitext)| ((site_id, page_id), wikitext)),
        );
    }

    let mut definitions = BTreeMap::new();
    for (category_id, site_id, template_page_id) in category_templates {
        if let Some(Some(wikitext)) = template_wikitext.get(&(site_id, template_page_id))
            && let Some(definition) = parse_wikidot_data_form_definition(wikitext)
        {
            definitions.insert(category_id, definition);
        }
    }

    Ok(definitions)
}

pub(in crate::services::render) fn parse_wikidot_data_form_definition(
    wikitext: &str,
) -> Option<ListPagesDataFormDefinition> {
    let start = wikitext.find("[[form]]")? + "[[form]]".len();
    let end = wikitext[start..].find("[[/form]]")? + start;
    let body = &wikitext[start..end];
    let mut definition = ListPagesDataFormDefinition::default();
    let mut in_fields = false;
    let mut current_field: Option<String> = None;
    let mut current_values_field: Option<String> = None;

    for line in body.lines() {
        let line = line.trim_end();
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if indent == 0 && trimmed == "fields:" {
            in_fields = true;
            current_field = None;
            current_values_field = None;
            continue;
        }
        if !in_fields {
            continue;
        }
        if indent == 2
            && let Some(field) = trimmed.strip_suffix(':')
            && valid_wikidot_data_form_field_name(field)
        {
            let field = field.to_owned();
            definition.fields.entry(field.clone()).or_default();
            current_field = Some(field);
            current_values_field = None;
            continue;
        }
        let Some(field) = current_field.as_deref() else {
            continue;
        };
        if indent == 4 {
            let Some((key, value)) = trimmed.split_once(':') else {
                current_values_field = None;
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "label" => {
                    if let Some(field) = definition.fields.get_mut(field) {
                        field.label = unquote_wikidot_data_form_scalar(value).to_owned();
                    }
                    current_values_field = None;
                }
                "hint" => {
                    if let Some(field) = definition.fields.get_mut(field) {
                        field.hint = unquote_wikidot_data_form_scalar(value).to_owned();
                    }
                    current_values_field = None;
                }
                "type" => {
                    if let Some(field) = definition.fields.get_mut(field) {
                        field.field_type =
                            Some(unquote_wikidot_data_form_scalar(value).to_owned());
                    }
                    current_values_field = None;
                }
                "values" if value.is_empty() => {
                    current_values_field = Some(field.to_owned());
                }
                _ => {
                    current_values_field = None;
                }
            }
            continue;
        }
        if indent >= 6
            && current_values_field.as_deref() == Some(field)
            && let Some((key, value)) = trimmed.split_once(':')
            && let Some(field) = definition.fields.get_mut(field)
        {
            field.values.insert(
                unquote_wikidot_data_form_scalar(key.trim()).to_owned(),
                unquote_wikidot_data_form_scalar(value.trim()).to_owned(),
            );
        }
    }

    Some(definition)
}

pub(in crate::services::render) fn substitute_list_pages_form_data(
    field: &str,
    values: &BTreeMap<String, String>,
    definition: Option<&ListPagesDataFormDefinition>,
) -> Option<String> {
    if let Some(value) = values.get(field) {
        let Some(field_definition) =
            definition.and_then(|definition| definition.fields.get(field))
        else {
            return Some(value.clone());
        };
        if field_definition.field_type.as_deref() == Some("select") {
            return Some(
                field_definition
                    .values
                    .get(value)
                    .cloned()
                    .unwrap_or_else(|| value.clone()),
            );
        }
        return Some(value.clone());
    }
    definition.map(|_| String::new())
}

pub(in crate::services::render) fn substitute_list_pages_form_raw(
    field: &str,
    values: &BTreeMap<String, String>,
    definition: Option<&ListPagesDataFormDefinition>,
) -> Option<String> {
    values
        .get(field)
        .cloned()
        .or_else(|| definition.map(|_| String::new()))
}

pub(in crate::services::render) fn substitute_list_pages_form_label(
    field: &str,
    definition: Option<&ListPagesDataFormDefinition>,
) -> Option<String> {
    definition.map(|definition| {
        definition
            .fields
            .get(field)
            .map(|field| field.label.clone())
            .unwrap_or_default()
    })
}

pub(in crate::services::render) fn substitute_list_pages_form_hint(
    field: &str,
    definition: Option<&ListPagesDataFormDefinition>,
) -> Option<String> {
    definition.map(|definition| {
        definition
            .fields
            .get(field)
            .filter(|field| field.field_type.as_deref() != Some("select"))
            .map(|field| field.hint.clone())
            .unwrap_or_default()
    })
}

fn valid_wikidot_data_form_field_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
        })
}

fn unquote_wikidot_data_form_scalar(value: &str) -> &str {
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if matches!((first, last), (b'\'', b'\'') | (b'"', b'"')) {
            return &value[1..value.len() - 1];
        }
    }

    value
}
