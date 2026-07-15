use super::*;

#[test]
fn indexes_wikidot_and_rendered_html_literal_regions() {
    let source = concat!(
        "outside\n",
        "[[code]]\ncode-example\n[[/code]]\n",
        "@@escaped-example@@\n",
        "[!-- comment-example --]\n",
        "[[html]]\nhtml-example\n[[/html]]\n",
        "> [[raw]]\n> raw-example\n> [[/raw]]\n",
        "<pre>pre-example</pre>\n",
        r#"<div class="code"><div>panel-example</div></div>"#,
    );
    let index = LiteralRegionIndex::new(source);

    assert!(!index.contains(source.find("outside").unwrap()));
    for needle in [
        "code-example",
        "escaped-example",
        "comment-example",
        "html-example",
        "raw-example",
        "pre-example",
        "panel-example",
    ] {
        assert!(index.contains(source.find(needle).unwrap()), "{needle}");
    }
}

#[test]
fn earlier_block_literal_owns_inline_delimiters() {
    let source = concat!(
        "[[code]]\n@@\n[!--\n[[/code]]\n",
        "[!-- @@ --]\n",
        "@@[!--@@\n",
        "following-live-text",
    );
    let index = LiteralRegionIndex::new_wikidot_syntax(source);

    assert!(index.contains(source.find("@@\n[!--").unwrap()));
    assert!(!index.contains(source.find("following-live-text").unwrap()));
}

#[test]
fn earlier_comment_owns_block_delimiters() {
    for block in ["code", "html", "math", "raw"] {
        let source =
            format!("[!--before\n[[{block}]]\nafter--]\n[[/{block}]]\nfollowing",);
        let index = LiteralRegionIndex::new_wikidot_syntax(&source);
        let block_open = format!("[[{block}]]");
        let block_close = format!("[[/{block}]]");

        assert!(index.contains(source.find(block_open.as_str()).unwrap()));
        assert!(!index.contains(source.find(block_close.as_str()).unwrap()));
        assert!(!index.contains(source.find("following").unwrap()));
    }
}

#[test]
fn module_recognition_preserves_literal_html_and_tag_interiors() {
    let source = concat!(
        "[[module Members]]\n",
        "[[div data-module=\"[[module NewPage]]\"]]body[[/div]]\n",
        "<div data-module=\"[[module Clone]]\">body</div>\n",
        "<pre>[[module Members]]</pre>\n",
        "<!-- [[module NewPage]] -->\n",
        "[[code]]\n[[module Clone]]\n[[/code]]\n",
    );
    let index = LiteralRegionIndex::new_wikidot_module_recognition(source);

    assert!(!index.contains(source.find("[[module Members]]").unwrap()));
    for module in ["[[module NewPage]]", "[[module Clone]]"] {
        for (offset, _) in source.match_indices(module) {
            assert!(index.contains(offset), "{module} at {offset}");
        }
    }
    let pre_members = source.rfind("[[module Members]]").unwrap();
    assert!(index.contains(pre_members));
}

#[test]
fn merges_dense_sorted_range_streams_in_source_order() {
    const RANGE_COUNT: usize = 20_000;
    let left = (0..RANGE_COUNT)
        .map(|index| index * 4..index * 4 + 1)
        .collect();
    let right = (0..RANGE_COUNT)
        .map(|index| index * 4 + 2..index * 4 + 3)
        .collect();
    let merged = merge_sorted_ranges(left, right);

    assert_eq!(merged.len(), RANGE_COUNT * 2);
    assert!(merged.windows(2).all(|pair| pair[0].end < pair[1].start));
}

#[test]
fn monotone_cursor_advances_each_literal_range_at_most_once() {
    const RANGE_COUNT: usize = 20_000;
    let source = "@@x@@.".repeat(RANGE_COUNT);
    let index = LiteralRegionIndex::new_list_pages_syntax(&source);
    let mut cursor = index.monotone_cursor();
    let base = super::base_candidates::collect_base_candidates(
        &source,
        super::base_candidates::BaseCandidatePolicy::FAIL_CLOSED_RUNTIME,
    );

    for (offset, _) in source.match_indices('.') {
        assert!(
            !cursor.contains(offset),
            "offset={offset}, ranges={:?}, base_tail={:?}",
            index.ranges,
            &base[base.len().saturating_sub(4)..],
        );
    }
    assert_eq!(cursor.advances(), RANGE_COUNT);
}

