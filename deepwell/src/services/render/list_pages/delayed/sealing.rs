//! Delayed ListPages output sealing helpers.

use super::{
    CompatHtmlFragments, DelayedInput, Error, ErrorType, GeneratedInput, GeneratedKind,
    GeneratedValue, InputSegment, ListPagesRuntimeTextRange, PageExistenceSnapshot,
    PageInfo, PendingDelayedListPagesOutput, PendingDelayedListPagesSlotValue, Range,
    RenderService, Result, SlotBindings, SlotId, TextOrigin, WikitextMode,
    WikitextSettings, parse_delayed_list, register_generated_list_pages_html,
    strip_generated_list_pages_html_markers,
};

pub(in crate::services::render) fn seal_list_pages_delayed_output(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    seal_list_pages_delayed_output_with_mode(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        page_info,
        settings,
        compat_html,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::services::render) fn seal_list_pages_delayed_output_with_mode(
    output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    block_output: bool,
) -> Result<String> {
    seal_list_pages_delayed_output_with_modes(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        delayed_html_fragments,
        page_info,
        settings,
        compat_html,
        block_output,
        false,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn seal_list_pages_delayed_output_with_modes(
    mut output: String,
    delayed_occurrences: Vec<(Range<usize>, GeneratedValue<'static>)>,
    runtime_text_ranges: Vec<ListPagesRuntimeTextRange>,
    delayed_html_fragments: Vec<CompatHtmlFragments>,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    block_output: bool,
    list_pages_inline: bool,
    page_existence: Option<&PageExistenceSnapshot>,
) -> Result<String> {
    if output.is_empty()
        && delayed_occurrences.is_empty()
        && runtime_text_ranges.is_empty()
        && delayed_html_fragments.is_empty()
    {
        return Ok(String::new());
    }
    // A non-wrapper, combined ListPages body with no generated HTML or typed
    // values is already ordinary Wikidot source. Keep it in the outer page
    // stream so surrounding text remains in the same paragraph; registering
    // the sealed `<p>` as a block would manufacture an extra paragraph around
    // an otherwise inline module.
    if !block_output
        && delayed_occurrences.is_empty()
        && runtime_text_ranges.is_empty()
        && delayed_html_fragments.is_empty()
    {
        return Ok(output);
    }
    // Static rows have no typed source ranges to preserve. Apply FTML's
    // compatibility pass here as a final guard for direct/static sealing;
    // delayed rows are preprocessed before their typed ranges are allocated
    // above, so their `%%...%%` markers remain byte-stable.
    if settings.enable_page_syntax
        && delayed_occurrences.is_empty()
        && runtime_text_ranges.is_empty()
    {
        ftml::preprocess_for_layout(&mut output, settings.layout);
    }
    if delayed_occurrences.is_empty() && runtime_text_ranges.is_empty() {
        // Static rows may already contain the narrowly generated table or
        // numbered-list HTML produced by Wikijump's ListPages compatibility
        // renderer. Protect those blocks before the mandatory FTML List-mode
        // pass so FTML does not correctly escape them as authored raw HTML.
        // Rows with typed ranges remain untouched because replacing bytes
        // here would invalidate their source ownership offsets.
        output = register_generated_list_pages_html(output, compat_html);
    }
    let generated_capacity = delayed_occurrences.len();
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

    let mut segments = Vec::with_capacity(occurrences.len() * 2 + 1);
    let mut bindings = Vec::with_capacity(generated_capacity);
    let mut cursor = 0;
    let mut generated_index = 0usize;
    for (source_range, value) in occurrences {
        if source_range.start < cursor {
            return Err(Error::new(
                "typed ListPages source ranges crossed",
                ErrorType::Render,
            )
            .into());
        }
        if cursor < source_range.start {
            segments.push(InputSegment::text(
                cursor..source_range.start,
                TextOrigin::Authored,
            ));
        }
        match value {
            PendingDelayedListPagesSlotValue::Generated(value) => {
                let id = SlotId::new(
                    u32::try_from(generated_index)
                        .expect("ListPages generated slot count is budget-bounded"),
                );
                generated_index += 1;
                let kind = match &value {
                    GeneratedValue::PageLink { .. } => GeneratedKind::PageLink,
                    GeneratedValue::TagLinks { .. } => GeneratedKind::TagLinks,
                };
                segments.push(InputSegment::generated(GeneratedInput {
                    source_range: source_range.clone(),
                    id,
                    kind,
                    occurrence: 0,
                }));
                bindings.push((id, value));
            }
            PendingDelayedListPagesSlotValue::RuntimeText(origin) => {
                segments.push(InputSegment::text(source_range.clone(), origin));
            }
        }
        cursor = source_range.end;
    }
    if cursor < output.len() {
        segments.push(InputSegment::text(
            cursor..output.len(),
            TextOrigin::Authored,
        ));
    }

    let delayed_input = DelayedInput::new(&output, segments).map_err(|error| {
        Error::new(
            format!("invalid typed ListPages stream: {error}"),
            ErrorType::Render,
        )
    })?;
    let mut list_settings =
        WikitextSettings::from_mode(WikitextMode::List, settings.layout);
    list_settings.enable_html_blocks = settings.enable_html_blocks;
    list_settings.list_pages_inline = list_pages_inline;
    let delayed_tree = parse_delayed_list(&delayed_input, page_info, &list_settings)
        .map_err(|error| {
            Error::new(
                format!("failed to parse typed ListPages stream: {error}"),
                ErrorType::Render,
            )
        })?;
    let bindings = SlotBindings::new(bindings).map_err(|error| {
        Error::new(
            format!("invalid typed ListPages bindings: {error}"),
            ErrorType::Render,
        )
    })?;
    let bound = delayed_tree.bind(&bindings).map_err(|error| {
        Error::new(
            format!("failed to bind typed ListPages stream: {error}"),
            ErrorType::Render,
        )
    })?;
    let sealed = match page_existence {
        Some(page_existence) => bound.render_html_with_page_existence(
            page_info,
            &list_settings,
            page_existence,
        ),
        None => bound.render_html(page_info, &list_settings),
    };
    let mut sealed_body = sealed.body().to_owned();
    for fragments in &delayed_html_fragments {
        sealed_body = fragments.restore(&sealed_body);
    }
    let html_blocks = sealed
        .html_blocks()
        .iter()
        .map(|html| {
            let mut html = html.to_string();
            for fragments in &delayed_html_fragments {
                html = fragments.restore(&html);
            }
            compat_html.restore(&html)
        })
        .collect::<Vec<_>>();
    sealed_body = RenderService::rewrite_wikidot_html_block_iframe_urls(
        sealed_body,
        page_info,
        &html_blocks,
    );
    sealed_body = strip_generated_list_pages_html_markers(sealed_body);
    // A residual alignment opener here belongs to one selected page summary.
    // Restore it while typed row boundaries can still keep it inside that row.
    if sealed_body.contains("[[=")
        || sealed_body.contains("[[<")
        || sealed_body.contains("[[>")
        || sealed_body.contains("[[/=")
        || sealed_body.contains("[[/<")
        || sealed_body.contains("[[/>")
    {
        sealed_body =
            RenderService::restore_residual_wikidot_alignment_markers(&sealed_body);
    }
    // Wikijump owns the fixed ListPages runtime containers and registers them
    // as trusted block fragments. Resolve those markers after FTML has parsed
    // the authored List-mode template, then protect the complete sealed block
    // for the outer page parse. Leaving nested markers inside the new block
    // fragment would intentionally prevent recursive restoration.
    sealed_body = compat_html.restore(&sealed_body);
    let delayed_block_output = !block_output && sealed_body.starts_with("\n\n");
    if !block_output {
        sealed_body = strip_single_list_pages_paragraph(sealed_body);
    }
    Ok(if block_output || delayed_block_output {
        compat_html.push_block_html_allowing_span_parent(sealed_body)
    } else {
        compat_html.push_html(sealed_body)
    })
}

pub(super) fn strip_single_list_pages_paragraph(html: String) -> String {
    let without_trailing_newline = html.strip_suffix('\n').unwrap_or(&html);
    let Some(inner) = without_trailing_newline
        .strip_prefix("<p>")
        .and_then(|value| value.strip_suffix("</p>"))
    else {
        return html;
    };
    if inner.contains("<p") || inner.contains("</p>") {
        return html;
    }
    inner.to_owned()
}

pub(in crate::services::render) fn seal_protected_list_pages_delayed_output(
    protected: &str,
    pending: PendingDelayedListPagesOutput,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    seal_protected_list_pages_delayed_output_with_mode(
        protected,
        pending,
        page_info,
        settings,
        compat_html,
        true,
    )
}

fn seal_protected_list_pages_delayed_output_with_mode(
    protected: &str,
    pending: PendingDelayedListPagesOutput,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
    block_output: bool,
) -> Result<String> {
    let PendingDelayedListPagesOutput {
        slots,
        html_fragments,
        list_pages_inline,
        page_existence,
        ..
    } = pending;
    let mut output = String::with_capacity(protected.len());
    let mut delayed_occurrences = Vec::with_capacity(slots.len());
    let mut runtime_text_ranges = Vec::with_capacity(slots.len());
    let mut cursor = 0;
    for slot in slots {
        let Some(relative_start) = protected[cursor..].find(&slot.marker) else {
            return Err(Error::new(
                "typed ListPages slot marker was lost during runtime expansion",
                ErrorType::Render,
            )
            .into());
        };
        let marker_start = cursor + relative_start;
        output.push_str(&protected[cursor..marker_start]);
        let source_start = output.len();
        output.push_str(&slot.source);
        let source_end = output.len();
        match slot.value {
            PendingDelayedListPagesSlotValue::Generated(value) => {
                delayed_occurrences.push((source_start..source_end, value));
            }
            PendingDelayedListPagesSlotValue::RuntimeText(origin) => {
                runtime_text_ranges.push(ListPagesRuntimeTextRange {
                    source_range: source_start..source_end,
                    origin,
                });
            }
        }
        cursor = marker_start + slot.marker.len();
    }
    output.push_str(&protected[cursor..]);

    seal_list_pages_delayed_output_with_modes(
        output,
        delayed_occurrences,
        runtime_text_ranges,
        html_fragments,
        page_info,
        settings,
        compat_html,
        block_output,
        list_pages_inline,
        page_existence.as_ref(),
    )
}

pub(in crate::services::render) fn seal_pending_list_pages_delayed_outputs(
    wikitext: &mut String,
    pending_outputs: Vec<PendingDelayedListPagesOutput>,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
) -> Result<()> {
    for pending in pending_outputs {
        let Some((start_marker, end_marker)) = pending.boundary_markers.clone() else {
            return Err(Error::new(
                "typed ListPages output was not bounded before runtime expansion",
                ErrorType::Render,
            )
            .into());
        };
        let Some(start) = wikitext.find(&start_marker) else {
            return Err(Error::new(
                "typed ListPages opening boundary was lost during runtime expansion",
                ErrorType::Render,
            )
            .into());
        };
        let body_start = start + start_marker.len();
        let Some(relative_end) = wikitext[body_start..].find(&end_marker) else {
            return Err(Error::new(
                "typed ListPages closing boundary was lost during runtime expansion",
                ErrorType::Render,
            )
            .into());
        };
        let body_end = body_start + relative_end;
        let block_output = pending.block_output;
        let replacement = seal_protected_list_pages_delayed_output_with_mode(
            &wikitext[body_start..body_end],
            pending,
            page_info,
            settings,
            compat_html,
            block_output,
        )?;
        let replacement_end = body_end + end_marker.len();
        wikitext.replace_range(start..replacement_end, &replacement);
    }
    Ok(())
}
