use super::*;
use crate::services::render::module_arguments::{
    module_arguments_are_complete, wikidot_list_pages_arguments,
};

#[path = "tests/stress.rs"]
mod stress;

#[test]
fn runtime_head_recognizer_matches_the_execution_regex() {
    for (head, expected) in [
        ("", true),
        (" name=\"x\"", true),
        ("name='x'other=bare", true),
        ("name=\"unterminated", true),
        ("name=\"unterminated next=bare", true),
        ("name=\"x\"wrapper=\"y\"", true),
        ("name=\"x\"wrapper=", false),
        ("1=bare", false),
        ("[!--=bare", false),
        ("name=a]b", false),
        ("limit\n=\"1\"", true),
        ("name=\"secret@site.example\" wrapper=no\"", true),
        ("name = \"secret\"wrapper=\"no\"", true),
        ("name = \"secret\\\" wrapper=no\"", true),
        (r#"order="name"" limit="1""#, true),
        (r#"tags="+scp rating="<0" separate="no""#, true),
        ("name=\u{00a0}bare", true),
        ("name=\"x\" \u{000b}", true),
        ("name=\"x\" @", false),
        ("ſ=\"x\"", false),
        ("K=\"x\"", false),
    ] {
        assert_eq!(
            runtime_regex_recognizes_entire_head(head),
            expected,
            "{head:?}",
        );
        assert_eq!(
            module_arguments_are_complete(head),
            expected,
            "execution regex: {head:?}",
        );
    }
}

#[test]
fn crossing_url_quote_rejects_trailing_unsupported_head_bytes() {
    let malformed = r#"offset="@URL|1 "created_by="Fireknight" unsupported"#;

    assert!(
        !list_pages_url_quote_crossing_head_can_execute(malformed),
        "a recovered created_by quote must end at the head boundary: {malformed:?}",
    );
    assert!(
        !list_pages_runtime_head_can_execute(malformed),
        "unsupported bytes after the recovered quote must fail closed: {malformed:?}",
    );
}

#[test]
fn runtime_head_validation_accepts_static_wildcard_selectors() {
    for head in [r#" category="*" "#, r#" category="*" tags="codex" "#] {
        assert_eq!(
            validate_module_head(head, true),
            ModuleHeadValidation::RuntimeSafe,
            "{head:?}",
        );
        assert!(list_pages_runtime_head_is_safe(head), "{head:?}");
    }
}

#[test]
fn corpus_complete_heads_that_execute_live_remain_runtime_executable() {
    for head in [
        r#"name="*" category="-nav -system -forum -admin" tags="-管理 -" order="created_at desc desc" limit="1" offset="@URL|0""#,
        r#"limit="1" created_by="@URL" tags="-hub,-návod,-české,-autor""#,
        r#"category="_default" order="name desc desc" wrapper="no" separate="no" perPage="250""#,
        "separate=\"no\" tags=\"@URL\" created_at=\"@URL\" updated_at=\"@URL\" created_by=\"@URL\" rating=\"@URL\" offset=\"@URL|0\" perPage=\"1\"\u{3000}limit=\"1\" order=\"@URL|created_at desc\" category=\"*\"",
        r#"order="random" perPage="250" limit="250""#,
    ] {
        assert!(
            list_pages_runtime_head_can_execute(head),
            "fresh anonymous Wikidot preview executed {head:?}",
        );
    }
}

#[test]
fn scanner_keeps_the_body_after_an_executable_head_with_trailing_note_text() {
    // Exact-source replay provenance: en:scp-7992:L59:B1630. Live Wikidot
    // executes the supported selectors before the prose `NOTE:` tail and
    // starts the row template immediately after this complete opener.
    let source = concat!(
        "[[module ListPages limit=\"1\" category=\"*\" ",
        "order=\"updated_at desc\" NOTE: module end is at bottom of page. ",
        "Also, if you try and import a component from another wiki inside ",
        "the module, then everything inside the module will disappear.]]\n",
        "ALPHA\n",
        "@@@@\n",
        "@@@@\n",
        "BRAVO\n",
        "[[=]]\n",
        "[[div class=\"addendum\"]]\n",
        "CHARLIE\n",
        "[[/div]]\n",
        "[[/=]]\n",
        "[[/module]]",
    );

    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].head,
        concat!(
            "limit=\"1\" category=\"*\" order=\"updated_at desc\" ",
            "NOTE: module end is at bottom of page. Also, if you try and ",
            "import a component from another wiki inside the module, then ",
            "everything inside the module will disappear.",
        ),
    );
    assert!(modules[0].body.starts_with("\nALPHA\n"), "{modules:#?}");
    assert!(modules[0].body.contains("BRAVO"), "{modules:#?}");
}

#[test]
fn scanner_keeps_generated_comment_gate_rows_inside_the_module() {
    let source = concat!(
        "[[module ListPages name=\"gate\"]]\n",
        "[[#ifexpr %%created_by_id%% < 1486450 |  | [!-- ]]\n",
        "LOW\n",
        "[!-- --]\n",
        "[[#ifexpr %%created_by_id%% > 6000000 |  | [!-- ]]\n",
        "HIGH\n",
        "[!-- --]\n",
        "[[/module]]\n",
        "[[module ListPages name=\"later\"]]LATER[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2, "later modules must remain visible");
    assert!(modules[0].body.contains("[[#ifexpr %%created_by_id%%"));
    assert!(!modules[0].body.contains("[[/module]]"));

    let literal = concat!(
        "[[code]]\n",
        "[[module ListPages name=\"literal\"]]\n",
        "[[#ifexpr %%created_by_id%% < 1486450 |  | [!-- ]]\n",
        "LOW\n",
        "[!-- --]\n",
        "[[/module]]\n",
        "[[/code]]",
    );
    assert!(
        find_list_pages_module_matches(literal).is_empty(),
        "generated-looking gates in code remain literal",
    );

    let decoy = concat!(
        "[[module ListPages name=\"gate-decoy\"]]\n",
        "[[#ifexpr %%created_by_id%% < 1486450 |  | [!-- ]]\n",
        "LOW [[/module]]\n",
        "[!-- --]\n",
        "REAL\n",
        "[[/module]]\n",
        "[[module ListPages name=\"later\"]]LATER[[/module]]",
    );
    let modules = find_list_pages_module_matches(decoy);
    assert_eq!(
        modules.len(),
        2,
        "the inactive-branch closer is not structural"
    );
    assert!(modules[0].body.contains("LOW [[/module]]"));
    assert!(modules[0].body.contains("REAL"));
}

#[test]
fn scanner_ignores_inert_prose_after_supported_list_pages_arguments() {
    // Anonymous PagePreview boundary matrix:
    // listpages-head-recovery-{prose-ascii,prose-unicode,prose-nested-block,
    // prose-crossing-quote}. The assignments and custom row execute while
    // arbitrary trailing prose remains inert.
    for head in [
        r#"limit="1" category="*" order="name" NOTE: arbitrary words can move freely."#,
        r#"limit="1" category="*" order="name" NOTA: conteúdo muda livremente."#,
        r#"limit="1" category="*" order="name" NOTE: arbitrary [[span]] words."#,
        r#"limit="1" category="*" order="name" NOTE: arbitrary "quoted words"."#,
    ] {
        let source = format!("[[module ListPages {head}]]\nALPHA\nBRAVO\n[[/module]]");
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
        assert_eq!(modules[0].head, head, "{source:?}: {modules:#?}");
        assert_eq!(
            modules[0].body, "\nALPHA\nBRAVO\n",
            "{source:?}: {modules:#?}",
        );
        assert!(
            list_pages_runtime_head_can_execute(modules[0].head),
            "{modules:#?}",
        );
    }
}

#[test]
fn scanner_recovers_final_unclosed_quotes_as_default_template_modules() {
    // Anonymous PagePreview boundary matrix:
    // listpages-head-recovery-final-unclosed-{unknown-a,unknown-b,known,
    // spaced,single-bracket,spaced-brackets}. Wikidot consumes the authored
    // body but renders its default row template.
    for malformed_tail in [
        r#"mystery="alpha]]"#,
        r#"alternate="bravo-42]]"#,
        r#"fullname="main:about]]"#,
        r#"mystery="alpha   ]]"#,
        r#"mystery="alpha]"#,
        r#"mystery="alpha] ]"#,
    ] {
        let source = format!(
            concat!(
                "[[module ListPages category=\"*\" limit=\"1\" order=\"name\" ",
                "{}\n",
                "AUTHORED-CUSTOM-ROW\n",
                "[[/module]]\n",
                "[[module ListPages category=\"*\" fullname=\"second\"]]\n",
                "SECOND\n",
                "[[/module]]",
            ),
            malformed_tail,
        );

        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 2, "{source:?}: {modules:#?}");
        assert_eq!(
            modules[0].body, "",
            "the malformed opener uses Wikidot's default template: {modules:#?}",
        );
        assert!(
            modules[0].original.contains("AUTHORED-CUSTOM-ROW")
                && modules[0].original.ends_with("[[/module]]"),
            "the malformed opener still owns its authored close: {modules:#?}",
        );
        assert_eq!(modules[1].head, "category=\"*\" fullname=\"second\"");
        assert_eq!(modules[1].body, "\nSECOND\n");
    }
}

#[test]
fn scanner_executes_the_live_factory_head_with_a_crossing_rating_quote() {
    // Exact-source PagePreview provenance: zh-tr:factory-hub:L242:B6951.
    // The first module's `rating="<-0"` assignment crosses the still-open
    // `tags` quote. Live nevertheless closes the opener at the line-final
    // `]]`, executes it, and independently executes the later valid module.
    let source = concat!(
        "[[module ListPages order=\"created_at Asc\" limit=\"100\" ",
        "tags=\"-已歸檔 -管理 -作者頁面 -沙盒 scp -tale -en-goi2014 +工廠 ",
        "+scp rating=\"<-0\" separate=\"no\" perPage=\"100\"]]\n",
        "FIRST\n",
        "[[/module]]\n",
        "[[module ListPages order=\"created_at Asc\" limit=\"100\" ",
        "tags=\"+故事 +工廠\" separate=\"no\" perPage=\"100\"]]\n",
        "SECOND\n",
        "[[/module]]",
    );

    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert_eq!(
        modules[0].head,
        concat!(
            "order=\"created_at Asc\" limit=\"100\" ",
            "tags=\"-已歸檔 -管理 -作者頁面 -沙盒 scp -tale -en-goi2014 +工廠 ",
            "+scp rating=\"<-0\" separate=\"no\" perPage=\"100\"",
        ),
    );
    assert_eq!(modules[0].body, "\nFIRST\n");
    assert!(!modules[0].runtime_safe, "{modules:#?}");
    assert_eq!(modules[1].body, "\nSECOND\n");
}