#[test]
fn shared_list_pages_scanner_indexes_match_independent_builds() {
    for source in [
        concat!(
            "\n[[module css]]\n.example { content: '[[module ListPages]]'; }\n[[/module]]\n",
            "[[# anchor]]\n[[module ListPages range=\".\"]]body[[/module]]",
        ),
        concat!(
            "\t> quoted [[module ListPages]]ignored[[/module]]\r\n",
            "\r\n[[module ListPages name=\"live\"]]body[[/module]]\r\n",
        ),
    ] {
        let projection = ListPagesSourceProjection::new(source)
            .expect("fixture should require a source projection");
        let expected_direct = LiteralRegionIndex::new_list_pages_scanner_syntax(source);
        let expected_projected =
            LiteralRegionIndex::new_already_projected_list_pages_syntax(
                projection.source(),
            );
        let expected_css =
            LiteralRegionIndex::new_list_pages_downstream_css_syntax(source);
        let expected_anchors = LiteralRegionIndex::new_list_pages_anchor_syntax(source);

        let indexes =
            LiteralRegionIndex::new_list_pages_scanner_indexes(source, Some(&projection));

        assert_eq!(indexes.direct.ranges, expected_direct.ranges);
        assert_eq!(
            indexes.projected.expect("projected index").ranges,
            expected_projected.ranges,
        );
        assert_eq!(
            indexes.original_css.expect("original CSS index").ranges,
            expected_css.ranges,
        );
        assert_eq!(
            indexes
                .original_anchors
                .expect("original anchor index")
                .ranges,
            expected_anchors.ranges,
        );
    }
}

#[test]
fn handles_dense_single_kind_inline_literals() {
    const LITERAL_COUNT: usize = 20_000;

    for source in [
        "@@x@@.".repeat(LITERAL_COUNT),
        "[!--x--].".repeat(LITERAL_COUNT),
    ] {
        let index = LiteralRegionIndex::new_wikidot_syntax(&source);

        assert_eq!(index.ranges.len(), LITERAL_COUNT);
        assert!(index.contains(source.rfind('x').unwrap()));
        assert!(!index.contains(source.len() - 1));
    }
}

#[test]
fn runtime_meta_raw_owns_exactly_six_delimiter_bytes() {
    let source = "@@@@@@ live";
    let runtime = LiteralRegionIndex::new_list_pages_syntax(source);

    assert!((0..6).all(|offset| runtime.contains(offset)));
    assert!(!runtime.contains(source.find("live").unwrap()));
    assert!(
        LiteralRegionIndex::new_wikidot_syntax(source)
            .contains(source.find("live").unwrap())
    );
}

#[test]
fn color_restoration_treats_only_standalone_code_as_non_literal() {
    let source = concat!(
        r#"<code class="wj-monospace">inline-marker</code>"#,
        "\n<pre><code>pre-marker</code></pre>",
        "\n<div class=\"code\"><code>panel-marker</code></div>",
        "\n<script>script-marker</script>",
    );
    let index = LiteralRegionIndex::new_html_color_restoration(source);

    assert!(!index.contains(source.find("inline-marker").unwrap()));
    for marker in ["pre-marker", "panel-marker", "script-marker"] {
        assert!(index.contains(source.find(marker).unwrap()), "{marker}");
    }
}

#[test]
fn identifies_valid_wikidot_native_quote_lines() {
    for source in [
        "> [[module CSS]]",
        ">> [[module CSS]]",
        "> > [[module CSS]]",
        " \t>> text [[module CSS]]",
    ] {
        let offset = source.find("[[module").unwrap();
        let index = WikidotNativeQuoteIndex::new(source);
        assert!(index.contains(offset), "{source:?}");
    }
    for source in [
        ">[[module CSS]]",
        "text [[module CSS]]",
        " \t[[module CSS]]",
    ] {
        let offset = source.find("[[module").unwrap();
        let index = WikidotNativeQuoteIndex::new(source);
        assert!(!index.contains(offset), "{source:?}");
    }
}

