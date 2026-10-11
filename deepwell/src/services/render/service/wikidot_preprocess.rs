//! Wikidot compatibility preprocessing before FTML parsing and include expansion.

use super::*;

impl RenderService {
    pub(in crate::services::render) fn prepare_wikidot_conditionals_for_include_expansion(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
    ) {
        resolve_unbound_include_variable_iftags(wikitext);
        if wikitext.contains("[[#") {
            *wikitext = resolve_wikidot_parser_functions_outside_list_pages(wikitext);
        }
        Self::resolve_wikidot_iftags(wikitext, page_info, preserved);
    }

    pub(super) fn normalize_wikidot_cross_closed_div_collapsibles(wikitext: &mut String) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Block {
            Div,
            Collapsible,
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Marker {
            OpenDiv,
            CloseDiv,
            OpenCollapsible,
            CloseCollapsible,
        }

        fn marker_kind(marker: &str) -> Option<Marker> {
            let marker = marker.to_ascii_lowercase();
            if marker == "[[/div]]" {
                return Some(Marker::CloseDiv);
            }
            if marker == "[[/collapsible]]" {
                return Some(Marker::CloseCollapsible);
            }
            if marker.ends_with("]]")
                && (marker == "[[div]]"
                    || marker.starts_with("[[div ")
                    || marker == "[[div_]]"
                    || marker.starts_with("[[div_ "))
            {
                return Some(Marker::OpenDiv);
            }
            if marker.ends_with("]]")
                && (marker == "[[collapsible]]" || marker.starts_with("[[collapsible "))
            {
                return Some(Marker::OpenCollapsible);
            }
            None
        }

        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
        let markers = Self::wikitext_line_ranges(wikitext)
            .into_iter()
            .filter_map(|(start, _, line)| {
                let marker = Self::trim_wikitext_line(line);
                let relative_start = line.find(marker)?;
                let marker_start = start + relative_start;
                if literal_regions.contains(marker_start) {
                    return None;
                }
                marker_kind(marker)
                    .map(|kind| (kind, marker_start..marker_start + marker.len()))
            })
            .collect::<Vec<_>>();

        let mut stack = Vec::new();
        let mut replacements = Vec::new();
        let mut index = 0usize;
        while index < markers.len() {
            let (kind, range) = &markers[index];
            match kind {
                Marker::OpenDiv => stack.push(Block::Div),
                Marker::OpenCollapsible => stack.push(Block::Collapsible),
                Marker::CloseDiv
                    if stack.ends_with(&[Block::Div, Block::Collapsible])
                        && markers.get(index + 1).is_some_and(|(next, _)| {
                            *next == Marker::CloseCollapsible
                        }) =>
                {
                    replacements.push((range.clone(), "[[/collapsible]]"));
                    replacements.push((markers[index + 1].1.clone(), "[[/div]]"));
                    stack.truncate(stack.len() - 2);
                    index += 1;
                }
                Marker::CloseDiv if stack.last() == Some(&Block::Div) => {
                    stack.pop();
                }
                Marker::CloseCollapsible if stack.last() == Some(&Block::Collapsible) => {
                    stack.pop();
                }
                Marker::CloseDiv | Marker::CloseCollapsible => {}
            }
            index += 1;
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, replacement);
        }
    }

    pub(super) fn prepare_wikidot_conditionals_before_include_expansion(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
        include_depth: usize,
    ) {
        // Keep incomplete boundaries available for adjacent caller/include
        // source while still pruning self-contained inactive gates early.
        if include_depth == 0 {
            // Nested sources were already resolved against their include callsite immediately before recursion.
            resolve_unbound_include_variable_iftags(wikitext);
        }
        if include_depth == 0 && wikitext.contains("[[#") {
            *wikitext = Self::resolve_wikidot_parser_functions_before_include_collection(
                wikitext,
            );
        }
        resolve_outermost_wikidot_iftags_before_include_expansion(
            wikitext,
            &page_info.tags,
            preserved,
        );
    }

    pub(super) fn prepare_wikidot_conditionals_before_include_expansion_for_page_preview(
        wikitext: &mut String,
        preserved: &mut CompatTextFragments,
        include_depth: usize,
    ) {
        if include_depth == 0 {
            resolve_unbound_include_variable_iftags(wikitext);
        }
        if include_depth == 0 && wikitext.contains("[[#") {
            *wikitext = Self::resolve_wikidot_parser_functions_before_include_collection(
                wikitext,
            );
        }
        resolve_outermost_wikidot_iftags_before_include_expansion_for_page_preview(
            wikitext, preserved,
        );
    }

    pub(super) fn resolve_wikidot_iftags(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
    ) {
        resolve_outermost_wikidot_iftags(wikitext, &page_info.tags, preserved);
    }

    pub(in crate::services::render) fn resolve_wikidot_parser_functions(
        value: &str,
    ) -> String {
        if !value.contains("[[#") {
            return value.to_owned();
        }
        ftml::preproc::resolve_wikidot_parser_functions(value)
    }

    pub(super) fn resolve_wikidot_parser_functions_before_include_collection(
        source: &str,
    ) -> String {
        let ranges = wikidot_include_directive_ranges(source);
        if ranges.is_empty() {
            return resolve_wikidot_parser_functions_outside_list_pages(source);
        }

        let mut fragments = CompatTextFragments::new(source);
        let mut protected = source.to_owned();
        for range in ranges.into_iter().rev() {
            let marker = fragments.push(&source[range.clone()]);
            protected.replace_range(range, &marker);
        }
        let resolved = resolve_wikidot_parser_functions_outside_list_pages(&protected);
        fragments.restore(&resolved)
    }

    pub(super) fn normalize_wikidot_div_style_url_quotes(wikitext: &mut String) {
        let mut normalized = String::with_capacity(wikitext.len());
        let mut changed = false;

        for line in wikitext.split_inclusive('\n') {
            if !line.trim_start().starts_with("[[div") || !line.contains("url(\"") {
                normalized.push_str(line);
                continue;
            }

            let mut line = line.to_owned();
            let mut search_start = 0usize;
            while let Some(open_offset) = line[search_start..].find("url(\"") {
                let open_quote = search_start + open_offset + "url(".len();
                let value_start = open_quote + 1;
                let Some(close_offset) = line[value_start..].find("\")") else {
                    break;
                };
                let close_quote = value_start + close_offset;

                line.replace_range(close_quote..close_quote + 1, "'");
                line.replace_range(open_quote..open_quote + 1, "'");
                search_start = open_quote + "url('".len();
                changed = true;
            }

            normalized.push_str(&line);
        }

        if changed {
            *wikitext = normalized;
        }
    }

    pub(super) fn protect_wikidot_unbound_include_variables(
        wikitext: &mut String,
        fragments: &mut CompatTextFragments,
    ) {
        if !wikitext.contains("{$") {
            return;
        }
        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
        let monospace_regions =
            LiteralRegionIndex::new_wikidot_monospace_syntax(wikitext);
        *wikitext = INCLUDE_VARIABLE_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let matched = captures.get(0).expect("full match");
                if literal_regions.contains(matched.start())
                    || monospace_regions.contains(matched.start())
                {
                    matched.as_str().to_owned()
                } else {
                    fragments.push(matched.as_str())
                }
            })
            .into_owned();
    }

    pub(super) fn normalize_wikidot_multiline_page_links(wikitext: &mut String) {
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new(&source);
        let mut normalized = String::with_capacity(source.len());
        let mut last = 0usize;
        let mut changed = false;

        for captures in WIKIDOT_MULTILINE_LABELED_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            normalized.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if literal_regions.contains(link_match.start()) {
                normalized.push_str(link_match.as_str());
                continue;
            }

            let Some(target) = captures
                .name("target")
                .map(|matched| matched.as_str().trim())
                .filter(|target| !target.is_empty())
            else {
                normalized.push_str(link_match.as_str());
                continue;
            };
            let Some(label) = captures
                .name("label")
                .map(|matched| Self::collapse_wikidot_inline_whitespace(matched.as_str()))
                .filter(|label| !label.is_empty())
            else {
                normalized.push_str(link_match.as_str());
                continue;
            };

            normalized.push_str(&format!("[[[{target}|{label}]]]"));
            changed = true;
        }

        if !changed {
            return;
        }

        normalized.push_str(&source[last..]);
        *wikitext = normalized;
    }

    pub(super) fn collapse_wikidot_inline_whitespace(value: &str) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub(in crate::services::render) fn remove_wikijump_table_body_wrappers(
        html: &str,
    ) -> String {
        html.replace("<tbody>", "").replace("</tbody>", "")
    }

    pub(in crate::services::render) fn remove_wikidot_compat_style_blocks(
        html: &str,
    ) -> String {
        WIKIDOT_COMPAT_STYLE_BLOCK_REGEX
            .replace_all(html, "")
            .into_owned()
    }

    pub(in crate::services::render) fn remove_wikijump_underline_wrappers(
        html: &str,
    ) -> String {
        // FTML uses semantic <s> elements for paired Wikidot --text--
        // strikethrough. Those are visible formatting, not plain wrappers.
        html.replace("<u>", "").replace("</u>", "")
    }

    pub(super) fn normalize_wikidot_multiline_includes(wikitext: &mut String) {
        let lines = Self::wikitext_line_ranges(wikitext);
        let mut replacements = Vec::new();
        let mut line_index = 0;

        while line_index < lines.len() {
            let (start, _, line) = lines[line_index];
            let trimmed = Self::trim_wikitext_line(line);
            if !Self::is_wikidot_multiline_include_head(trimmed) {
                line_index += 1;
                continue;
            }
            let image_block_include = Self::is_wikidot_image_block_include_head(trimmed);

            let mut include_lines = vec![trimmed.to_owned()];
            let mut end_line_index = line_index;
            let mut valid = true;
            while !Self::trim_wikitext_line(lines[end_line_index].2).ends_with("]]") {
                end_line_index += 1;
                if end_line_index >= lines.len() {
                    valid = false;
                    break;
                }
                let continuation = Self::trim_wikitext_line(lines[end_line_index].2);
                if !continuation.ends_with("]]")
                    && !continuation.starts_with('|')
                    && !(image_block_include
                        && Self::is_wikidot_image_block_parameter_line(continuation))
                {
                    valid = false;
                    break;
                }
                include_lines.push(continuation.to_owned());
            }

            if !valid || end_line_index >= lines.len() {
                line_index += 1;
                continue;
            }

            let (_, end, _) = lines[end_line_index];
            let mut normalized = include_lines.join(" ");
            while normalized.contains("  ") {
                normalized = normalized.replace("  ", " ");
            }
            normalized = normalized.replace(" ]]", "]]");
            normalized.push('\n');
            replacements.push((start..end, normalized));
            line_index = end_line_index + 1;
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, &replacement);
        }
    }

    pub(super) fn normalize_wikidot_alignment_markers(wikitext: &mut String) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut offset = 0;
        for line in source.split_inclusive('\n') {
            let line_body = line.strip_suffix('\n').unwrap_or(line);
            let trimmed = line_body.trim();
            let replacement = match trimmed {
                "[[=]]" => Some("[[div style=\"text-align: center;\"]]"),
                "[[<]]" | "[[&lt;]]" => Some("[[div style=\"text-align: left;\"]]"),
                "[[>]]" | "[[&gt;]]" => Some("[[div style=\"text-align: right;\"]]"),
                "[[/=]]" | "[[/<]]" | "[[/&lt;]]" | "[[/>]]" | "[[/&gt;]]" => {
                    Some("[[/div]]")
                }
                _ => None,
            };
            if let Some(replacement) = replacement {
                let marker_start = offset + line_body.find(trimmed).unwrap_or(0);
                if !Self::is_inside_wikidot_literal_region(&source, marker_start) {
                    let prefix_len = line_body.len() - line_body.trim_start().len();
                    let suffix_len = line_body.len() - line_body.trim_end().len();
                    output.push_str(&line_body[..prefix_len]);
                    output.push_str(replacement);
                    output.push_str(&line_body[line_body.len() - suffix_len..]);
                    output.push('\n');
                    offset += line.len();
                    continue;
                }
            }
            output.push_str(line);
            offset += line.len();
        }
        *wikitext = output;
    }

    pub(super) fn is_wikidot_multiline_include_head(line: &str) -> bool {
        let Some(rest) = line.strip_prefix("[[include") else {
            return false;
        };
        if !rest
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_whitespace())
        {
            return false;
        }
        let rest = rest.trim_start();
        if rest.is_empty() || rest.contains("]]") {
            return false;
        }
        let target_end = rest
            .find(|character: char| character.is_ascii_whitespace() || character == '|')
            .unwrap_or(rest.len());
        target_end > 0
    }

    fn is_wikidot_image_block_include_head(line: &str) -> bool {
        let Some(rest) = line.strip_prefix("[[include") else {
            return false;
        };
        let rest = rest.trim_start();
        let target_end = rest
            .find(|character: char| character.is_ascii_whitespace() || character == '|')
            .unwrap_or(rest.len());
        &rest[..target_end] == "component:image-block"
    }

    fn is_wikidot_image_block_parameter_line(line: &str) -> bool {
        let Some(parameter) = line.strip_suffix('|') else {
            return false;
        };
        parse_wikidot_include_argument(parameter).is_some()
    }

    pub(super) fn expand_wikidot_image_block_includes(
        wikitext: &mut String,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> Vec<PageRef> {
        Self::expand_wikidot_image_block_includes_with_provenance(
            wikitext,
            page_info,
            attachment_owner,
            None,
        )
    }

    pub(super) fn expand_wikidot_image_block_includes_with_provenance(
        wikitext: &mut String,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
        attachment_provenance: Option<&AttachmentProvenanceRegistry>,
    ) -> Vec<PageRef> {
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(&source);
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
        let mut included_pages = Vec::new();
        let mut search_start = 0;

        while let Some(captures) =
            WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX.captures(&source[search_start..])
        {
            let include_start =
                search_start + captures.get(0).expect("whole match exists").start();
            let match_end =
                search_start + captures.get(0).expect("whole match exists").end();
            let after = captures
                .name("after")
                .expect("after delimiter exists")
                .as_str();
            let (args_start, include_end) = if after == "]]" {
                (match_end - 2, match_end)
            } else {
                let args_start = match_end - after.len();
                let Some(include_end) = find_wikidot_image_block_directive_end(
                    &source,
                    match_end,
                    source.len(),
                ) else {
                    search_start = match_end;
                    continue;
                };
                (args_start, include_end)
            };

            search_start = include_end;

            if literal_regions.contains(include_start)
                || !Self::should_expand_wikidot_image_block_include(
                    captures.name("site").map(|site| site.as_str()),
                    page_info,
                )
            {
                continue;
            }

            let Some(args) = Self::parse_wikidot_include_arguments(
                &source[args_start..include_end - 2],
                attachment_provenance,
            ) else {
                continue;
            };
            let Some(name) = args.get("name") else {
                continue;
            };

            let caption = args
                .get("caption")
                .map_or("", |argument| argument.value.as_str());
            let width = args
                .get("width")
                .map_or("300px", |argument| argument.value.as_str());
            let align = args
                .get("align")
                .map_or("right", |argument| argument.value.as_str());
            let raw_link = args
                .get("link")
                .map_or("#", |argument| argument.value.as_str());
            let Some(semantic_name) = semantic_attachment_value(&name.value) else {
                continue;
            };
            if semantic_name.is_empty() {
                continue;
            }
            let image_source = match &name.attachment_owner {
                Some(owner) => {
                    if relative(semantic_name) {
                        owned_url(owner, semantic_name)
                    } else {
                        name.value.clone()
                    }
                }
                None => Self::wikidot_image_block_source(
                    semantic_name,
                    page_info,
                    attachment_owner,
                ),
            };
            let link = match args
                .get("link")
                .and_then(|argument| argument.attachment_owner.as_ref())
            {
                Some(owner) => {
                    let Some(semantic) = semantic_attachment_value(raw_link) else {
                        continue;
                    };
                    if relative(semantic) {
                        owned_url(owner, semantic)
                    } else {
                        semantic.to_owned()
                    }
                }
                None => {
                    let Some(semantic) = semantic_attachment_value(raw_link) else {
                        continue;
                    };
                    if relative(semantic) {
                        owned_url(
                            &Self::wikidot_image_block_attachment_owner(
                                page_info,
                                attachment_owner,
                            ),
                            semantic,
                        )
                    } else {
                        semantic.to_owned()
                    }
                }
            };
            let image_attribute = args
                .get("alt")
                .map(|argument| argument.value.as_str())
                .filter(|attribute| is_include_variable_name(attribute))
                .zip(args.get("alt-text"))
                .map(|(attribute, value)| {
                    format!(r#" {attribute}="{}""#, value.value.replace('"', "&quot;"),)
                })
                .unwrap_or_default();
            let link_attribute = if raw_link == "#" {
                String::new()
            } else {
                format!(" link={}", preserve_argument_quotes(raw_link, &link),)
            };

            let replacement = format!(
                concat!(
                    r#"[[div class="scp-image-block block-{align}" style="width:{width};"]]"#,
                    "\n",
                    r#"[[image {image_source}{image_attribute}{link_attribute}]]"#,
                    "\n",
                    r#"[[div class="scp-image-caption"]]"#,
                    "\n",
                    "{caption}\n",
                    "[[/div]]\n",
                    "[[/div]]"
                ),
                align = align,
                width = width,
                image_source = image_source,
                image_attribute = image_attribute,
                link_attribute = link_attribute,
                caption = caption,
            );

            Self::push_wikidot_image_block_include_refs(
                &mut included_pages,
                captures.name("site").map(|site| site.as_str()),
            );
            replacements.push((include_start..include_end, replacement));
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, &replacement);
        }

        included_pages
    }

    pub(super) fn push_wikidot_image_block_include_refs(
        included_pages: &mut Vec<PageRef>,
        site: Option<&str>,
    ) {
        included_pages.push(Self::wikidot_image_block_page_ref(
            site,
            "component:image-block",
        ));
        included_pages.push(Self::wikidot_image_block_page_ref(
            site,
            "component:image-block-base",
        ));
    }

    pub(super) fn wikidot_image_block_page_ref(
        site: Option<&str>,
        page: &str,
    ) -> PageRef {
        match site {
            Some(site) => PageRef::page_and_site(site, page),
            None => PageRef::page_only(page),
        }
    }

    pub(super) fn should_expand_wikidot_image_block_include(
        include_site: Option<&str>,
        page_info: &PageInfo<'_>,
    ) -> bool {
        page_info.site.as_ref() == "scp-wiki"
            && include_site.is_none_or(|site| site == "scp-wiki")
    }

    pub(super) fn is_inside_wikidot_code_block(source: &str, start: usize) -> bool {
        let mut in_code = false;
        for line in source[..start].lines() {
            let marker = line.trim_start().to_ascii_lowercase();
            if marker.starts_with("[[code") {
                in_code = true;
            } else if marker.starts_with("[[/code]]") {
                in_code = false;
            }
        }
        in_code
    }

    pub(super) fn is_inside_wikidot_html_block(source: &str, start: usize) -> bool {
        let mut in_html = false;
        for line in source[..start].lines() {
            let marker = line.trim_start().to_ascii_lowercase();
            if marker.starts_with("[[html") {
                in_html = true;
            } else if marker.starts_with("[[/html]]") {
                in_html = false;
            }
        }
        in_html
    }

    pub(super) fn is_inside_wikidot_escape(source: &str, start: usize) -> bool {
        source[..start].matches("@@").count() % 2 == 1
    }

    pub(in crate::services::render) fn is_inside_wikidot_literal_region(
        source: &str,
        start: usize,
    ) -> bool {
        Self::is_inside_wikidot_code_block(source, start)
            || Self::is_inside_wikidot_escape(source, start)
            || Self::is_inside_wikidot_html_block(source, start)
            || Self::is_inside_wikidot_comment(source, start)
    }

    pub(super) fn is_inside_wikidot_comment(source: &str, start: usize) -> bool {
        let before = &source[..start];
        let last_open = before.rfind("[!--");
        let last_close = before.rfind("--]");
        match (last_open, last_close) {
            (Some(open), Some(close)) => open > close,
            (Some(_), None) => true,
            _ => false,
        }
    }

    pub(super) fn mask_wikidot_literal_include_markers(wikitext: &mut String) {
        if !has_include_opening_candidate(wikitext) {
            return;
        }
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(&source);
        let mut replacements = Vec::new();

        for captures in WIKIDOT_INCLUDE_OPEN_REGEX.captures_iter(&source) {
            let keyword = captures
                .name("keyword")
                .expect("include keyword capture exists");
            if literal_regions.contains(keyword.start()) {
                replacements.push(keyword.range());
            }
        }

        for range in replacements.into_iter().rev() {
            wikitext.replace_range(range, WIKIDOT_LITERAL_INCLUDE_SENTINEL);
        }
    }

    pub(super) fn unmask_wikidot_literal_include_markers(wikitext: &mut String) {
        if wikitext.contains(WIKIDOT_LITERAL_INCLUDE_SENTINEL) {
            *wikitext = wikitext.replace(WIKIDOT_LITERAL_INCLUDE_SENTINEL, "include");
        }
    }

    pub(super) fn wikidot_image_block_source(
        name: &str,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> String {
        if name.starts_with("http://")
            || name.starts_with("https://")
            || name.starts_with('/')
        {
            return name.to_owned();
        }

        // Wikidot keeps extensionless root-page attachments on its implicit
        // attachment path, which supplies the medium resize and original-file
        // link. Qualified include owners still use the direct URL path below.
        if attachment_owner.is_none()
            && !name
                .rsplit('/')
                .next()
                .is_some_and(|file| file.contains('.'))
        {
            return name.to_owned();
        }

        let owner =
            Self::wikidot_image_block_attachment_owner(page_info, attachment_owner);

        format!(
            "http://{}.wikidot.com/local--files/{}/{}",
            owner.site_slug,
            owner.page_slug,
            percent_encode_path_segment(name),
        )
    }

    pub(super) fn wikidot_image_block_attachment_owner(
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> AttachmentOwner {
        attachment_owner
            .map(|(site, page)| AttachmentOwner {
                site_slug: site.to_owned(),
                page_slug: page.to_owned(),
            })
            .unwrap_or_else(|| {
                let page_slug = match page_info.category.as_deref() {
                    Some(category) => format!("{category}:{}", page_info.page),
                    None => page_info.page.to_string(),
                };
                AttachmentOwner {
                    site_slug: page_info.site.to_string(),
                    page_slug,
                }
            })
    }

    pub(super) fn parse_wikidot_include_arguments(
        args: &str,
        attachment_provenance: Option<&AttachmentProvenanceRegistry>,
    ) -> Option<BTreeMap<String, WikidotImageBlockArgument>> {
        let segments = split_wikidot_image_block_argument_segments(args)?;
        let mut arguments = BTreeMap::new();

        for segment in segments {
            if wikidot_include_segment_is_space(segment) {
                continue;
            }
            let argument = parse_wikidot_include_argument(segment)?;
            let (value, attachment_owner) = attachment_provenance
                .and_then(|registry| registry.decode(argument.value))
                .map_or_else(
                    || (argument.value.to_owned(), None),
                    |(value, owner)| (value.clone(), Some(owner.clone())),
                );
            let self_reference = value
                .strip_prefix("{$")
                .and_then(|value| value.strip_suffix('}'))
                == Some(argument.raw_key);
            if !self_reference {
                arguments
                    .entry(argument.raw_key.to_ascii_lowercase())
                    .or_insert(WikidotImageBlockArgument {
                        value,
                        attachment_owner,
                    });
            }
        }

        Some(arguments)
    }

    #[allow(dead_code)]
    pub(super) fn find_preview_component_separator_markers(
        wikitext: &str,
    ) -> Option<(usize, usize, usize, usize)> {
        let lines = Self::wikitext_line_ranges(wikitext);

        for include_start_line in 0..lines.len() {
            let include_start = Self::trim_wikitext_line(lines[include_start_line].2);
            if !include_start.starts_with("[[include") {
                continue;
            }

            let mut include_text = String::new();
            let mut include_end_line = include_start_line;
            loop {
                let line = lines[include_end_line].2;
                include_text.push_str(line);
                if line.contains("]]") {
                    break;
                }

                include_end_line += 1;
                if include_end_line >= lines.len() {
                    break;
                }
            }

            if !include_text
                .to_ascii_lowercase()
                .contains("component:preview")
            {
                continue;
            }

            if include_start_line == 0 || include_end_line + 1 >= lines.len() {
                continue;
            }

            let before = lines[include_start_line - 1];
            let after = lines[include_end_line + 1];
            if Self::trim_wikitext_line(before.2) == "====="
                && Self::trim_wikitext_line(after.2) == "====="
            {
                return Some((before.0, before.1, after.0, after.1));
            }
        }

        None
    }

    pub(super) fn wikitext_line_ranges(wikitext: &str) -> Vec<(usize, usize, &str)> {
        let mut lines = Vec::new();
        let mut start = 0;

        for (index, character) in wikitext.char_indices() {
            if character == '\n' {
                let end = index + character.len_utf8();
                lines.push((start, end, &wikitext[start..end]));
                start = end;
            }
        }

        if start < wikitext.len() {
            lines.push((start, wikitext.len(), &wikitext[start..]));
        }

        lines
    }

    pub(super) fn trim_wikitext_line(line: &str) -> &str {
        line.trim_end_matches(['\r', '\n']).trim()
    }
}