#[test]
fn scanner_accepts_an_inline_comment_between_legacy_arguments() {
    let source = concat!(
        "[[module ListPages rating=\">60\" order=\"rating desc\" category=\"*\" ",
        "separate=\"false\" limit=\"200\" perPage=\"35\" date=\"@URL|2023\"",
        "[!-- UPDATE THIS TO CURRENT YEAR --] ",
        "prependLine=\"||~ Title ||~ Rating ||\"]]\n",
        "ROW\n[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(modules[0].body, "\nROW\n");
    assert!(list_pages_runtime_head_can_execute(modules[0].head));
    assert_eq!(
        wikidot_list_pages_arguments(modules[0].head)
            .into_iter()
            .map(|argument| (argument.key, argument.value))
            .collect::<Vec<_>>(),
        vec![
            ("rating", ">60"),
            ("order", "rating desc"),
            ("category", "*"),
            ("separate", "false"),
            ("limit", "200"),
            ("perPage", "35"),
            ("date", "@URL|2023"),
            ("prependLine", "||~ Title ||~ Rating ||"),
        ],
        "inline comment contents must not become semantic arguments",
    );
}

#[test]
fn incomplete_module_openers_at_eof_fail_closed() {
    // Without the required closing delimiter, this unsupported shape must
    // stay outside the scanner's executable module matches.
    for source in ["[[module ListPages", "[[module ListPages]"] {
        let modules = find_list_pages_module_matches(source);
        assert!(modules.is_empty(), "{source:?}: {modules:#?}");
    }
}

#[test]
fn unclosed_listpages_head_consumes_immediate_raw_closer() {
    let source = concat!(
        "[[module ListPages separate=\"no\" limit=\"250\" perPage=\"250\" ",
        "tags=\"group-of-interest-form, beyond +_entropy-, -scp, -story,\" ",
        "order=\"title\"\n\n",
        "[[/module]]\nAFTER_MALFORMED\n",
        "[[module ListPages name=\"listpages-head-boundary-target\"]]",
        "SECOND|%%fullname%%[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert!(modules[0].consume_empty_tail, "{modules:#?}");
    assert_eq!(modules[0].body, "");
    assert!(modules[0].original.ends_with("[[/module]]"));
    assert_eq!(modules[1].body, "SECOND|%%fullname%%");
}

#[test]
fn corpus_unicode_tag_head_remains_runtime_executable() {
    let head = r#"separate="1" tags="+阿尔兹海默症 -中心" order="random"  perPage="50""#;
    assert_eq!(
        validate_module_head(head, true),
        ModuleHeadValidation::RuntimeSafe,
    );
    assert!(runtime_regex_recognizes_entire_head(head));
    assert!(list_pages_runtime_head_can_execute(head));

    let source = format!(
        "[[module ListPages {head}]]\n* %%title_linked%% - %%created_by_linked%%\n[[/module]]",
    );
    let modules = find_list_pages_module_matches(&source);
    assert_eq!(modules.len(), 1);
    assert!(modules[0].runtime_safe);
}

#[test]
fn linear_list_pages_scanner_preserves_nested_modules_and_source_order() {
    let source = concat!(
        "before\n",
        "[[module Listpages name=\"first\"]]A",
        "[[module ListUsers users=\".\"]]nested[[/module]]",
        "B[[/module]]\n",
        "between\n",
        "[[module LISTPAGES name=\"second\"]]C[[/module]]\n",
        "after",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert!(modules[0].body.contains("[[module ListUsers"));
    assert!(modules[0].body.ends_with('B'));
    assert_eq!(modules[1].head, "name=\"second\"");
    assert_eq!(modules[1].body, "C");
    assert!(modules[0].end < modules[1].start);
    assert_eq!(
        modules[0].original,
        "[[module Listpages name=\"first\"]]A[[module ListUsers users=\".\"]]nested[[/module]]B[[/module]]",
    );
}

#[test]
fn list_pages_scanner_requires_module_and_subname_delimiters() {
    let source = concat!(
        "[[moduleListPages name=\"joined\"]]ignored[[/module]]\n",
        "[[module ListPagesExtra name=\"suffix\"]]ignored[[/module]]\n",
        "[[module ListPages.other name=\"suffix\"]]ignored[[/module]]\n",
        "[[module654 ListPages name=\"legacy\"]]ignored[[/module654]]\n",
        "[[module\tLISTPAGES\tname=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(modules[0].body, "kept");
    assert!(modules[0].runtime_safe);

    let missing_subname = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module]]missing[[/module]]",
        "after[[/module]]",
    );
    let modules = find_list_pages_module_matches(missing_subname);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module]]missing");

    let legacy_nested = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/module654]]C",
        "[[module654 ListUsers]]D[[/module]]E",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(legacy_nested);
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].body,
        "A[[module654 ListUsers]]B[[/module654]]C[[module654 ListUsers]]D[[/module]]E",
    );
}

#[test]
fn list_pages_scanner_accepts_pinned_horizontal_space_delimiters() {
    let source = concat!(
        "[[ \tmodule\tListPages name=\"spaced\"]]A[[/ \tmodule \t]]\n",
        "[[module ListPages name=\"outer\"]]B",
        "[[ \tmodule654\tListUsers]]nested[[/ \tmodule654\t]]C",
        "[[/module]]\n",
        "[[ module654 ListPages name=\"legacy\"]]ignored[[/ module654 ]]\n",
        "[[module ListPages name=\"last\"]]D[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 3);
    assert_eq!(modules[0].head, "name=\"spaced\"");
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[0].original,
        "[[ \tmodule\tListPages name=\"spaced\"]]A[[/ \tmodule \t]]",
    );
    assert_eq!(modules[1].head, "name=\"outer\"");
    assert_eq!(
        modules[1].body,
        "B[[ \tmodule654\tListUsers]]nested[[/ \tmodule654\t]]C",
    );
    assert_eq!(modules[2].head, "name=\"last\"");
    assert_eq!(modules[2].body, "D");
}

#[test]
fn list_pages_scanner_uses_pinned_module_name_delimiters() {
    for invalid_open in [
        "[[\nmodule ListPages name=\"lf-before-module\"]]body[[/module]]",
        "[[module \nListPages name=\"space-before-lf\"]]body[[/module]]",
        "[[module\u{000b}ListPages name=\"vt\"]]body[[/module]]",
        "[[module\u{000c}ListPages name=\"ff\"]]body[[/module]]",
    ] {
        assert!(
            find_list_pages_module_matches(invalid_open).is_empty(),
            "invalid opener was recognized: {invalid_open:?}",
        );
    }

    for invalid_close in ["[[/\nmodule]]", "[[/module \n]]"] {
        let source =
            format!("[[module ListPages name=\"live\"]]A{invalid_close}B[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "invalid closer {invalid_close:?}");
        assert_eq!(
            modules[0].body,
            format!("A{invalid_close}B"),
            "invalid closer {invalid_close:?}",
        );
    }

    let scored_root = "[[module ListPages name=\"live\"]]A[[/module_]]OUT[[/module]]";
    let modules = find_list_pages_module_matches(scored_root);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A");

    let scored_nested = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/module654_]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(scored_nested);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module654 ListUsers]]B[[/module654_]]C");

    for optional_left_block_close in ["[[/[[module]]", "[[/ [[ module]]"] {
        let source = format!(
            "[[module ListPages name=\"live\"]]A{optional_left_block_close}OUT[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{optional_left_block_close:?}");
        assert_eq!(modules[0].body, "A", "{optional_left_block_close:?}");
    }

    let nested_optional_left_block = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/ [[ module654_]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(nested_optional_left_block);
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].body,
        "A[[module654 ListUsers]]B[[/ [[ module654_]]C"
    );

    for trimmed_close in ["[[/module\u{000b}]]", "[[/module\u{000c}]]"] {
        let source =
            format!("[[module ListPages name=\"live\"]]A{trimmed_close}B[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "trimmed closer {trimmed_close:?}");
        assert_eq!(modules[0].body, "A", "trimmed closer {trimmed_close:?}");
    }

    for (label, separator) in [
        ("LF", "\n"),
        ("CRLF", "\r\n"),
        ("CR", "\r"),
        ("paragraph LF", "\n\n"),
        ("paragraph CRLF", "\r\n\r\n"),
    ] {
        let source = format!(
            "[[module{separator} \tListPages name=\"{label}\"]]body[[/module{separator} \t]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, format!("name=\"{label}\""), "{label}");
        assert_eq!(modules[0].body, "body", "{label}");
    }
}

#[test]
fn count_pages_preflight_accepts_regex_valid_mixed_whitespace() {
    for separator in [" \n", "\t\r\n \t", "\n "] {
        let source =
            format!("[[module{separator}CountPages tags=\"+active\"]]A[[/module]]");
        assert!(
            has_count_pages_module_opening_candidate(&source),
            "CountPages preflight should accept regex-valid separator {separator:?}",
        );
    }
}

#[test]
fn standalone_countpages_detector_matches_frozen_ownership_shapes() {
    // Frozen Wikidot observation: a standalone nested opening is treated as
    // the empty-row shape, while a same-line module body takes the separate
    // legacy-tail path. Comments own their module-shaped text per the syntax
    // comment specification.
    let standalone = "[[module CountPages category=\"*\" tags=\"+active\" limit=\"10\"]]";
    assert!(list_pages_body_has_standalone_count_pages_opening(
        standalone
    ));

    let inline = concat!(
        "BEFORE_INLINE\n",
        "[[module CountPages category=\"*\" tags=\"+active\" limit=\"10\"]]",
        "INLINE_COUNT=%%total%%[[/module]]\n",
        "AFTER_INLINE",
    );
    assert!(!list_pages_body_has_standalone_count_pages_opening(inline));
    assert!(list_pages_body_inline_count_pages_legacy_tail(inline).is_some());

    let comment = concat!(
        "[!--\n",
        "[[module CountPages category=\"*\" tags=\"+active\" limit=\"10\"]]\n",
        "--]",
    );
    assert!(!list_pages_body_has_standalone_count_pages_opening(comment));
}

#[test]
fn list_pages_preflight_keeps_pinned_module_name_delimiters() {
    assert!(!has_list_pages_module_opening_candidate(
        "[[module \nListPages name=\"space-before-lf\"]]body[[/module]]",
    ));
    assert!(!has_list_pages_module_opening_candidate(
        "[[moduleFoo ListPages name=\"unknown-module-name\"]]body[[/module]]",
    ));
}

