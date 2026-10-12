/*
 * services/page_query/service/filtering.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Page-query selector normalization and deferred data-form filtering.

use super::{
    BTreeMap, ColumnTrait, ComparisonOperation, Condition, DataFormSelector,
    DateSelector, DateTimeResolution, EntityTrait, Error, ErrorType, Expr,
    IncludedCategories, QueryFilter, Result, ResultExt, ScoreSelector, ServiceContext,
    SimpleExpr, Value, effective_vote_count_sql, page, page_revision,
    parse_static_wikidot_data_form_values, score_comparison_operator,
    static_wikidot_data_form_matches, text,
};

pub(super) fn wikidot_name_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len());
    let trailing_star = value.ends_with('*') && value.matches('*').count() == 1;
    for (index, character) in value.char_indices() {
        match character {
            '*' if trailing_star && index + character.len_utf8() == value.len() => {
                pattern.push('%');
            }
            '*' => pattern.push_str("\\*"),
            '?' => pattern.push('_'),
            '%' => pattern.push_str("\\%"),
            '_' => pattern.push_str("\\_"),
            '\\' => pattern.push_str("\\\\"),
            _ => pattern.push(character),
        }
    }
    pattern
}

pub(super) fn category_local_wikidot_name_patterns(
    included_categories: IncludedCategories<'_>,
    value: &str,
) -> Option<Vec<String>> {
    let IncludedCategories::List(categories) = included_categories else {
        return None;
    };
    if value.contains(':') {
        return None;
    }

    let local_pattern = wikidot_name_pattern(value);
    Some(
        categories
            .iter()
            .map(|category| {
                if category.as_ref() == "_default" {
                    local_pattern.clone()
                } else {
                    format!(
                        "{}:{local_pattern}",
                        wikidot_name_pattern(category.as_ref())
                    )
                }
            })
            .collect(),
    )
}

pub(super) fn date_selector_condition(
    column: page::Column,
    selector: DateSelector,
) -> Condition {
    match selector {
        DateSelector::FromPresent { start }
            if start == time::OffsetDateTime::UNIX_EPOCH =>
        {
            Condition::all()
        }
        DateSelector::FromPresent { start } => Condition::all().add(column.gte(start)),
        DateSelector::Span {
            timestamp,
            resolution,
            comparison,
        } => {
            let (start, end) = date_span_bounds(timestamp, resolution);
            match (comparison, end) {
                (ComparisonOperation::GreaterThan, Some(end)) => {
                    Condition::all().add(column.gte(end))
                }
                (ComparisonOperation::GreaterThan, None) => {
                    Condition::all().add(Expr::cust("FALSE"))
                }
                (ComparisonOperation::LessThan, _) => {
                    Condition::all().add(column.lt(start))
                }
                (ComparisonOperation::GreaterOrEqualThan, _) => {
                    Condition::all().add(column.gte(start))
                }
                (ComparisonOperation::LessOrEqualThan, Some(end)) => {
                    Condition::all().add(column.lt(end))
                }
                (ComparisonOperation::LessOrEqualThan, None) => {
                    Condition::all().add(column.is_not_null())
                }
                (ComparisonOperation::Equal, Some(end)) => {
                    Condition::all().add(column.gte(start)).add(column.lt(end))
                }
                (ComparisonOperation::Equal, None) => {
                    Condition::all().add(column.gte(start))
                }
                (ComparisonOperation::NotEqual, Some(end)) => {
                    Condition::any().add(column.lt(start)).add(column.gte(end))
                }
                (ComparisonOperation::NotEqual, None) => {
                    Condition::all().add(column.lt(start))
                }
            }
        }
    }
}

pub(super) fn date_span_bounds(
    timestamp: time::OffsetDateTime,
    resolution: DateTimeResolution,
) -> (time::OffsetDateTime, Option<time::OffsetDateTime>) {
    let start = match resolution {
        DateTimeResolution::Second => timestamp.replace_nanosecond(0).unwrap(),
        DateTimeResolution::Minute => timestamp
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap(),
        DateTimeResolution::Hour => timestamp
            .replace_minute(0)
            .unwrap()
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap(),
        DateTimeResolution::Day => timestamp
            .date()
            .with_time(time::Time::MIDNIGHT)
            .assume_offset(timestamp.offset()),
        DateTimeResolution::Month => {
            time::Date::from_calendar_date(timestamp.year(), timestamp.month(), 1)
                .unwrap()
                .with_time(time::Time::MIDNIGHT)
                .assume_offset(timestamp.offset())
        }
        DateTimeResolution::Year => {
            time::Date::from_calendar_date(timestamp.year(), time::Month::January, 1)
                .unwrap()
                .with_time(time::Time::MIDNIGHT)
                .assume_offset(timestamp.offset())
        }
    };
    let end = match resolution {
        DateTimeResolution::Second => start.checked_add(time::Duration::SECOND),
        DateTimeResolution::Minute => start.checked_add(time::Duration::MINUTE),
        DateTimeResolution::Hour => start.checked_add(time::Duration::HOUR),
        DateTimeResolution::Day => start.checked_add(time::Duration::DAY),
        DateTimeResolution::Month => {
            let (year, month) = if start.month() == time::Month::December {
                (start.year().saturating_add(1), time::Month::January)
            } else {
                (start.year(), start.month().next())
            };
            time::Date::from_calendar_date(year, month, 1)
                .ok()
                .map(|date| {
                    date.with_time(time::Time::MIDNIGHT)
                        .assume_offset(start.offset())
                })
        }
        DateTimeResolution::Year => time::Date::from_calendar_date(
            start.year().saturating_add(1),
            time::Month::January,
            1,
        )
        .ok()
        .map(|date| {
            date.with_time(time::Time::MIDNIGHT)
                .assume_offset(start.offset())
        }),
    };
    (start, end)
}

fn vote_selector_value(selector: &ScoreSelector) -> Value {
    Value::Double(Some(selector.score.to_f64()))
}

pub(super) fn vote_selectors_condition(selectors: &[ScoreSelector]) -> SimpleExpr {
    debug_assert!(!selectors.is_empty());
    let vote_count = effective_vote_count_sql("page.page_id");
    let conditions = selectors
        .iter()
        .enumerate()
        .map(|(index, selector)| {
            format!(
                "({vote_count})::double precision {} ${}",
                score_comparison_operator(selector.comparison),
                index + 1,
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    Expr::cust_with_values(
        conditions,
        selectors
            .iter()
            .map(vote_selector_value)
            .collect::<Vec<_>>(),
    )
}

pub(super) fn postgres_bind_placeholders(count: usize) -> String {
    (1..=count)
        .map(|index| format!("${index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) async fn load_pages_static_data_form_values(
    ctx: &ServiceContext<'_>,
    pages: &[page::Model],
) -> Result<BTreeMap<i64, BTreeMap<String, String>>> {
    let make_error = || {
        Error::new(
            "failed to load ListPages data form values",
            ErrorType::PageQuery,
        )
    };
    let revision_ids = pages
        .iter()
        .filter_map(|page| page.latest_revision_id)
        .collect::<Vec<_>>();
    if revision_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let revisions_by_id = page_revision::Entity::find()
        .filter(page_revision::Column::RevisionId.is_in(revision_ids))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|revision| (revision.revision_id, revision.wikitext_hash))
        .collect::<BTreeMap<_, _>>();

    let hashes = revisions_by_id.values().cloned().collect::<Vec<_>>();
    let text_by_hash = text::Entity::find()
        .filter(text::Column::Hash.is_in(hashes))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|text| (text.hash, text.contents))
        .collect::<BTreeMap<_, _>>();

    Ok(pages
        .iter()
        .filter_map(|page| {
            let values = page
                .latest_revision_id
                .and_then(|revision_id| revisions_by_id.get(&revision_id))
                .and_then(|hash| text_by_hash.get(hash))
                .map(|wikitext| parse_static_wikidot_data_form_values(wikitext))?;
            Some((page.page_id, values))
        })
        .collect())
}

pub(super) async fn filter_pages_by_data_form_fields(
    ctx: &ServiceContext<'_>,
    pages: Vec<page::Model>,
    selectors: &[DataFormSelector<'_>],
) -> Result<Vec<page::Model>> {
    let values_by_page_id = load_pages_static_data_form_values(ctx, &pages).await?;

    Ok(pages
        .into_iter()
        .filter(|page| {
            let values = values_by_page_id
                .get(&page.page_id)
                .cloned()
                .unwrap_or_default();

            static_wikidot_data_form_matches(&values, selectors)
        })
        .collect())
}
