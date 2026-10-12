//! Nested ListPages boundary-protection helpers for delayed rows.

use super::{
    CompatHtmlFragments, SharedRenderCostBudget,
    find_list_pages_module_matches_with_delayed_links,
    find_list_pages_module_matches_with_delayed_links_budgeted,
};

pub(super) fn protect_nested_list_pages(
    source: &str,
    fragments: &mut CompatHtmlFragments,
    render_cost_budget: Option<&SharedRenderCostBudget>,
) -> String {
    let mut ranges = nested_list_pages_boundary_ranges(source, render_cost_budget);
    if ranges.is_empty() {
        return source.to_owned();
    }

    ranges.sort_unstable_by_key(|(_, start, _)| *start);

    let mut protected = source.to_owned();
    for (marker_index, start, end) in ranges.into_iter().rev() {
        if start == end {
            continue;
        }
        let original = source[start..end].to_owned();
        let Some(marker) =
            nested_list_pages_marker(&protected, marker_index, original.len())
        else {
            continue;
        };
        let marker = fragments.push_exact_html(marker, original);
        protected.replace_range(start..end, &marker);
    }
    protected
}

pub(super) fn nested_list_pages_boundary_ranges(
    source: &str,
    render_cost_budget: Option<&SharedRenderCostBudget>,
) -> Vec<(usize, usize, usize)> {
    let mut pending = vec![(0usize, source.len())];
    let mut ranges = Vec::new();
    while let Some((base, end)) = pending.pop() {
        let segment = &source[base..end];
        let modules = render_cost_budget.map_or_else(
            || find_list_pages_module_matches_with_delayed_links(segment),
            |budget| {
                find_list_pages_module_matches_with_delayed_links_budgeted(
                    segment, budget,
                )
            },
        );
        if render_cost_budget.is_some_and(|budget| budget.is_exhausted()) {
            return Vec::new();
        }
        for module in modules {
            let body_start = base + module.body_start;
            let body_end = body_start + module.body.len();
            ranges.push((0, base + module.start, body_start));
            ranges.push((0, body_end, base + module.end));
            // A malformed or crossing scanner match must not enqueue the same
            // source span forever. Nested protection is a compatibility guard,
            // not an alternate parser, so leave non-shrinking matches alone.
            if body_start > base && body_start < body_end && body_end <= end {
                pending.push((body_start, body_end));
            }
        }
    }
    ranges
        .into_iter()
        .enumerate()
        .map(|(index, (_, start, end))| (index, start, end))
        .collect()
}

fn nested_list_pages_marker(source: &str, index: usize, length: usize) -> Option<String> {
    let prefix = format!("WJLP{index}Z");
    if prefix.len() > length {
        return None;
    }
    let mut marker = String::with_capacity(length);
    for (offset, byte) in prefix
        .bytes()
        .chain(std::iter::repeat_n(b'X', length - prefix.len()))
        .enumerate()
    {
        if offset < prefix.len() {
            marker.push(byte as char);
        } else {
            marker.push('X');
        }
    }
    (!source.contains(&marker)).then_some(marker)
}