#[test]
fn multiline_module_names_participate_in_active_nesting() {
    for (label, separator) in [
        ("LF", "\n"),
        ("CRLF", "\r\n"),
        ("CR", "\r"),
        ("paragraph", "\n\n"),
    ] {
        let nested = format!("[[module{separator} Foo]]B[[/module{separator} ]]C");
        let source = format!("[[module ListPages name=\"outer\"]]A{nested}[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].body, format!("A{nested}"), "{label}");
    }
}

#[test]
fn list_pages_scanner_uses_projected_structural_opens_only_for_active_nesting() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]before",
        "[[module\0Foo]]generic[[/module]]",
        "[[module ListPages name=\"inner\"]]inner[[/module]]",
        "after[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "before[[module\0Foo]]generic[[/module]]",
            "[[module ListPages name=\"inner\"]]inner[[/module]]after",
        ),
    );

    let projected_roots = concat!(
        "[[module\0ListPages name=\"nul\"]]ignored[[/module]]\n",
        "[[module\\\n ListPages name=\"continued\"]]ignored[[/module]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(projected_roots);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_projects_continued_structural_open_delimiters() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let nested_open = format!("[[module\\{line_end} Foo]]");
        let source = format!(
            "[[module ListPages name=\"outer\"]]A{nested_open}B[[/module]]\
             [[module ListPages name=\"inner\"]]C[[/module]]D[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert!(modules[0].body.contains(&nested_open), "{label}");
        assert!(modules[0].body.ends_with('D'), "{label}");
    }

    let cascading = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module\\\\\n\n Foo]]B[[/module]]",
        "[[module ListPages name=\"inner\"]]C[[/module]]D",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(cascading);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert!(modules[0].body.ends_with('D'));
}

#[test]
fn list_pages_scanner_does_not_invent_a_missing_projected_module_delimiter() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]A\
             [[module\\{line_end}Foo]]B[[/module]]\
             [[module ListPages name=\"inner\"]]C[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert_eq!(modules[1].head, "name=\"inner\"", "{label}");
    }
}

#[test]
fn list_pages_scanner_projects_structural_module_closers() {
    for (label, close) in [
        ("NUL", "[[/module\0]]"),
        ("LF", "[[/module\\\n ]]"),
        ("CRLF", "[[/module\\\r\n ]]"),
        ("CR", "[[/module\\\r ]]"),
    ] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]body{close}\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert_eq!(modules[0].body, "body", "{label}");
        assert_eq!(
            modules[0].original,
            format!("[[module ListPages name=\"outer\"]]body{close}"),
            "{label}"
        );
        assert_eq!(modules[1].head, "name=\"live\"", "{label}");
    }
}

#[test]
fn projection_guard_is_consumed_only_at_its_candidate() {
    let source = "plain";
    let lowercase = source.to_ascii_lowercase();
    let literal_regions =
        LiteralRegionIndex::new_list_pages_scanner_syntax(source).unwrap();
    let mut scanner = ModuleEventScanner::new(source, &lowercase, &literal_regions);

    scanner.pending_projection_guard = Some((5, 20));
    assert_eq!(scanner.take_projection_guard(4), None);
    assert_eq!(scanner.pending_projection_guard, Some((5, 20)));
    assert_eq!(scanner.take_projection_guard(5), Some(20));
    assert_eq!(scanner.pending_projection_guard, None);

    scanner.pending_projection_guard = Some((7, 30));
    assert_eq!(scanner.take_projection_guard(8), None);
    assert_eq!(scanner.pending_projection_guard, None);
}

#[test]
fn list_pages_scanner_respects_left_bracket_token_precedence() {
    for run in [4usize, 8] {
        let source = format!(
            "{}module ListPages name=\"forged\"]]body[[/module]]",
            "[".repeat(run),
        );
        assert!(
            find_list_pages_module_matches(&source).is_empty(),
            "run {run}"
        );
    }

    let nine = format!(
        "{}module ListPages name=\"real\"]]body[[/module]]",
        "[".repeat(9),
    );
    assert!(find_list_pages_module_matches(&nine).is_empty());

    let ten = format!(
        "{}module ListPages name=\"real\"]]body[[/module]]",
        "[".repeat(10),
    );
    let modules = find_list_pages_module_matches(&ten);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].start, 8);
    assert_eq!(modules[0].body, "body");

    let forged_close = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[[[/module]]B[[/module]]\n",
        "[[module ListPages name=\"inner\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(forged_close);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].body, "A[[[[/module]]B");
    assert_eq!(modules[1].head, "name=\"inner\"");
}

#[test]
fn projected_scanning_is_applied_exactly_once() {
    for source in [
        "\0> [[module ListPages name=\"nul\"]]A[[/module]]",
        "\\\n > [[module ListPages name=\"continued\"]]B[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
    }
}

#[test]
fn projected_structural_end_excludes_deleted_eof_suffix() {
    let source = "[[module ListPages name=\"live\"]]body[[/module]]\\\n";
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].original,
        "[[module ListPages name=\"live\"]]body[[/module]]"
    );
    assert_eq!(modules[0].end, source.len() - 2);
}

#[test]
fn unresolved_parser_functions_always_fail_closed() {
    let ambiguous = concat!(
        "prefix\\\n",
        "[[#if true]][[module ListPages name=\"hidden\"]]A[[/module]][[/if]]",
    );
    assert!(find_list_pages_module_matches(ambiguous).is_empty());

    for unresolved in ["[[#if true]]text[[/if]]", "[[[#if true]]text[[/if]]"] {
        let source =
            format!("{unresolved}[[module ListPages name=\"hidden\"]]B[[/module]]",);
        assert!(find_list_pages_module_matches(&source).is_empty());
    }

    let rolled_back_generic = concat!(
        "[[span title='[[#if true]]owned[[/if]]']]",
        "[[module ListPages name=\"hidden\"]]B[[/module]]",
    );
    assert!(find_list_pages_module_matches(rolled_back_generic).is_empty());
}

#[test]
fn inline_ifexpr_does_not_suppress_listpages_modules() {
    let source = concat!(
        "[[module ListPages category=\"*\"]]\n",
        "[[#ifexpr %%rating_votes%% == 0 | ZERO_VOTES | HAS_VOTES]]\n",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].body,
        "\n[[#ifexpr %%rating_votes%% == 0 | ZERO_VOTES | HAS_VOTES]]\n",
    );

    let source = concat!(
        "[[#ifexpr 1 == 1 | yes | no]] ",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn owned_unresolved_and_whole_head_markers_do_not_suppress_live_modules() {
    for source in [
        concat!(
            "@@[[#if true]]@@ ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[[span title=\"[[#if true]]owned[[/if]]\"]] ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[https://e.test/ [[user foo [[module ListPages name=\"fake\"]]",
            "X[[/module]] ]] ] ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[[# tabanchor]]\n",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{source:?}");
    }
}

#[test]
fn original_css_ownership_filters_projected_structural_events() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]A\n",
        ">\0[[module CSS]]",
        "[[module ListPages name=\"hidden\"]]B[[/module]]",
        "[[/module]]C",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert!(modules[0].body.ends_with('C'));
}

#[test]
fn continued_quote_close_can_still_be_a_direct_list_pages_candidate() {
    for line_end in ["\n", "\r\n", "\r"] {
        let source =
            format!("[[module ListPages name=\"live\"\\{line_end}]]body[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{line_end:?}");
        assert_eq!(modules[0].body, "body", "{line_end:?}");
    }
}

#[test]
fn malformed_nested_module_head_does_not_swallow_the_outer_close() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module Foo [[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(modules[0].body, "A[[module Foo ");
}

#[test]
fn list_pages_scanner_ignores_right_block_tokens_inside_double_quoted_arguments() {
    let source = concat!(
        "[[module ListPages name=\"first\" prependLine=\"literal ]] value\"]]",
        "A[[/module]]\n",
        "[[module ListPages name=\"doubled\" prependLine=\"\"literal ]] value\"\"]]",
        "B[[/module]]\n",
        "[[module ListPages name=\"embedded\" prependLine=\"the \"literal ]] value\" suffix\"]]",
        "C[[/module]]\n",
        "[[module ListPages name=\"last\"]]D[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 4);
    assert_eq!(
        modules[0].head,
        "name=\"first\" prependLine=\"literal ]] value\"",
    );
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[1].head,
        "name=\"doubled\" prependLine=\"\"literal ]] value\"\"",
    );
    assert_eq!(modules[1].body, "B");
    assert_eq!(
        modules[2].head,
        "name=\"embedded\" prependLine=\"the \"literal ]] value\" suffix\"",
    );
    assert_eq!(modules[2].body, "C");
    assert_eq!(modules[3].head, "name=\"last\"");
    assert_eq!(modules[3].body, "D");
}

#[test]
fn list_pages_scanner_ignores_right_block_tokens_inside_single_quoted_arguments() {
    let source = concat!(
        "[[module ListPages name='first' prependLine='literal ]] value']]",
        "A[[/module]]\n",
        r#"[[module ListPages name='escaped' prependLine='literal \' ]] value']]"#,
        "B[[/module]]\n",
        "[[module ListPages name='last']]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 3);
    assert_eq!(
        modules[0].head,
        "name='first' prependLine='literal ]] value'",
    );
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[1].head,
        r#"name='escaped' prependLine='literal \' ]] value'"#,
    );
    assert_eq!(modules[1].body, "B");
    assert_eq!(modules[2].head, "name='last'");
    assert_eq!(modules[2].body, "C");
}

#[test]
fn list_pages_scanner_keeps_crossing_url_quote_heads_visible() {
    let source = concat!(
        r#"[[module ListPages category="fragment" parent="." limit="1" order="name" offset="@URL|1 "created_by="Fireknight"]]"#,
        "\n%%content%%\n[[/module]]",
    );
    assert!(list_pages_url_quote_crossing_head_can_execute(
        r#" category="fragment" parent="." limit="1" order="name" offset="@URL|1 "created_by="Fireknight""#,
    ));
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "\n%%content%%\n");
    assert!(
        modules[0]
            .head
            .contains(r#"offset="@URL|1 "created_by="Fireknight"#)
    );

    assert!(!list_pages_url_quote_crossing_head_can_execute(
        r#" category="fragment" parent="." limit="1" order="name" offset="@URL|1 "created_by="Fireknight" unsupported"#,
    ));
}

#[test]
fn list_pages_scanner_consumes_surplus_right_brackets_at_module_head_boundary() {
    for right_brackets in ["]]]", "]]]]", "]]]]]", "]]]]]]"] {
        let source =
            format!("[[module ListPages name=\"live\"{right_brackets}kept[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{right_brackets}");
        assert_eq!(modules[0].head, "name=\"live\"", "{right_brackets}");
        assert_eq!(modules[0].body, "kept", "{right_brackets}");
    }
}

#[test]
fn list_pages_scanner_accepts_physical_newlines_in_complete_heads() {
    for source in [
        "[[module ListPages\nname=\"live\"\norder=\"name\"\n]]kept[[/module]]",
        "[[module ListPages name=\"live\" order=\"name\n\"]]kept[[/module]]",
        "[[module ListPages name=\"live\nname\" limit=\"1\"]]kept[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(source);

        assert_eq!(modules.len(), 1, "{source:?}");
        assert_eq!(modules[0].body, "kept", "{source:?}");
        assert!(modules[0].runtime_safe, "{source:?}");
    }
}

#[test]
fn list_pages_scanner_rejects_right_link_runs_as_module_close_terminators() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]",
        "before[[/module]]]after",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(modules[0].body, "before[[/module]]]after");
}

