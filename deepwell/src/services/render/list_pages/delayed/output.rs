/*
 * services/render/list_pages/delayed/output.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Final protection, sealing, and pending-output lifecycle for delayed ListPages rows.

use super::{
    CompatHtmlFragments, CompatTextFragments, DelayedInput, Error, ErrorType,
    GeneratedValue, InputSegment, ListPagesRuntimeTextRange, PageExistenceSnapshot,
    PageInfo, PendingDelayedListPagesOutput, PendingDelayedListPagesSlot,
    PendingDelayedListPagesSlotValue, Range, RenderService, Result,
    SharedRenderCostBudget, SlotBindings, TextOrigin, WikitextMode, WikitextSettings,
    find_list_pages_module_matches_with_delayed_links_budgeted, parse_delayed_list,
    register_generated_list_pages_html, seal_list_pages_delayed_output_with_modes,
};

pub(in crate::services::render) fn replace_recursive_list_pages_with_error(
    source: &str,
    fragments: &mut CompatHtmlFragments,
    render_cost_budget: &SharedRenderCostBudget,
) -> String {
    let modules = find_list_pages_module_matches_with_delayed_links_budgeted(
        source,
        render_cost_budget,
    );
    if modules.is_empty() || render_cost_budget.is_exhausted() {
        return source.to_owned();
    }

    let error = fragments.push_block_html_unwrapping_residual_div_prefix(
        r#"<div class="error-block">The ListPages module does not work recursively.</div>"#
            .to_owned(),
    );
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for module in modules {
        if module.start < cursor || module.preserve_original || !module.runtime_safe {
            continue;
        }
        output.push_str(&source[cursor..module.start]);
        output.push_str(&error);
        cursor = module.end;
    }
    if cursor == 0 {
        return source.to_owned();
    }
    output.push_str(&source[cursor..]);
    output
}

pub(in crate::services::render) fn seal_zero_row_list_pages_wrapper(
    output: &str,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if output.is_empty() {
        let wrapper = compat_html.push_block_html_allowing_span_parent(
            r#"<div class="list-pages-box"></div>"#.to_owned(),
        );
        return Ok(format!("\n\n{wrapper}\n\n"));
    }
    let segments = vec![InputSegment::text(0..output.len(), TextOrigin::Authored)];
    let delayed_input = DelayedInput::new(output, segments).map_err(|error| {
        Error::new(
            format!("invalid zero-row ListPages stream: {error}"),
            ErrorType::Render,
        )
    })?;
    let list_settings = WikitextSettings::from_mode(WikitextMode::List, settings.layout);
    let delayed_tree = parse_delayed_list(&delayed_input, page_info, &list_settings)
        .map_err(|error| {
            Error::new(
                format!("failed to parse zero-row ListPages stream: {error}"),
                ErrorType::Render,
            )
        })?;
    let bindings = SlotBindings::new(Vec::new()).map_err(|error| {
        Error::new(
            format!("invalid zero-row ListPages bindings: {error}"),
            ErrorType::Render,
        )
    })?;
    let bound = delayed_tree.bind(&bindings).map_err(|error| {
        Error::new(
            format!("failed to bind zero-row ListPages stream: {error}"),
            ErrorType::Render,
        )
    })?;
    let sealed = bound.render_html(page_info, &list_settings);
    let mut sealed_body = sealed.body().to_owned();
    let html_blocks = sealed
        .html_blocks()
        .iter()
        .map(|html| compat_html.restore(html))
        .collect::<Vec<_>>();
    sealed_body = RenderService::rewrite_wikidot_html_block_iframe_urls(
        sealed_body,
        page_info,
        &html_blocks,
    );
    let trimmed_source = output.trim_matches(['\r', '\n']);
    if !trimmed_source.contains(['\r', '\n'])
        && trimmed_source.ends_with(" _")
        && let Some(inner) = sealed_body
            .strip_prefix("<p>")
            .and_then(|body| body.strip_suffix("</p>"))
            .or_else(|| {
                sealed_body
                    .strip_prefix("<p>")
                    .and_then(|body| body.strip_suffix("</p>\n"))
            })
    {
        sealed_body = format!("{inner}<br>\n");
    }
    let sealed_body = compat_html.restore(&sealed_body);
    let wrapper = compat_html.push_block_html_allowing_span_parent(format!(
        r#"<div class="list-pages-box">{sealed_body}</div>"#,
    ));
    Ok(format!("\n\n{wrapper}\n\n"))
}

pub(in crate::services::render) fn protect_list_pages_delayed_output(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    compat_text: &mut CompatTextFragments,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    protect_list_pages_delayed_output_with_mode(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        compat_text,
        true,
    )
}

pub(in crate::services::render) fn protect_list_pages_delayed_output_with_mode(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    compat_text: &mut CompatTextFragments,
    block_output: bool,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    protect_list_pages_delayed_output_with_modes(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        compat_text,
        block_output,
        false,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn protect_list_pages_delayed_output_with_modes(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    compat_text: &mut CompatTextFragments,
    block_output: bool,
    list_pages_inline: bool,
    page_existence: Option<PageExistenceSnapshot>,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    let mut occurrences = delayed_occurrences
        .into_iter()
        .map(|(range, value)| (range, PendingDelayedListPagesSlotValue::Generated(value)))
        .chain(runtime_text_ranges.into_iter().map(|range| {
            (
                range.source_range,
                PendingDelayedListPagesSlotValue::RuntimeText(range.origin),
            )
        }))
        .collect::<Vec<_>>();
    occurrences.sort_by_key(|(range, _)| range.start);

    let mut protected = String::with_capacity(output.len());
    let mut slots = Vec::with_capacity(occurrences.len());
    let mut cursor = 0;
    for (source_range, value) in occurrences {
        if source_range.start < cursor || source_range.end > output.len() {
            return Err(Error::new(
                "typed ListPages slot escaped or crossed its generated output",
                ErrorType::Render,
            )
            .into());
        }
        let Some(source) = output.get(source_range.clone()) else {
            return Err(Error::new(
                "typed ListPages slot did not align to UTF-8 boundaries",
                ErrorType::Render,
            )
            .into());
        };
        protected.push_str(&output[cursor..source_range.start]);
        let marker = compat_text.push("");
        protected.push_str(&marker);
        slots.push(PendingDelayedListPagesSlot {
            marker,
            source: source.to_owned(),
            value,
        });
        cursor = source_range.end;
    }
    protected.push_str(&output[cursor..]);

    Ok((
        protected,
        Some(PendingDelayedListPagesOutput {
            slots,
            html_fragments: delayed_html_fragments,
            block_output,
            list_pages_inline,
            boundary_markers: None,
            page_existence,
        }),
    ))
}

#[allow(clippy::too_many_arguments)]
pub(in crate::services::render) fn finish_or_defer_list_pages_delayed_output(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    defer_for_include_expansion: bool,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    compat_text: &mut CompatTextFragments,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    finish_or_defer_list_pages_delayed_output_with_mode(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        defer_for_include_expansion,
        page_info,
        settings,
        compat_html,
        compat_text,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::services::render) fn finish_or_defer_list_pages_delayed_output_with_mode(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    defer_for_include_expansion: bool,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    compat_text: &mut CompatTextFragments,
    block_output: bool,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    finish_or_defer_list_pages_delayed_output_with_modes(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        defer_for_include_expansion,
        page_info,
        settings,
        compat_html,
        compat_text,
        block_output,
        false,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::services::render) fn finish_or_defer_list_pages_delayed_output_with_modes(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    defer_for_include_expansion: bool,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    compat_text: &mut CompatTextFragments,
    block_output: bool,
    list_pages_inline: bool,
    page_existence: Option<&PageExistenceSnapshot>,
) -> Result<(String, Option<PendingDelayedListPagesOutput>)> {
    // Every executed ListPages body belongs to FTML's List mode, including a
    // wholly authored/static row. Parsing only rows with typed values here
    // makes otherwise identical templates depend on whether they happen to
    // reference a runtime scalar, and loses line-start constructs beside the
    // generated outer containers. Include-bearing output is protected until
    // expansion, then sealed through this same path.
    if defer_for_include_expansion {
        let (protected, pending) = protect_list_pages_delayed_output_with_modes(
            output,
            delayed_occurrences,
            runtime_text_ranges,
            delayed_html_fragments,
            compat_text,
            block_output,
            list_pages_inline,
            page_existence.cloned(),
        )?;
        Ok((
            register_generated_list_pages_html(protected, compat_html),
            pending,
        ))
    } else {
        Ok((
            seal_list_pages_delayed_output_with_modes(
                output,
                delayed_occurrences,
                runtime_text_ranges,
                delayed_html_fragments,
                page_info,
                settings,
                compat_html,
                block_output,
                list_pages_inline,
                page_existence,
            )?,
            None,
        ))
    }
}

pub(in crate::services::render) fn wrap_pending_list_pages_delayed_output(
    output: &mut String,
    pending: &mut PendingDelayedListPagesOutput,
    compat_text: &mut CompatTextFragments,
) {
    let start = compat_text.push("");
    let end = compat_text.push("");
    output.insert_str(0, &start);
    output.push_str(&end);
    pending.boundary_markers = Some((start, end));
}

pub(in crate::services::render) fn restore_pending_nested_list_pages(
    source: &str,
    pending_outputs: &[PendingDelayedListPagesOutput],
) -> String {
    pending_outputs
        .iter()
        .fold(source.to_owned(), |source, pending| {
            pending
                .html_fragments
                .iter()
                .fold(source, |source, fragments| {
                    fragments.restore_exact_fragments(&source)
                })
        })
}
