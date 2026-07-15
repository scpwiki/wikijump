/*
 * services/render/metacomponent.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

const BEGIN_MARKER: &str = "[!-- Begin metacomponent context detection --]";
const END_MARKER: &str = "[!-- End metacomponent context detection --]";
const REVEAL_START: &str = "[[iftags +component]]-][[/iftags]]";
const REVEAL_END: &str = "[[iftags +component]][!-[[/iftags]]- --]";

/// Source ownership at the point where Wikidot's metacomponent context trick is selected.
///
/// The distinction must be made before include sources are concatenated. Once expanded, a
/// caller's tag set cannot identify which marked region belonged to the root page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MetacomponentSourceContext {
    /// The saved root page has the `component` tag, so its own documentation is visible.
    RootComponent,
    /// The saved root page is not a component definition.
    RootNonComponent,
    /// An include source, whose documentation is hidden under the documented metacomponent usage.
    Included,
}

pub(super) fn select_metacomponent_documentation(
    source: &mut String,
    context: MetacomponentSourceContext,
) {
    for (region_start, region_end) in complete_regions(source).into_iter().rev() {
        let replacement = match context {
            MetacomponentSourceContext::RootComponent => {
                reveal_documentation(&source[region_start..region_end])
            }
            MetacomponentSourceContext::RootNonComponent
            | MetacomponentSourceContext::Included => Some(String::new()),
        };

        let Some(replacement) = replacement else {
            continue;
        };
        source.replace_range(region_start..region_end, &replacement);
    }
}

fn complete_regions(source: &str) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut pending_begin = None;
    let mut in_code = false;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        let lowercase = trimmed.to_ascii_lowercase();
        if !in_code && (lowercase == "[[code]]" || lowercase.starts_with("[[code ")) {
            in_code = true;
        } else if in_code && lowercase == "[[/code]]" {
            in_code = false;
        } else if !in_code && trimmed == BEGIN_MARKER {
            // A later begin supersedes an unterminated earlier candidate. The earlier bytes remain
            // untouched, while the later independently complete region can still be selected.
            pending_begin = Some(offset);
        } else if !in_code
            && trimmed == END_MARKER
            && let Some(begin) = pending_begin.take()
        {
            regions.push((begin, offset + line.len()));
        }
        offset += line.len();
    }
    regions
}

fn reveal_documentation(region: &str) -> Option<String> {
    let start_marker = region.find(REVEAL_START)?;
    if region[start_marker + REVEAL_START.len()..].contains(REVEAL_START) {
        return None;
    }
    let content_search_start = start_marker + REVEAL_START.len();
    let content_start =
        content_search_start + region[content_search_start..].find('\n')? + 1;
    let content_end = content_start + region[content_start..].find(REVEAL_END)?;
    if region[content_end + REVEAL_END.len()..].contains(REVEAL_END) {
        return None;
    }
    let documentation = region[content_start..content_end].trim_end_matches('\n');
    Some(format!("{documentation}\n"))
}

#[cfg(test)]
mod tests {
    use super::{MetacomponentSourceContext, select_metacomponent_documentation};

    const REGION: &str = concat!(
        "[!-- Begin metacomponent context detection --]\n",
        "[!-- explanation --]\n",
        "[!-- -[[iftags +component]]-][[/iftags]]\n",
        "documentation\n",
        "[[code type=\"css\"]]\n",
        ".component { display: block; }\n",
        "[[/code]]\n",
        "[[iftags +component]][!-[[/iftags]]- --]\n",
        "[!-- End metacomponent context detection --]\n",
    );

    #[test]
    fn reveals_every_complete_root_component_region() {
        let mut source = format!("before\n{REGION}middle\n{REGION}after\n");
        select_metacomponent_documentation(
            &mut source,
            MetacomponentSourceContext::RootComponent,
        );
        assert_eq!(
            source,
            concat!(
                "before\n",
                "documentation\n[[code type=\"css\"]]\n.component { display: block; }\n[[/code]]\n",
                "middle\n",
                "documentation\n[[code type=\"css\"]]\n.component { display: block; }\n[[/code]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn removes_complete_regions_from_non_component_roots_and_includes() {
        for context in [
            MetacomponentSourceContext::RootNonComponent,
            MetacomponentSourceContext::Included,
        ] {
            let mut source = format!("before\n{REGION}after\n");
            select_metacomponent_documentation(&mut source, context);
            assert_eq!(source, "before\nafter\n");
        }
    }

    #[test]
    fn preserves_malformed_regions_and_continues_after_them() {
        let malformed = REGION.replace(
            "[[iftags +component]][!-[[/iftags]]- --]",
            "missing reveal end",
        );
        let mut source = format!("{malformed}{REGION}");
        select_metacomponent_documentation(
            &mut source,
            MetacomponentSourceContext::RootComponent,
        );
        assert!(source.contains("missing reveal end"));
        assert!(source.ends_with(
            "documentation\n[[code type=\"css\"]]\n.component { display: block; }\n[[/code]]\n",
        ));
    }

    #[test]
    fn skips_an_unterminated_begin_before_a_later_complete_region() {
        let mut source = format!(
            "[!-- Begin metacomponent context detection --]\nunterminated\n{REGION}"
        );
        select_metacomponent_documentation(
            &mut source,
            MetacomponentSourceContext::RootComponent,
        );
        assert!(source.starts_with(
            "[!-- Begin metacomponent context detection --]\nunterminated\n"
        ));
        assert!(source.ends_with(
            "documentation\n[[code type=\"css\"]]\n.component { display: block; }\n[[/code]]\n",
        ));
    }

    #[test]
    fn preserves_ambiguous_reveal_sentinels() {
        let mut source = REGION.replace(
            "documentation\n",
            "documentation\n[[iftags +component]]-][[/iftags]]\n",
        );
        let original = source.clone();
        select_metacomponent_documentation(
            &mut source,
            MetacomponentSourceContext::RootComponent,
        );
        assert_eq!(source, original);
    }

    #[test]
    fn does_not_recognize_markers_in_code_or_inline_literals() {
        let mut source = format!(
            "[[code]]\n{REGION}[[/code]]\n@@[!-- Begin metacomponent context detection --]@@\n"
        );
        let original = source.clone();
        select_metacomponent_documentation(
            &mut source,
            MetacomponentSourceContext::RootNonComponent,
        );
        assert_eq!(source, original);
    }
}