#[test]
fn list_pages_scanner_keeps_a_multiline_quote_open_until_a_later_quote() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name=\"first\"]]A[[/module]]{line_end}\
             [[module ListPages name=\"unterminated{line_end}\
             [[module ListPages name=\"second\"]]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{label}");
        assert_eq!(modules[0].head, "name=\"first\"", "{label}");
        assert_eq!(modules[0].body, "A", "{label}");
        assert!(
            modules[1]
                .head
                .contains("[[module ListPages name=\"second\""),
            "{label}: {:?}",
            modules[1].head,
        );
        assert_eq!(modules[1].body, "B", "{label}");
    }
}

#[test]
fn list_pages_scanner_fails_closed_when_a_single_quoted_head_reaches_a_physical_line_end()
{
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name='first']]A[[/module]]{line_end}\
             [[module ListPages name='unterminated{line_end}\
             [[module ListPages name='second']]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert!(modules.is_empty(), "{label}: {modules:#?}");
    }
}

#[test]
fn list_pages_scanner_does_not_treat_quoted_attribute_literals_as_syntax() {
    for (attribute, expected_head) in [
        ("prependLine=\"@@\"", "name=\"first\" prependLine=\"@@\""),
        (
            r#"prependLine="[!--""#,
            r#"name="first" prependLine="[!--""#,
        ),
    ] {
        let source = format!(
            "[[module ListPages name=\"first\" {attribute}]]A[[/module]]\n\
             [[module ListPages name=\"second\"]]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "attribute {attribute:?}");
        assert_eq!(modules[0].head, expected_head, "attribute {attribute:?}");
        assert_eq!(modules[0].body, "A", "attribute {attribute:?}");
        assert_eq!(
            modules[1].head, "name=\"second\"",
            "attribute {attribute:?}"
        );
        assert_eq!(modules[1].body, "B", "attribute {attribute:?}");
    }

    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let malformed_generic_head = format!(
            "[[span title='@@ [!--{line_end}\
             [[module ListPages name=\"live\"]]body[[/module]]",
        );
        let modules = find_list_pages_module_matches(&malformed_generic_head);
        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");

        let malformed_unquoted_head = format!(
            "[[span class=unterminated title=\"still-malformed\"{line_end}\
             [[module ListPages name=\"live\"]]body[[/module]]",
        );
        let modules = find_list_pages_module_matches(&malformed_unquoted_head);
        assert_eq!(modules.len(), 1, "unquoted {label}");
        assert_eq!(modules[0].head, "name=\"live\"", "unquoted {label}");
    }
}

#[test]
fn list_pages_scanner_jumps_to_the_end_of_an_owning_literal_region() {
    for literal in ["[!--\n[[broken\n--]", "@@[[broken@@", "@<[[broken>@"] {
        let source =
            format!("{literal}\n[[module ListPages name=\"live\"]]body[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{literal:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{literal:?}");
        assert_eq!(modules[0].body, "body", "{literal:?}");
    }
}