#[test]
fn leaves_html_opening_attributes_outside_the_literal_body() {
    let source = r#"<code data-example="marker">body</code> tail"#;
    let index = LiteralRegionIndex::new(source);

    assert!(!index.contains(source.find("marker").unwrap()));
    assert!(index.contains(source.find("body").unwrap()));
    assert!(!index.contains(source.find("tail").unwrap()));
}

#[test]
fn ends_wikidot_blocks_at_the_closing_marker() {
    let source = "[[code]]\ninside\n[[/code]] [[#expr 1+1]]";
    let index = LiteralRegionIndex::new(source);

    assert!(index.contains(source.find("inside").unwrap()));
    assert!(!index.contains(source.find("[[#expr").unwrap()));
}

#[test]
fn resumes_inline_scanning_after_same_line_block_close() {
    let source = "[[code]]inside[[/code]] @@escaped@@ following";
    let index = LiteralRegionIndex::new_wikidot_syntax(source);

    assert!(index.contains(source.find("inside").unwrap()));
    assert!(index.contains(source.find("escaped").unwrap()));
    assert!(!index.contains(source.find("following").unwrap()));
}

#[test]
fn unclosed_wikidot_comments_and_blocks_extend_to_eof() {
    for source in ["live [!--comment", "live\n[[code]]\nblock"] {
        let index = LiteralRegionIndex::new_wikidot_syntax(source);

        assert!(!index.contains(source.find("live").unwrap()), "{source:?}");
        assert!(index.contains(source.len() - 1), "{source:?}");
    }
}

#[test]
fn common_index_preserves_legacy_multiline_and_unclosed_raw_ranges() {
    for source in [
        "live @@raw\nacross lines@@ following",
        "live @@unclosed\nthrough eof",
    ] {
        let common = LiteralRegionIndex::new_wikidot_syntax(source);
        let runtime = LiteralRegionIndex::new_list_pages_syntax(source);
        let needle = if source.contains("through") {
            "through"
        } else {
            "across"
        };
        let offset = source.find(needle).unwrap();

        assert!(common.contains(offset));
        assert!(!runtime.contains(offset));
    }
}

#[test]
fn shallower_quote_ends_unclosed_wikidot_literal_block() {
    let source = "> [[raw]]\n> inside\noutside";
    let index = LiteralRegionIndex::new_wikidot_syntax(source);

    assert!(index.contains(source.find("inside").unwrap()));
    assert!(!index.contains(source.find("outside").unwrap()));
}

#[test]
fn quoted_block_closes_only_at_its_opening_depth() {
    let source = concat!(
        "> [[raw]]\n",
        ">> [[/raw]]\n",
        "> inside\n",
        "> [[/raw]] @@escaped@@\n",
        "outside",
    );
    let index = LiteralRegionIndex::new_wikidot_syntax(source);

    assert!(index.contains(source.find(">> [[/raw]]").unwrap()));
    assert!(index.contains(source.find("inside").unwrap()));
    assert!(index.contains(source.find("escaped").unwrap()));
    assert!(!index.contains(source.find("outside").unwrap()));
}

#[test]
fn protection_index_includes_wikidot_and_html_tag_attributes() {
    let source = concat!(
        "outside ##red|yes##\n",
        "[[span data-value=\"##red|no]] yet##\"]]body[[/span]]\n",
        "<span title='quoted > ##red|no##'>body</span>",
    );
    let index = LiteralRegionIndex::new_wikidot_protection(source);
    assert!(!index.contains(source.find("##red|yes").unwrap()));
    for offset in source.match_indices("##red|no").map(|(offset, _)| offset) {
        assert!(index.contains(offset));
    }
}

#[test]
fn html_restoration_index_includes_tags_comments_and_raw_text() {
    let source = "marker <a title='marker'>marker</a><!-- marker --><code>marker</code>";
    let index = LiteralRegionIndex::new_html_restoration(source);
    let offsets = source
        .match_indices("marker")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert!(!index.contains(offsets[0]));
    assert!(index.contains(offsets[1]));
    assert!(!index.contains(offsets[2]));
    assert!(index.contains(offsets[3]));
    assert!(index.contains(offsets[4]));
}