#[test]
fn list_pages_scanner_ignores_modules_inside_pinned_literal_forms() {
    let cases = [
        (
            "inline raw",
            "@@[[module ListPages name=\"fake\"]]body[[/module]]@@",
        ),
        (
            "angle raw",
            "@<[[module ListPages name=\"fake\"]]body[[/module]]>@",
        ),
        (
            "inline math",
            "[[$ [[module ListPages name=\"fake\"]]body[[/module]] $]]",
        ),
        (
            "math block",
            "[[math]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/math]]",
        ),
        (
            "multiline math head",
            "[[math\n label=\"display\"]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/math]]",
        ),
        (
            "empty embed block",
            "[[embed]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/embed]]",
        ),
    ];

    for (label, literal) in cases {
        let source =
            format!("{literal}\n[[module ListPages name=\"live\"]]kept[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }

    let wikidot_unclosed_css = concat!(
        "[[module CSS]]\n",
        "[[module ListPages name=\"recovered\"]]body\n",
        "[[/module]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(wikidot_unclosed_css);

    // Anonymous Wikidot PagePreview executes both root ListPages modules for
    // this boundary family. The matching two-module evidence has raw HTML
    // SHA-256
    // 48a80a2dd6ca8c27d1813e82ed7e0f9a1700214d34095cb81fdefda88344365f.
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"recovered\"");
    assert_eq!(modules[0].body, "body\n");
    assert_eq!(modules[1].head, "name=\"live\"");
    assert_eq!(modules[1].body, "kept");

    for (label, opener) in [
        ("unclosed inline raw", "@@"),
        ("unclosed angle raw", "@<"),
        ("unclosed inline math", "[[$"),
    ] {
        let source = format!(
            "{opener}[[module ListPages name=\"fake\"]]\n\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
    }

    let continued_raw = concat!(
        "@@prefix\\\n",
        "[[module ListPages name=\"fake\"]]body[[/module]]@@\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(continued_raw);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_skips_module_text_inside_generic_tag_heads() {
    let source = concat!(
        "[[span title=\"[[module ListPages name='fake']]fake[[/module]]\"]]",
        "outside",
        "[[module ListPages name=\"outer\"]]",
        "A[[span title=\"[[module ListUsers]]nested[[/module]]\"]]B",
        r#"C[[span title="escaped \" [[module ListUsers]]nested[[/module]] value"]]D"#,
        "[[/module]]",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "A[[span title=\"[[module ListUsers]]nested[[/module]]\"]]B",
            r#"C[[span title="escaped \" [[module ListUsers]]nested[[/module]] value"]]D"#,
        ),
    );
    assert_eq!(modules[1].head, "name=\"live\"");
    assert_eq!(modules[1].body, "C");
}

#[test]
fn list_pages_scanner_skips_module_text_inside_multiline_generic_tag_heads() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[div{line_end}\
             class=\"[[module ListPages name='fake']]fake[[/module]]\"{line_end}\
             data-kind=\"compatibility\"]]{line_end}\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }
}

#[test]
fn list_pages_scanner_preserves_continued_generic_tag_head_quotes() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[span title=\"before\\{line_end}\
             [[module ListPages name='hidden']]ignored[[/module]]\"]]\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }

    let cascading = concat!(
        "[[span title=\"before\\\\\n\n",
        "[[module ListPages name='hidden']]ignored[[/module]]\"]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(cascading);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_preserves_multiline_image_link_and_quoted_alt_head() {
    for image in ["image", "=image", "<image", ">image", "f<image", "f>image"] {
        for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
            let source = format!(
                "[[{image} https://e.test/x.png link= \t #{line_end}\
                 alt=\"[[module ListPages name='hidden']]ignored[[/module]]\"]]{line_end}\
                 [[module ListPages name=\"live\"]]kept[[/module]]",
            );
            let modules = find_list_pages_module_matches(&source);

            assert_eq!(modules.len(), 1, "{image} {label}");
            assert_eq!(modules[0].head, "name=\"live\"", "{image} {label}");
            assert_eq!(modules[0].body, "kept", "{image} {label}");
        }
    }

    let nested_syntax_shaped_value = concat!(
        "[[image x.png link=#fragment[[module ListPages name='hidden']]ignored[[/module]] ",
        "alt=\"safe\"]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(nested_syntax_shaped_value);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn positional_heads_keep_competing_tokens_and_quotes_in_the_source_value() {
    for source_value in ["foo]]]\n", "foo$]]\n", "foo--]]\n", "foo\"bar\n"] {
        let source = format!(
            "[[image {source_value}alt=\"[[module ListPages name='fake']]ignored[[/module]]\"\n]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{source_value:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{source_value:?}");
        assert_eq!(modules[0].body, "kept", "{source_value:?}");
    }

    for star in ["*radio", "* radio"] {
        let source = format!(
            "[[{star} choice=one\n title=\"[[module ListPages name='fake']]ignored[[/module]]\"\n]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{star}");
        assert_eq!(modules[0].head, "name=\"real\"", "{star}");
    }

    for separator in ["\u{000b}", "\u{000c}"] {
        let source = format!(
            "[[radio foo{separator}[[module ListPages = \"x\"]]ignored[[/module]] ]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{separator:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{separator:?}");
    }
}

#[test]
fn complete_whole_heads_own_nested_blocks_across_competing_right_tokens() {
    for competing in ["$]]", "--]]"] {
        let source = format!(
            "[[user foo{competing} [[module ListPages name=\"hidden\"]]x[[/module]] ]]\n\
             [[module ListPages name=\"also-hidden\"]]y[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{competing:?}");
        assert_eq!(modules[0].head, "name=\"also-hidden\"", "{competing:?}");
    }

    let uppercase_url = concat!(
        "[[user HTTP://e.test/a$]][[module ListPages name=\"hidden\"]]x[[/module]] ]]",
        "[[module ListPages name=\"also-hidden\"]]y[[/module]]",
    );
    let modules = find_list_pages_module_matches(uppercase_url);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"also-hidden\"");

    let lowercase_url = concat!(
        "[[user http://e.test/a$]]",
        "[[module ListPages name=\"live\"]]x[[/module]]",
    );
    let modules = find_list_pages_module_matches(lowercase_url);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let url_owned_comment_prefix = concat!(
        "[[target https://e.test/a--]]] [[module ListPages]]fake[[/module]] ]]\n",
        "[[module ListPages]]real[[/module]]",
    );
    let modules = find_list_pages_module_matches(url_owned_comment_prefix);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "");
}

#[test]
fn malformed_generic_heads_roll_back_to_the_first_nested_block() {
    let whole_head_newline = concat!(
        "[[module ListPages name=\"first\"]]A[[/module]]",
        "[[user foo [[module\n ListPages name=\"hidden\"]]H[[/module]]",
    );
    let modules = find_list_pages_module_matches(whole_head_newline);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert_eq!(modules[1].head, "name=\"hidden\"");

    let invalid_argument_tail = concat!(
        "[[module ListPages name=\"first\"]]A[[/module]]",
        "[[span title=\"x\" bad other=\"[[module ListPages]]H[[/module]]\"]]",
    );
    let modules = find_list_pages_module_matches(invalid_argument_tail);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert_eq!(modules[1].head, "");
    assert_eq!(modules[1].body, "H");

    let nonmodule_nested_block = concat!(
        "[[span title=\"[[span]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(nonmodule_nested_block);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let single_quoted_generic = concat!(
        "[[span title='",
        "[[module ListPages name=\"inner\"]]Y[[/module]]",
        "']]",
    );
    let modules = find_list_pages_module_matches(single_quoted_generic);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"inner\"");

    let malformed_generic_module = concat!(
        "[[module Foo title=\"[[span]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(malformed_generic_module);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn pinned_generic_quote_lookahead_counts_its_examined_suffix() {
    // `data-*` attributes are documented for span blocks. When one quoted
    // value is followed by another attribute, the lookahead ends at that
    // assignment's equals sign and reports precisely the bytes it inspected.
    let source = r#"[[span data-first="x" data-second="y"]]"#;
    let quote = source.find("\" data-second").unwrap();
    let head_end = source.rfind("]]").unwrap();
    let equals = source.find("data-second=").unwrap() + "data-second".len();
    let (ends_argument, inspected) = pinned_double_quote_ends_generic_argument(
        source.as_bytes(),
        quote,
        head_end,
        &TextTokenCursor::new(source),
    );

    assert!(ends_argument);
    assert_eq!(inspected, equals - quote);

    let newline_source = "[[span data-first=\"x\"\n]]";
    let quote = newline_source.find("\"\n").unwrap();
    let head_end = newline_source.rfind("]]").unwrap();
    let (ends_argument, inspected) = pinned_double_quote_ends_generic_argument(
        newline_source.as_bytes(),
        quote,
        head_end,
        &TextTokenCursor::new(newline_source),
    );
    assert!(ends_argument);
    assert_eq!(inspected, 1);

    let spaced_newline_source = "[[span data-first=\"x\" \n]]";
    let quote = spaced_newline_source.find("\" \n").unwrap();
    let head_end = spaced_newline_source.rfind("]]").unwrap();
    let (ends_argument, inspected) = pinned_double_quote_ends_generic_argument(
        spaced_newline_source.as_bytes(),
        quote,
        head_end,
        &TextTokenCursor::new(spaced_newline_source),
    );
    assert!(!ends_argument);
    assert_eq!(inspected, 2);

    let close_source = r#"[[span data-first="x"]]"#;
    let quote = close_source.find("\"]]").unwrap();
    let head_end = close_source.rfind("]]").unwrap();
    let (ends_argument, inspected) = pinned_double_quote_ends_generic_argument(
        close_source.as_bytes(),
        quote,
        head_end,
        &TextTokenCursor::new(close_source),
    );
    assert!(ends_argument);
    assert_eq!(inspected, 0);
}

#[test]
fn pinned_generic_quote_lookahead_rejects_an_empty_next_attribute_key() {
    // The punctuation after the quoted value cannot begin a supported
    // `key="value"` attribute and must terminate the lookahead promptly.
    let source = r#"[[span data-first="x" ?]]"#;
    let quote = source.find("\" ?").unwrap();
    let head_end = source.rfind("]]").unwrap();
    let (ends_argument, inspected) = pinned_double_quote_ends_generic_argument(
        source.as_bytes(),
        quote,
        head_end,
        &TextTokenCursor::new(source),
    );

    assert!(!ends_argument);
    assert_eq!(inspected, 2);
}

#[test]
fn rolled_back_competing_parser_function_runs_fail_closed() {
    let live = "[[module ListPages name=\"live\"]]Y[[/module]]";
    for malformed in [
        format!("[[span title='[[[#if true]]owned[[/if]]']] {live}",),
        format!("[[module Foo title='[[[#if true]]owned[[/if]]']] {live}",),
        format!("[[user foo [[[#if true\n{live}"),
    ] {
        assert!(
            find_list_pages_module_matches(&malformed).is_empty(),
            "{malformed:?}",
        );
    }

    let complete_owner = format!("[[span title=\"[[[#if true]]owned[[/if]]\"]] {live}",);
    let modules = find_list_pages_module_matches(&complete_owner);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn quotes_after_emails_close_attribute_values_like_plain_text() {
    // The pinned FTML email token ends at its last domain label, as Wikidot
    // does, so a following quote closes the value exactly as after plain text.
    let shapes = [
        concat!(
            "[[span title=\"{}\" x=y ",
            "[[module ListPages name='fake']]ignored[[/module]] end\" safe=\"ok\"]]",
            "[[module ListPages name=\"real\"]]kept[[/module]]",
        ),
        concat!(
            "[[module ListPages title=\"{}\" x=y ",
            "[[module ListPages]]fake[[/module]] end\" safe=\"ok\"]]",
            "OUTER[[/module]]",
        ),
        "[[module ListPages title=\"{}\" ]]BODY[[/module]]",
        "[[module ListPages name = \"{}\" wrapper=no\"]]x[[/module]]",
        concat!(
            "[[span a=\"prefix\" foo={} ",
            "[[module ListPages]]fake[[/module]] final\"]]",
            "[[module ListPages name=\"real\"]]kept[[/module]]",
        ),
        concat!(
            "[[span a=\"prefix\"\\\n",
            "foo={} [[module ListPages]]fake[[/module]] final\"]]",
            "[[module ListPages name=\"real\"]]kept[[/module]]",
        ),
        concat!(
            "[[span title=\"\\{}\n",
            "[[module ListPages name='live']]X[[/module]]\n\"]]",
        ),
    ];
    let summary = |source: &str| {
        find_list_pages_module_matches(source)
            .into_iter()
            .map(|module| (module.body.to_owned(), module.runtime_safe))
            .collect::<Vec<_>>()
    };
    for shape in shapes {
        let email = shape.replace("{}", "foo@bar.example");
        let plain = shape.replace("{}", "foobarexample");
        assert_eq!(summary(&email), summary(&plain), "{email:?}");
    }
    let modules = find_list_pages_module_matches(
        "[[module ListPages title=\"foo@bar.example\" ]]BODY[[/module]]",
    );
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "BODY");

    let url_owned_backslash = concat!(
        "[[span a=\"https://e.test/x\\\" b=\" foo=bar ",
        "[[module ListPages]]fake[[/module]] final\"]]",
        "[[module ListPages name=\"real\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(url_owned_backslash);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"real\"");
}

#[test]
fn owned_url_bytes_preserve_positional_tabs_and_line_continuations() {
    for outer in [
        "[[image https://e.test/a\t[[module ListPages = \"x\"]]X[[/module]] ]]",
        "[[image x link=https://e.test/a\t[[module ListPages = \"x\"]]X[[/module]] ]]",
    ] {
        let source =
            format!("{outer}\n[[module ListPages name=\"real\"]]kept[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{outer:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{outer:?}");
    }

    let continued = concat!(
        "[[module ListPages title=\"https://e.test/a\\\n",
        "\" name=\"x\"]]BODY[[/module]]",
    );
    let modules = find_list_pages_module_matches(continued);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "BODY");
}

#[test]
fn module_head_validation_preserves_nesting_without_executing_malformed_heads() {
    let source = concat!(
        "[[module ListPages limit=\"1\"]]A",
        "[[module Foo bare]]B[[/module]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module Foo bare]]B");

    for malformed in [
        "[[module ListPages limit\n=\"1\"]]x[[/module]]",
        "[[module ListPages limit=\"1\"\u{000b}]]x[[/module]]",
        "[[module ListPages limit=\u{000c}1]]x[[/module]]",
        "[[module ListPages \u{00a0}limit=\"1\"]]x[[/module]]",
        "[[module ListPages name=\"secret@site.example\" wrapper=no\"]]x[[/module]]",
        "[[module ListPages name=a]b]]x[[/module]]",
        "[[module ListPages ---]=\"x\"]]x[[/module]]",
        "[[module ListPages 1=bare]]x[[/module]]",
        "[[module ListPages [!--=bare]]x[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(malformed);
        assert!(modules.is_empty(), "{malformed:?}: {modules:#?}",);
    }

    for compatible in [
        "[[module ListPages limit=1]]x[[/module]]",
        "[[module ListPages name='x']]x[[/module]]",
        "[[module ListPages _field!=value]]x[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(compatible);
        assert_eq!(modules.len(), 1, "{compatible:?}");
        assert!(modules[0].runtime_safe, "{compatible:?}");
    }

    for structurally_valid_but_unsafe in [
        "[[module ListPages name\0=\0\"x\"]]x[[/module]]",
        "[[module ListPages limit\\\n=\"1\"]]x[[/module]]",
        "[[module ListPages limit\\\n\u{00a0}=\"1\"]]x[[/module]]",
        "[[module ListPages limit\\\r\n\u{2007}=\"1\"]]x[[/module]]",
        "[[module ListPages limit=\\\n\"1\"]]x[[/module]]",
        "[[module ListPages\n\u{00a0}\u{2007}limit=\"1\"]]x[[/module]]",
        "[[module ListPages name = \"secret\"wrapper=\"no\"]]x[[/module]]",
        "[[module ListPages name = \"secret\\\" wrapper=no\"]]x[[/module]]",
        "[[module ListPages name=\"foo\tbar\"]]x[[/module]]",
        "[[module ListPages name=\"foo...\"]]x[[/module]]",
        "[[module ListPages name=\"`x'\"]]x[[/module]]",
        "[[module ListPages name=\"foo\\nbar\"]]x[[/module]]",
        "[[module ListPages [!--=\"x\"]]x[[/module]]",
        "[[module ListPages --]=\"x\"]]x[[/module]]",
        "[[module ListPages [!----]=\"x\"]]x[[/module]]",
        "[[module ListPages 1=\"x\"]]x[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(structurally_valid_but_unsafe);
        assert_eq!(modules.len(), 1, "{structurally_valid_but_unsafe:?}");
        assert!(
            !modules[0].runtime_safe,
            "{structurally_valid_but_unsafe:?}"
        );
    }

    let positional_subname = concat!(
        "[[module Foo]]][[module ListPages name='hidden']]X[[/module]] ]]",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(positional_subname);
    assert!(modules.is_empty(), "{modules:#?}");

    let definite_invalid_marker = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]",
        "[[module Foo [!-?=x]]ignored[[/module]]",
        "[[module ListPages name=\"later\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(definite_invalid_marker);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"complete\"");
    assert_eq!(modules[1].head, "name=\"later\"");

    for raw_runtime_invalid_head in ["1=bare", "name=a]b"] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]A[[module ListPages {raw_runtime_invalid_head}]]B[[/module]]C[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{raw_runtime_invalid_head:?}");
        assert_eq!(
            modules[0].body,
            format!("A[[module ListPages {raw_runtime_invalid_head}]]B"),
            "{raw_runtime_invalid_head:?}",
        );
    }

    let competing_left_run = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module ListPages name=foo[[[[bar]]B[[/module]]C[[/module]]",
    );
    assert!(find_list_pages_module_matches(competing_left_run).is_empty());

    let self_masked_malformed_head = concat!(
        "[[module ListPages name=foo",
        "[[module ListPages name=\"secret\"]]H[[/module]]",
    );
    assert!(find_list_pages_module_matches(self_masked_malformed_head).is_empty());

    let invalid_list_pages_head = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]",
        "[[module ListPages limit\n=\"1\"]]ignored",
        "[[module ListPages name=\"inner\"]]B[[/module]]",
        "[[/module]]",
    );
    assert!(find_list_pages_module_matches(invalid_list_pages_head).is_empty());

    for (complete_multiline_head, expected_head, expected_body) in [
        (
            concat!(
                "[[module ListPages name=\"x\ny\"]]",
                "[[module ListPages name=secret]]H[[/module]][[/module]]",
            ),
            "name=\"x\ny\"",
            "[[module ListPages name=secret]]H[[/module]]",
        ),
        (
            concat!(
                "[[module ListPages name=\"outer\"]]A",
                "[[module ListPages name=\"x\ny\"]]B[[/module]]C[[/module]]",
            ),
            "name=\"outer\"",
            "A[[module ListPages name=\"x\ny\"]]B[[/module]]C",
        ),
    ] {
        let modules = find_list_pages_module_matches(complete_multiline_head);
        assert_eq!(modules.len(), 1, "{complete_multiline_head:?}");
        assert_eq!(
            modules[0].head, expected_head,
            "{complete_multiline_head:?}",
        );
        assert_eq!(
            modules[0].body, expected_body,
            "{complete_multiline_head:?}",
        );
    }

    for runtime_ambiguous_head in [
        concat!(
            "[[module ListPages name=\"x",
            "[[module ListPages name=secret]]%%content%%\" 1=bad]]",
            "outer[[/module]]",
        ),
        concat!(
            "[[module ListPages name=foo[[x]]",
            "[[module ListPages name=secret]]H[[/module]][[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[[[bar]]B[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[x=y]]B[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=\"unterminated\n]]B",
            "[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[x=\"unterminated]]B",
            "[[/module]]C[[/module]]",
        ),
    ] {
        let modules = find_list_pages_module_matches(runtime_ambiguous_head);
        assert!(
            modules.is_empty(),
            "{runtime_ambiguous_head:?}: {modules:#?}",
        );
    }

    let invalid_head_with_suppressed_nested_open = concat!(
        "[[module ListPages limit=\"1\"]]A",
        "[[module Foo x='[[module ListPages name=\"inner\"]]']]B",
        "[[/module]]C[[/module]]",
    );
    let modules =
        find_list_pages_module_matches(invalid_head_with_suppressed_nested_open);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "limit=\"1\"");
}

#[test]
fn full_source_typography_changes_make_direct_heads_runtime_unsafe() {
    assert_eq!(
        validate_module_head(r#"name="``foo''" limit=1"#, true),
        ModuleHeadValidation::ValidRuntimeUnsafe,
    );

    for source in [
        "`prefix [[module ListPages name=\"foo'bar\"]]B[[/module]]",
        "[[module ListPages name=\"foo`bar\"]]B' suffix[[/module]]",
        // A later bare value must not clear a projection contained in the head.
        r#"[[module ListPages name="``foo''" limit=1]]B[[/module]]"#,
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
        assert!(!modules[0].runtime_safe, "{source:?}");
    }
}

#[test]
fn long_unicode_trim_runs_keep_positional_and_whole_head_ownership() {
    let trim = "\u{000b}".repeat(32);
    let positional = format!(
        "[[radio{trim} foo[[module ListPages]]fake[[/module]] ]]\n\
         [[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(&positional);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let whole_head = format!(
        "[[user{trim} foo [[module ListPages]]fake[[/module]] ]]\n\
         [[module ListPages name=\"also-hidden\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(&whole_head);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"also-hidden\"");
}

#[test]
fn module_names_and_subnames_use_unicode_trim() {
    for separator in ["\u{000b}", "\u{000c}"] {
        let source = format!(
            "[[module ListPages{separator} name=\"not-list-pages\"]]ignored[[/module]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{separator:?}");
        assert_eq!(modules[0].head, "name=\"not-list-pages\"", "{separator:?}");
        assert_eq!(modules[1].head, "name=\"real\"", "{separator:?}");
    }

    let unicode = concat!(
        "[[module\u{00a0} ListPages\u{2007} name=\"unicode\"]]A[[/module\u{00a0} ]]\n",
        "[[module ListPages name=\"ascii\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(unicode);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"unicode\"");
    assert_eq!(modules[1].head, "name=\"ascii\"");
}

#[test]
fn module_heads_do_not_close_on_math_or_comment_right_tokens() {
    for competing in ["$]]", "--]]"] {
        let source = format!(
            "[[module ListPages {competing}A[[/module]]\n\
             [[module ListPages name=\"live\"]]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{competing:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{competing:?}");
    }
}

#[test]
fn list_pages_scanner_rejects_right_link_runs_as_generic_tag_terminators() {
    let same_line = concat!(
        "[[span title=\"malformed\"]]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    assert!(find_list_pages_module_matches(same_line).is_empty());

    let next_line = concat!(
        "[[span title=\"malformed\"]]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(next_line);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(modules[0].body, "kept");
}

#[test]
fn list_pages_scanner_ignores_module_events_on_quoted_lines() {
    let source = concat!(
        "> [[module ListPages name=\"spaced\"]]ignored[[/module]]\n",
        ">> [[module ListPages name=\"nested\"]]ignored[[/module]]\n",
        "> > [[module ListPages name=\"split\"]]ignored[[/module]]\n",
        ">[[module ListPages name=\"tight\"]]ignored[[/module]]\n",
        "[[module ListPages name=\"outer\"]]A\n",
        "> [[module ListUsers]]\n",
        "> [[/module]]\n",
        "  >[[module ListUsers]]\n",
        "  > [[/module]]\n",
        "  >[[/module]]\n",
        "B[[/module]]\n",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "A\n> [[module ListUsers]]\n> [[/module]]\n",
            "  >[[module ListUsers]]\n  > [[/module]]\n  >[[/module]]\nB",
        ),
    );
    assert_eq!(modules[1].head, "name=\"live\"");
    assert_eq!(modules[1].body, "C");
}

#[test]
fn list_pages_scanner_keeps_completed_matches_before_an_unclosed_body() {
    let source = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]\n",
        "[[module ListPages name=\"unclosed\"]]B",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"complete\"");
    assert_eq!(modules[1].head, "name=\"unclosed\"");
    assert_eq!(modules[1].body, "");
    assert_eq!(
        modules[1].original,
        "[[module ListPages name=\"unclosed\"]]"
    );

    let unclosed_outer = concat!(
        "[[module ListPages name=\"unclosed\"]]A",
        "[[module ListPages name=\"nested\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(unclosed_outer);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"unclosed\"");
    assert_eq!(modules[0].body, "");

    for bare in [
        "[[module ListPages]]@@example@@",
        "[[module ListPages]]>@\n@<example>@",
        "[[module ListPages]]documentation",
    ] {
        assert!(find_list_pages_module_matches(bare).is_empty(), "{bare:?}");
    }
}

#[test]
fn corpus_inline_raw_documentation_tail_cannot_supply_a_module_close() {
    // Exact-source family: the multilingual ListPages tutorials wrap syntax
    // names in paired @@ spans after an otherwise complete argumentless
    // opener, then put the apparent module close after another @@ marker.
    // Live Wikidot leaves that opener literal and continues rendering the
    // document. A later ordinary module must remain independently visible.
    let source = concat!(
        "[[module ListPages]]@@. @@[[html]]@@ prose\n",
        "more @@[[html]]@@ prose\n",
        "@@[[/module]]\n",
        "[[module ListPages name=\"later\"]]ROW[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert!(modules[0].preserve_original, "{modules:#?}");
    assert_eq!(modules[0].original, "[[module ListPages]]");
    assert_eq!(modules[1].head, "name=\"later\"");
    assert_eq!(modules[1].body, "ROW");
    assert!(!modules[1].preserve_original, "{modules:#?}");

    for source in [
        "[[module ListPages]]}}documentation",
        "[[module ListPages]]@@}}documentation",
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{modules:#?}");
        assert!(modules[0].preserve_original, "{modules:#?}");
        assert_eq!(modules[0].original, "[[module ListPages]]");
    }

    let repeated_examples = concat!(
        "[[Module Listpages]]}}first {{[[Module Listpages]]}} second\n",
        "@@[[module ListPages name=\"later\"]]@@\n",
        "@@ROW@@\n",
        "@@[[/module]]@@",
    );
    let modules = find_list_pages_module_matches(repeated_examples);
    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert!(modules[0].preserve_original, "{modules:#?}");
    assert!(modules[1].preserve_original, "{modules:#?}");
    assert_eq!(modules[0].original, "[[Module Listpages]]");
    assert_eq!(modules[1].original, "[[Module Listpages]]");

    // A single ordinary inline-raw body remains a complete module. The
    // campaign evidence only rejects the multi-span documentation tail.
    let ordinary = "[[module ListPages name=\"ordinary\"]]@@literal@@[[/module]]";
    let modules = find_list_pages_module_matches(ordinary);
    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(modules[0].head, "name=\"ordinary\"");
    assert_eq!(modules[0].body, "@@literal@@");

    for executable in [
        "[[module ListPages]]@@ONE@@ + @@TWO@@[[/module]]",
        concat!(
            "[[module ListPages limit=\"4\"]]@@\n",
            "@@%%title_linked%%@@\n",
            "@@[[/module]]",
        ),
    ] {
        let modules = find_list_pages_module_matches(executable);
        assert_eq!(modules.len(), 1, "{modules:#?}");
        assert!(!modules[0].preserve_original, "{modules:#?}");
    }

    let formatted_author_template = concat!(
        "[[module ListPages created_by=\"@@**your name**@@\" ",
        "limit=\"4\" wrapper=\"no\" separate=\"yes\"]]@@\n",
        "@@**%%title_linked%%** [+%%rating%%]@@\n",
        "@@{{**Created:**%%created_at%%}}@@\n",
        "@@[[/module]]",
    );
    let modules = find_list_pages_module_matches(formatted_author_template);
    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(
        modules[0].end,
        formatted_author_template.len(),
        "{modules:#?}"
    );
    assert_eq!(modules[0].original, formatted_author_template);
    assert!(!modules[0].preserve_original, "{modules:#?}");

    let preprocessed_content_module = concat!(
        "[[module ListPages created_by=\"@URL\" limit=\"1\"]]@@\n",
        "@@[[%%content{0}%%module css]]@@\n",
        "@@.row { display: block; }@@\n",
        "@@[[%%content{0}%%/module]]@@\n",
        "@@[[/module]]",
    );
    let modules = find_list_pages_module_matches(preprocessed_content_module);
    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(modules[0].end, modules[0].body_start, "{modules:#?}");
    assert_eq!(
        modules[0].original,
        r#"[[module ListPages created_by="@URL" limit="1"]]"#,
    );
}

#[test]
fn corpus_unclosed_listpages_body_owns_legacy_quoted_continuation() {
    let source = concat!(
        "[[module ListPages offset=\"@URL|0\" range=\".\"]]@@\n",
        "> @@%%content{2}%%@@\n",
        "> @@[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "offset=\"@URL|0\" range=\".\"");
    assert_eq!(modules[0].body, "@@\n> @@%%content{2}%%@@\n> @@[[/module]]",);
    assert_eq!(modules[0].original, source);
    assert_eq!(modules[0].end, source.len());
}

#[test]
fn corpus_complete_argumentless_listpages_openers_execute_at_eof() {
    for source in ["[[module ListPages]]", "[[Module Listpages]]"] {
        let modules = find_list_pages_module_matches(source);

        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
        assert_eq!(modules[0].head, "");
        assert_eq!(modules[0].body, "");
        assert_eq!(modules[0].original, source);
        assert_eq!(modules[0].end, source.len());
    }
}

#[test]
fn corpus_complete_runtime_unsafe_listpages_openers_execute_at_eof() {
    for source in [
        concat!(
            "[[module ListPages created_by=\"morhadow\" tags=\"es\" ",
            "order=\"rating\" limit=\"5\" separate=\"no\"@@]]",
        ),
        concat!(
            "[[module ListPages created_by=\"여기다 이름을 적으시오\" separate=\"no\" ",
            "limit=\"250\" perPage=\"250\"tags=\"농담, -이야기, -번역\" order=\"title\"]]",
        ),
    ] {
        let modules = find_list_pages_module_matches(source);

        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
        assert_eq!(modules[0].body, "");
        assert_eq!(modules[0].original, source);
        assert_eq!(modules[0].end, source.len());
    }
}

#[test]
fn corpus_single_bracket_head_with_a_dangling_quote_uses_the_default_template() {
    let source = concat!(
        "[[module ListPages tags=\"+hiscon2017\" perPage=\"100\" order=\"评分：]\n",
        "* **%%title_linked%%**\n",
        "[[/module]]\n",
        "TAIL",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(
        modules[0].head,
        "tags=\"+hiscon2017\" perPage=\"100\" order=\"评分：",
    );
    assert_eq!(
        modules[0].body, "",
        "the final unclosed quote consumes the authored row and selects Wikidot's default template",
    );
    assert!(
        modules[0].consume_empty_tail,
        "a recovered default-template head must own its authored closing module",
    );
    assert_eq!(
        modules[0].original,
        concat!(
            "[[module ListPages tags=\"+hiscon2017\" perPage=\"100\" order=\"评分：]\n",
            "* **%%title_linked%%**\n",
            "[[/module]]",
        ),
    );
}

#[test]
fn corpus_legacy_list_pages_heads_remain_structurally_visible() {
    for source in [
        concat!(
            "[[module ListPages created_by=\"creambox\" order=\"random\"tags=\"+原创 +scp\" separate=\"no\"]]\n",
            "%%title_linked%%\n[[/module]]",
        ),
        concat!(
            "[[module ListPages created_at<\"2018.12.26\" updated_at>=\"2021.2.16\" ",
            "rating>=\"-7\" votes!=\"0\"]]\n",
            "%%title_linked%%\n[[/module]]",
        ),
        concat!(
            "[[module ListPages separate=\"no\" tags=\"+cn3000 -竞赛\" perPage=\"100\" order=\"random\" ]]]\n",
            "%%title_linked%%\n[[/module]]",
        ),
        concat!(
            "[[module ListPages order=\"title\" tags=\"+_herman-fuller -scp\" separate=\"no\" ",
            "order=\"title asc perPage=\"250]]\n%%title_linked%%\n[[/module]]",
        ),
        concat!(
            "[[module ListPages range=\"@URL|.\" rating=\">15\" tags=\"+scp\" ",
            "order=\"rating desc\" separate=\"false\" date=\"@URL\"]]\n",
            "%%title_linked%%\n[[/module]]",
        ),
        concat!(
            "[[module ListPages tags=\"{$tag}\" created_by=\"{$user}\" order=\"rating desc\" ",
            "limit=\"1\" separate=\"no\" wrapper=\"no\"]]\n",
            "%%content{0}%%\n[[/module]]",
        ),
        "[[module ListPages 属性...]]\n模块主体\n[[/module]]",
        "[[module ListPages 任意属性...]]\n模块主体\n[[/module]]",
        "[[module ListPages ???]]\n模块主体\n[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
    }
}

#[test]
fn documented_placeholder_head_reaches_the_direct_scanner() {
    let source = "[[module ListPages 属性...]]\n模块主体\n[[/module]]";
    let lowercase = source.to_ascii_lowercase();
    let ListPagesScannerLiteralIndexes { direct, .. } =
        LiteralRegionIndex::new_list_pages_scanner_indexes(source, None)
            .expect("fixture should stay within the scanner work budget");
    assert!(
        direct.containing_range(0).is_none(),
        "the exact runtime head must not be claimed by a literal owner",
    );
    let (events, _, _, ambiguous) =
        collect_module_events(ModuleEventScanner::new(source, &lowercase, &direct));
    assert!(!ambiguous);
    assert!(
        matches!(
            events.first(),
            Some(ModuleEvent::Open {
                direct_candidate: true,
                ..
            }),
        ),
        "the exact runtime head must reach the structural event stream",
    );

    let projection = ListPagesSourceProjection::new(source)
        .expect("the ASCII ellipsis has a pinned typography projection");
    let projected = projection.source();
    let projected_lowercase = projected.to_ascii_lowercase();
    let ListPagesScannerLiteralIndexes {
        projected: projected_literals,
        ..
    } = LiteralRegionIndex::new_list_pages_scanner_indexes(source, Some(&projection))
        .expect("fixture should stay within the scanner work budget");
    let projected_literals =
        projected_literals.expect("a source projection has a projected literal index");
    let (projected_events, _, _, projected_ambiguous) = collect_module_events(
        ModuleEventScanner::new(projected, &projected_lowercase, &projected_literals),
    );
    assert!(!projected_ambiguous);
    assert!(
        matches!(
            projected_events.first(),
            Some(ModuleEvent::Open {
                direct_candidate: true,
                ..
            }),
        ),
        "{projected:?}: {projected_events:#?}"
    );
}

#[test]
fn corpus_at_marker_footnote_tail_executes_as_default_list_pages() {
    let source = "[[module Listpages @@以降という認識で良い。 [[/footnote]]";
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(modules[0].start, 0);
    assert_eq!(modules[0].body_start, source.len());
    assert_eq!(modules[0].end, source.len());
    assert_eq!(modules[0].head, "");
    assert_eq!(modules[0].body, "");
    assert_eq!(modules[0].original, source);
}

#[test]
fn at_marker_footnote_tail_does_not_hide_a_later_valid_module() {
    let malformed = "[[module Listpages @@以降という認識で良い。 [[/footnote]]";
    let valid = concat!(
        "[[module ListPages name=\"later-valid\"]]\n",
        "%%fullname%%\n",
        "[[/module]]",
    );
    let source = format!("{malformed}\n{valid}");
    let modules = find_list_pages_module_matches(&source);

    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert_eq!(modules[0].start, 0);
    assert_eq!(modules[0].body_start, malformed.len());
    assert_eq!(modules[0].end, malformed.len());
    assert_eq!(modules[0].head, "");
    assert_eq!(modules[0].body, "");
    assert_eq!(modules[0].original, malformed);
    assert_eq!(modules[1].start, malformed.len() + 1);
    assert_eq!(modules[1].head, r#"name="later-valid""#);
    assert_eq!(modules[1].body, "\n%%fullname%%\n");
    assert_eq!(modules[1].original, valid);
}

#[test]
fn corpus_unclosed_at_marker_body_owns_the_first_collapsible_opening() {
    // Anonymous PagePreviewModule evidence captured 2026-08-01 shows that
    // Wikidot executes this unclosed ListPages opener with its default
    // template, consumes the raw-marker documentation prefix through the
    // first complete collapsible opening, then resumes at its body.
    let consumed = concat!(
        "[[module ListPages fullname=\"@@##red|missing-page##@@\" ",
        "separate=\"yes\" limit=\"250\"]]@@\n",
        "documentation\n",
        "> @@[[module ListPages fullname=\"@@##red|example##@@\"]]@@\n",
        "[[collapsible show=\"+ Syntax\" hide=\"- Syntax\"]]",
    );
    let later = concat!(
        "\nVISIBLE\n[[/collapsible]]\n",
        "[[module ListPages name=\"later-valid\"]]ROW[[/module]]",
    );
    let source = format!("{consumed}{later}");
    let modules = find_list_pages_module_matches(&source);

    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert_eq!(modules[0].start, 0);
    assert_eq!(
        modules[0].head,
        r#"fullname="@@##red|missing-page##@@" separate="yes" limit="250""#,
    );
    assert_eq!(modules[0].body, "");
    assert_eq!(modules[0].end, consumed.len());
    assert_eq!(modules[0].original, consumed);
    assert!(!modules[0].preserve_original);
    assert_eq!(
        modules[1].start,
        consumed.len() + "\nVISIBLE\n[[/collapsible]]\n".len()
    );
    assert_eq!(modules[1].head, r#"name="later-valid""#);
    assert_eq!(modules[1].body, "ROW");

    for unsupported in [
        concat!(
            "[[module ListPages name=\"missing\"]]@@\n",
            "documentation without a collapsible\n@@",
        ),
        concat!(
            "[[module ListPages name=\"missing\"]]@@\n",
            "documentation\n",
            "[[collapsible show=\"unterminated\" hide=\"- Syntax\"]\n",
            "VISIBLE",
        ),
        concat!(
            "[[module ListPages name=\"missing\"]]@@\n",
            "documentation\n",
            "[[collapsible show=\"+ Syntax\"]]\n",
            "VISIBLE",
        ),
    ] {
        let modules = find_list_pages_module_matches(unsupported);
        assert_eq!(modules.len(), 1, "{unsupported:?}: {modules:#?}");
        assert!(
            modules[0].preserve_original,
            "{unsupported:?}: {modules:#?}"
        );
        assert!(
            modules[0].preserve_as_module654,
            "{unsupported:?}: {modules:#?}"
        );
        assert_eq!(modules[0].end, modules[0].body_start);
    }
}

#[test]
fn corpus_raw_footnote_head_owns_prose_through_the_first_collapsible_opening() {
    // Exact-source boundary from
    // jp:advanced-formatting-and-you:L341:B11200. The malformed ListPages
    // head closes on the raw footnote closer. Live consumes the prose and
    // first collapsible opening, then resumes at the collapsible body.
    let consumed = concat!(
        "[[module Listpages @@以降という認識で良い。 [[/footnote]]",
        "には、表示させたいものを指定します。\n\n",
        "ListPagesモジュールが沢山あるとページの表示速度が著しく低下します。\n\n",
        "ListPagesモジュールで選択可能な条件の1例を次に示します。\n",
        "[[collapsible show=\"選択可能な条件\" hide=\"Hide\"]]",
    );
    let suffix = concat!(
        "\n[[div class=\"first\"]]VISIBLE-BODY[[/div]]\n",
        "[[/collapsible]]\n",
        "[[module ListPages name=\"later-valid\"]]ROW[[/module]]",
    );
    let source = format!("{consumed}{suffix}");
    let modules = find_list_pages_module_matches(&source);

    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert_eq!(modules[0].body, "");
    assert_eq!(modules[0].end, consumed.len(), "{modules:#?}");
    assert!(modules[0].consume_empty_tail);
    assert!(!modules[0].preserve_original);
    assert_eq!(modules[1].head, r#"name="later-valid""#);
    assert_eq!(modules[1].body, "ROW");
}

#[test]
fn unclosed_at_marker_preservation_does_not_hide_a_later_valid_module() {
    // When no evidenced boundary follows, Wikidot leaves the opener visible
    // as legacy module654 text. A later module after the closed raw span
    // remains independently executable.
    let preserved = "[[module ListPages name=\"missing\"]]";
    let raw_tail = "@@\nDOC\n@@\n";
    let later = "[[module ListPages name=\"later-valid\"]]ROW[[/module]]";
    let source = format!("{preserved}{raw_tail}{later}");
    let modules = find_list_pages_module_matches(&source);

    assert_eq!(modules.len(), 2, "{modules:#?}");
    assert_eq!(modules[0].original, preserved);
    assert!(modules[0].preserve_original);
    assert!(modules[0].preserve_as_module654);
    assert_eq!(modules[1].start, preserved.len() + raw_tail.len());
    assert_eq!(modules[1].head, r#"name="later-valid""#);
    assert_eq!(modules[1].body, "ROW");
}

#[test]
fn corpus_trailing_at_marker_head_keeps_its_evidenced_raw_row_tail_executable() {
    // Exact-source case es:listpages-magic-and-you:L177:B6617. Live executes
    // the default ListPages query, and the malformed raw row/close tail
    // contributes no downstream output.
    let source = concat!(
        "[[module ListPages created_by=\"morhadow\" tags=\"es\" ",
        "order=\"rating\" limit=\"5\" separate=\"no\"@@]]@@\n",
        "@@*@@ %%title_linked%% (+%%rating%%)\n",
        "@@[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert_eq!(
        modules[0].head,
        concat!(
            "created_by=\"morhadow\" tags=\"es\" ",
            "order=\"rating\" limit=\"5\" separate=\"no\"@@",
        ),
    );
    assert_eq!(modules[0].body, "");
    assert_eq!(modules[0].end, source.len());
    assert!(!modules[0].preserve_original);
    assert!(!modules[0].preserve_as_module654);
    assert!(modules[0].consume_empty_tail);
}

#[test]
fn unquoted_comparison_discriminators_remain_structurally_executable() {
    // Frozen Wikidot oracle: unquoted-comparison-live.jsonl. These observed
    // inert tokens must not force an executable module into literal fallback.
    for token in [
        "rating>100000",
        "score>100000",
        "votes>100000",
        "created_at>2100",
        "createdat>2100",
        "date>2100",
        "name!=definitely-missing",
        "parent>=component:image-block",
        "limit>=1",
        "offset!=1",
    ] {
        let head = format!(r#"fullname="scp-002" {token}"#);
        assert!(
            list_pages_runtime_head_can_execute(&head),
            "the evidenced inert comparison token must not invalidate {head:?}",
        );
        let source =
            format!("[[module ListPages {head}]]\nROW=%%fullname%%\n[[/module]]");
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
        assert!(modules[0].runtime_safe, "{source:?}: {modules:#?}");
    }
}

#[test]
fn live_inert_list_pages_head_tokens_remain_structurally_visible() {
    for head in [
        r#"| name="target" limit="1" order="name""#,
        r#"size name="target" limit="1" order="name""#,
        r#"name="target" limit="1" order="name" prependLine="#,
        r#"name="target" limit="1" order="name" appendLine="#,
        r#"name="target" appendLine= appendLine="POST""#,
        r#"name="target" appendLine="POST" appendLine="#,
        r#"name="target" prependLine= prependLine="PRE""#,
        r#"name="target" limit="1" order="name"@@"#,
        r#"| only-inert-tokens"#,
    ] {
        let source = format!("[[module ListPages {head}]]ROW|%%fullname%%[[/module]]");
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{source:?}: {modules:#?}");
        assert_eq!(modules[0].head, head);
    }

    for head in [
        r#"size prependLine="#,
        r#"name="target" [[module ListUsers]]"#,
    ] {
        let source = format!("[[module ListPages {head}]]ROW|%%fullname%%[[/module]]");
        assert!(
            find_list_pages_module_matches(&source).is_empty(),
            "unsupported malformed head must fail closed: {source:?}",
        );
    }
}

#[test]
fn list_pages_scanner_keeps_adjacent_custom_listing_modules() {
    let source = r#"[[div class="top-border"]]
[[/div]]
[[div style="text-align: center;"]]
+ SCPs
[[module Listpages created_by="=" category="-fragment" tag="+scp -co-authored" limit="1" order="random"]]
([[[*%%link%%|Random]]])
[[/module]]
-----
[[/div]]

[[module Listpages created_by="=" order="" category="-fragment" tag="+scp -co-authored" perPage="250"]]
[[div class="content-box no"]]
++ **%%title_linked%%**
[[div class="content-section"]]
------
**Rating:** +%%rating%%
**Comments:** %%comments%%
**+/- :** +[[#expr ((%%rating%%+%%rating_votes%%)/2)]]/-[[#expr ((%%rating_votes%%-%%rating%%)/2)]]
[[/div]]
[[/module]]"#;

    let second_start = source.rfind("[[module Listpages").unwrap();
    assert_eq!(
        find_list_pages_module_matches(&source[..second_start]).len(),
        1
    );
    assert_eq!(
        find_list_pages_module_matches(&source[second_start..]).len(),
        1
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
}

#[test]
fn linear_list_pages_scanner_ignores_literal_module_tokens() {
    let source = concat!(
        "@@[[module ListPages name=\"inline-literal\"]]@@\n",
        "stray literal example body[[/module]]\n",
        "[[code]]\n",
        "[[module ListPages name=\"block-literal\"]]ignored[[/module]]\n",
        "@@\n",
        "[!--\n",
        "[[/code]]\n",
        "[[module ListPages name=\"live\"]]",
        "before @@[[module ListUsers]][[/module]]@@ after",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(
        modules[0].body,
        "before @@[[module ListUsers]][[/module]]@@ after",
    );

    for body in [
        "before @@[[/module]]@@ after",
        "before @@[[module ListUsers]]@@ after",
    ] {
        let source = format!("[[module ListPages name=\"live\"]]{body}[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "literal token changed nesting: {body}");
        assert_eq!(modules[0].body, body);
    }
}

#[test]
fn corpus_monospace_and_comment_owners_hide_listpages_candidates() {
    for source in [
        "{{[[Module Listpages]]}}",
        concat!(
            "[!--\n",
            "[[module ListPages rating=\">100\" order=\"rating desc\" ",
            "separate=\"false\" limit=\"1000\" perPage=\"1000\"]]\n",
            "%%title_linked%%:: rating: %%rating%%\n",
            "[[/module]]\n\n",
            "---]",
        ),
        concat!(
            "[!----\n",
            "temporary hidden region\n",
            "[[module ListPages order=\"updated_at\" category=\"*\" ",
            "perPage=\"200\" separate=\"false\"]]\n",
            "%%title_linked%%\n",
            "[[/module]]\n",
            "---]",
        ),
    ] {
        assert!(
            find_list_pages_module_matches(source).is_empty(),
            "literal owner leaked a ListPages candidate:\n{source}",
        );
    }
}
#[test]
fn preserved_module654_suffix_charges_projected_merge_and_recursive_work() {
    // The unclosed head and raw-tail preservation is a retained live Wikidot
    // behavioral oracle (unclosed_at_marker_preservation_does_not_hide_a_later_valid_module).
    // Quoted-line continuation extensions below test internal work counters;
    // their exact renderings are NOT independently verified live Wikidot oracles.
    let preserved = "[[module ListPages name=\"missing\"]]";
    let raw_tail = "@@\nDOC\n@@\n";
    let later = "[[module ListPages name=\"later-valid\"]]ROW[[/module]]";
    for (label, prefix, gap, expected_work, expected_advances) in [
        ("no projection", "", "", 593, 4),
        ("quoted prefix", "> quoted\\\n> second row\n", "", 1_522, 9),
        ("quoted suffix", "", "> quoted\\\n> second row\n", 2_097, 14),
        (
            "literal and quoted prefix",
            "@@hidden@@\n> quoted\\\n> second row\n",
            "",
            1_569,
            12,
        ),
    ] {
        let source = format!("{prefix}{preserved}{raw_tail}{gap}{later}");
        let (modules, work, advances) =
            find_list_pages_module_matches_with_cursor_work(&source);
        assert_eq!(modules.len(), 2, "{label}: preserve and resume");
        assert_eq!(modules[0].start, prefix.len(), "{label}");
        assert_eq!(
            modules[0].body_start,
            prefix.len() + preserved.len(),
            "{label}"
        );
        assert_eq!(modules[0].end, modules[0].body_start, "{label}");
        assert!(
            modules[0].preserve_original && modules[0].preserve_as_module654,
            "{label}"
        );
        assert_eq!(modules[0].original, preserved, "{label}");
        assert_eq!(
            modules[1].start,
            prefix.len() + preserved.len() + raw_tail.len() + gap.len(),
            "{label}"
        );
        assert_eq!(modules[1].end, source.len(), "{label}");
        assert_eq!(modules[1].body, "ROW", "{label}");
        assert_eq!(modules[1].original, later, "{label}");
        assert_eq!(
            advances, expected_advances,
            "{label}: literal traversal total"
        );
        assert_eq!(
            work, expected_work,
            "{label}: direct + projected + advances + merge + suffix"
        );
        assert!(
            work <= source.len() * MAX_SINGLE_SCANNER_WORK_MULTIPLIER * 2,
            "{label}"
        );
    }
}
