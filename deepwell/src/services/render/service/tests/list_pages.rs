use super::*;

fn list_pages_substitution_context<'a>(
    rendered_limit: usize,
    user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
    page_wikitext: Option<&'a str>,
    data_form_values: &'a BTreeMap<String, String>,
) -> ListPagesSubstitutionContext<'a> {
    list_pages_substitution_context_with_mode(
        rendered_limit,
        user_displays,
        empty_list_pages_snapshot_displays(),
        page_wikitext,
        data_form_values,
        false,
    )
}

fn list_pages_substitution_context_with_mode<'a>(
    rendered_limit: usize,
    user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    page_wikitext: Option<&'a str>,
    data_form_values: &'a BTreeMap<String, String>,
    render_generated_html: bool,
) -> ListPagesSubstitutionContext<'a> {
    ListPagesSubstitutionContext {
        authored_limit: Some(rendered_limit as u64),
        ajax_module_response: false,
        page_preview: true,
        site: "scp-wiki",
        site_title: "SCP Wiki",
        category: "",
        tag_target: None,
        user_displays,
        snapshot_displays,
        runtime_displays: empty_list_pages_runtime_displays(),
        page_wikitext,
        page_rendered_content: None,
        page_rendered_summary: None,
        page_rendered_summary_is_block: false,
        default_summary_first_paragraph: false,
        fallback_link_titles: None,
        page_rendered_first_paragraph: None,
        page_compiled_body_html: page_wikitext,
        page_wikitext_scalar_count: page_wikitext
            .map(|wikitext| wikitext.chars().count()),
        page_parent_fullname: None,
        page_parent_display: None,
        page_child_count: None,
        page_revision_count: None,
        data_form_values,
        data_form_definition: None,
        render_generated_html,
    }
}

fn empty_list_pages_snapshot_displays() -> &'static BTreeMap<i64, ListPagesSnapshotDisplay>
{
    static EMPTY: std::sync::LazyLock<BTreeMap<i64, ListPagesSnapshotDisplay>> =
        std::sync::LazyLock::new(BTreeMap::new);
    &EMPTY
}

fn empty_list_pages_runtime_displays() -> &'static BTreeMap<i64, ListPagesRuntimeDisplay>
{
    static EMPTY: std::sync::LazyLock<BTreeMap<i64, ListPagesRuntimeDisplay>> =
        std::sync::LazyLock::new(BTreeMap::new);
    &EMPTY
}

/// A request that carried only a `tag` path argument.
fn url_tag(tag: Option<&str>) -> UrlArguments<'_> {
    UrlArguments {
        tag,
        ..UrlArguments::default()
    }
}

/// A request that carried only a `category` path argument.
fn url_category(category: Option<&str>) -> UrlArguments<'_> {
    UrlArguments {
        category,
        ..UrlArguments::default()
    }
}

/// A request that carried only an `offset` path argument.
fn url_offset(offset: Option<u32>) -> UrlArguments<'static> {
    UrlArguments {
        offset,
        ..UrlArguments::default()
    }
}

#[test]
fn a_url_tag_selector_resolves_to_the_requests_tag() {
    let arguments = parse_list_pages_arguments_with_url(
        r#" tags="@URL""#,
        url_tag(Some("golem-of-prague")),
    )
    .expect("url tag selector should parse");

    assert_eq!(arguments.default_tags, vec!["golem-of-prague"]);
}

#[test]
fn a_url_tag_selector_beats_its_own_fallback() {
    let arguments = parse_list_pages_arguments_with_url(
        r#" tags="@URL|_""#,
        url_tag(Some("golem-of-prague")),
    )
    .expect("url tag selector should parse");

    assert_eq!(arguments.default_tags, vec!["golem-of-prague"]);
}

#[test]
fn a_url_tag_selector_without_a_tag_falls_back() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" tags="@URL|_""#, url_tag(None))
            .expect("url tag selector should parse");

    assert_eq!(arguments.default_tags, vec!["_"]);
}

#[test]
fn an_unresolved_url_tag_selector_widens_rather_than_matching_nothing() {
    // Live lists the whole site here rather than rendering an empty list,
    // which is why `system:page-tags` writes the `|_` fallback.
    for tag in [None, Some("")] {
        let arguments =
            parse_list_pages_arguments_with_url(r#" tags="@URL""#, url_tag(tag))
                .expect("url tag selector should parse");

        assert!(arguments.default_tags.is_empty());
        assert!(arguments.all_tags.is_empty());
        assert!(arguments.no_tags.is_empty());
    }
}

#[test]
fn an_empty_url_tag_still_takes_the_fallback() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" tags="@URL|_""#, url_tag(Some("")))
            .expect("url tag selector should parse");

    assert_eq!(arguments.default_tags, vec!["_"]);
}

#[test]
fn a_resolved_url_tag_keeps_count_pages_literal() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" tags="@URL""#, url_tag(Some("alpha")))
            .expect("url tag selector should parse");

    assert!(arguments.unsupported_count_pages_filter);
}

#[test]
fn a_static_tags_selector_ignores_the_url_tag() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" tags="alpha""#, url_tag(Some("beta")))
            .expect("static tags selector should parse");

    assert_eq!(arguments.default_tags, vec!["alpha"]);
}

#[test]
fn a_url_offset_selector_resolves_to_the_requests_offset() {
    for offset in [0, 1, MAX_LISTPAGES_RENDER_OFFSET] {
        let arguments = parse_list_pages_arguments_with_url(
            r#" offset="@URL|0""#,
            url_offset(Some(offset)),
        )
        .expect("url offset selector should parse");

        assert_eq!(arguments.offset, offset);
        assert_eq!(arguments.offset_origin, ListPagesOffsetOrigin::Url);
    }
}

#[test]
fn a_url_offset_selector_falls_back_without_a_valid_request_offset() {
    for offset in [None, Some(MAX_LISTPAGES_RENDER_OFFSET + 1)] {
        let arguments = parse_list_pages_arguments_with_url(
            r#" offset="@URL|3""#,
            url_offset(offset),
        )
        .expect("url offset fallback should parse");

        assert_eq!(arguments.offset, 3);
        assert_eq!(arguments.offset_origin, ListPagesOffsetOrigin::Fallback);
    }
}

#[test]
fn a_url_offset_selector_without_an_authored_fallback_defaults_to_zero() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" offset="@URL""#, url_offset(None))
            .expect("bare url offset selector should parse");

    assert_eq!(arguments.offset, 0);
    assert_eq!(arguments.offset_origin, ListPagesOffsetOrigin::Fallback);
}

#[test]
fn a_static_offset_ignores_the_request_offset() {
    let arguments =
        parse_list_pages_arguments_with_url(r#" offset="2""#, url_offset(Some(7)))
            .expect("static offset should parse");

    assert_eq!(arguments.offset, 2);
    assert_eq!(arguments.offset_origin, ListPagesOffsetOrigin::Static);
}

#[test]
fn only_url_offset_content_counts_toward_the_extended_deadline() {
    let replacement = "expanded child content";

    assert_eq!(
        url_offset_list_pages_content_bytes(
            ListPagesOffsetOrigin::Url,
            true,
            replacement,
        ),
        replacement.len(),
    );
    for offset_origin in [
        ListPagesOffsetOrigin::Static,
        ListPagesOffsetOrigin::Fallback,
    ] {
        assert_eq!(
            url_offset_list_pages_content_bytes(offset_origin, true, replacement),
            0,
        );
    }
    assert_eq!(
        url_offset_list_pages_content_bytes(
            ListPagesOffsetOrigin::Url,
            false,
            replacement,
        ),
        0,
    );
}

#[test]
fn ajax_listpages_routes_positive_p_as_request_state() {
    let parameters = BTreeMap::from([
        ("category".to_owned(), "doc".to_owned()),
        ("p".to_owned(), "2".to_owned()),
        ("perPage".to_owned(), "10".to_owned()),
    ]);
    let request =
        build_wikidot_list_pages_module_request("%%fullname%%".to_owned(), &parameters)
            .expect("the live-observed positive AMC p parameter should be accepted");

    assert_eq!(request.page, Some(2));
    assert!(!request.source.contains(" p="));
    assert!(request.source.contains(" category=\"doc\""));
    assert!(request.source.contains(" perPage=\"10\""));

    for invalid in ["", "0", "-1", "1.5", "4294967296"] {
        let parameters = BTreeMap::from([("p".to_owned(), invalid.to_owned())]);
        assert!(
            build_wikidot_list_pages_module_request(String::new(), &parameters).is_none(),
            "unobserved AMC p={invalid:?} must fail closed",
        );
    }
}

#[test]
fn ajax_listpages_source_accepts_only_balanced_nested_modules() {
    let parameters = BTreeMap::from([
        ("limit".to_owned(), "1".to_owned()),
        ("name".to_owned(), "outer-row".to_owned()),
    ]);
    let nested_body = concat!(
        "OUTER_BEFORE %%fullname%%\n",
        "[[module ListPages name=\"inner-row\" limit=\"1\"]]\n",
        "INNER_ROW %%fullname%%\n",
        "[[/module]]\n",
        "OUTER_AFTER %%fullname%%",
    );

    let source =
        build_wikidot_list_pages_module_source(nested_body.to_owned(), &parameters)
            .expect("live Wikidot accepts a balanced nested ListPages body");
    let matches = find_list_pages_module_matches(&source);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].start, 0);
    assert_eq!(matches[0].end, source.len());
    assert_eq!(matches[0].body.trim(), nested_body);

    for malformed in [
        "BEFORE [[/module]] AFTER",
        "[[module ListPages name=\"inner-row\"]] UNCLOSED",
    ] {
        let source =
            build_wikidot_list_pages_module_source(malformed.to_owned(), &parameters)
                .expect("live Wikidot renders an unbalanced body marker literally");
        let matches = find_list_pages_module_matches(&source);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, source.len());
        assert!(
            matches[0].body.contains(AJAX_MODULE_LITERAL_MARKER_PREFIX),
            "the unbalanced marker should be replaced with an inert generated token",
        );
        assert!(
            !matches[0].body.contains("[[module")
                && !matches[0].body.contains("[[/module]]"),
            "the unbalanced marker must not participate in outer module scanning",
        );
    }
}

#[test]
fn a_url_category_selector_resolves_to_the_requests_category() {
    let arguments = parse_list_pages_arguments_with_url(
        r#" category="@URL""#,
        url_category(Some("wjcatzone")),
    )
    .expect("url category selector should parse");

    assert_eq!(arguments.categories, vec!["wjcatzone"]);
}

#[test]
fn a_url_category_selector_beats_its_own_fallback() {
    let arguments = parse_list_pages_arguments_with_url(
        r#" category="@URL|_default""#,
        url_category(Some("wjcatzone")),
    )
    .expect("url category selector should parse");

    assert_eq!(arguments.categories, vec!["wjcatzone"]);
}

#[test]
fn a_url_category_selector_without_a_category_falls_back() {
    let arguments = parse_list_pages_arguments_with_url(
        r#" category="@URL|_default""#,
        url_category(None),
    )
    .expect("url category selector should parse");

    assert_eq!(arguments.categories, vec!["_default"]);
}

#[test]
fn an_unresolved_url_category_selector_names_no_category() {
    // Live drops the constraint, which for `category` means the module's own
    // default rather than every category. Dropping is not matching everything.
    for category in [None, Some("")] {
        let arguments = parse_list_pages_arguments_with_url(
            r#" category="@URL""#,
            url_category(category),
        )
        .expect("url category selector should parse");

        assert!(arguments.categories.is_empty());
        assert!(arguments.excluded_categories.is_empty());
    }
}

#[test]
fn a_url_tag_does_not_resolve_a_category_selector() {
    // Each selector reads the path argument of its own name.
    let arguments = parse_list_pages_arguments_with_url(
        r#" category="@URL""#,
        url_tag(Some("alpha")),
    )
    .expect("url category selector should parse");

    assert!(arguments.categories.is_empty());
}

#[test]
fn parses_current_page_list_pages_name_selector() {
    let arguments = parse_list_pages_arguments(r#" name="=" limit="20""#)
        .expect("current page selector should parse");

    assert!(arguments.current_page_only);
}

#[test]
fn parses_current_page_list_pages_range_selector() {
    let arguments = parse_list_pages_arguments(r#" range=".""#)
        .expect("current page range selector should parse");

    assert!(arguments.current_page_only);
    assert_eq!(arguments.limit, Some(1));
}

#[test]
fn builds_current_page_list_pages_metadata_without_database_fields() {
    let page_info = ftml::data::PageInfo {
        page: Cow::Borrowed("some-page"),
        category: None,
        site: Cow::Borrowed("sandbox"),
        title: Cow::Borrowed("A page for the age"),
        alt_title: None,
        score: ftml::data::ScoreValue::Float(69.0),
        tags: vec![Cow::Borrowed("tale"), Cow::Borrowed("_cc")],
        language: Cow::Borrowed("default"),
    };
    let fields = FoundPageFields {
        title: true,
        alt_title: true,
        slug: true,
        tags: true,
        score: true,
        ..Default::default()
    };

    let row = current_page_info_list_pages_row(7, 11, &page_info, &fields)
        .expect("PageInfo-backed fields should not require a database load");

    assert_eq!(row.site_id, 7);
    assert_eq!(row.page_id, 11);
    assert_eq!(row.title.as_deref(), Some("A page for the age"));
    assert_eq!(row.slug.as_deref(), Some("some-page"));
    assert_eq!(
        row.tags.as_deref(),
        Some(["tale".to_owned(), "_cc".to_owned()].as_slice())
    );

    let database_fields = FoundPageFields {
        created_at: true,
        ..Default::default()
    };
    assert!(
        current_page_info_list_pages_row(7, 11, &page_info, &database_fields).is_none()
    );
}

#[test]
fn parses_other_pages_list_pages_range_selector() {
    let arguments = parse_list_pages_arguments(
        r#" category="*" created_by="=" tags="scp" perPage="15" range="others""#,
    )
    .expect("other pages range selector should parse");

    assert!(!arguments.current_page_only);
    assert!(arguments.exclude_current_page);
    assert_eq!(arguments.count_pages_per_page, Some(15));
}

#[test]
fn current_page_list_pages_selection_respects_limit_and_offset() {
    assert!(should_render_current_page_list_pages_row(true, Some(1), 0));
    assert!(!should_render_current_page_list_pages_row(true, Some(0), 0));
    assert!(!should_render_current_page_list_pages_row(true, Some(1), 1));
    assert!(!should_render_current_page_list_pages_row(
        false,
        Some(1),
        0
    ));
}

#[test]
fn current_page_list_pages_score_uses_render_page_score_when_requested() {
    let mut page_info = fallback_test_page_info("rated-page", "Rated page");
    page_info.score = ftml::data::ScoreValue::Integer(49);

    assert_eq!(
        requested_page_info_score(
            &crate::services::page_query::FoundPageFields {
                score: true,
                ..Default::default()
            },
            &page_info,
        ),
        Some(49.0),
    );
    assert_eq!(
        requested_page_info_score(
            &crate::services::page_query::FoundPageFields::default(),
            &page_info,
        ),
        None,
    );
}

#[test]
fn parses_corpus_list_pages_category_and_separate_arguments() {
    let arguments = parse_list_pages_arguments(
        r#" tags="1998" category=". +theme" separate="no" order="created_at desc" limit="5""#,
    )
    .expect("corpus ListPages selector should parse");

    assert!(!arguments.category_all);
    assert!(arguments.include_current_category);
    assert_eq!(arguments.categories, vec![Cow::Borrowed("theme")]);
    assert_eq!(arguments.default_tags, vec![Cow::Borrowed("1998")]);
    assert_eq!(arguments.limit, Some(5));
}

#[test]
fn exact_name_list_pages_batch_classifier_is_deliberately_narrow() {
    let default = parse_list_pages_arguments(r#" name="scp-173""#)
        .expect("exact-name selector should parse");
    let default_template =
        ListPagesTemplatePlan::compile("%%rating%%").expect("rating body should compile");
    let key = exact_name_list_pages_batch_key(
        r#" name="scp-173""#,
        &default_template,
        &default,
        "_default",
    )
    .expect("simple exact-name block should batch");
    assert!(!key.category_all);
    assert_eq!(key.categories, ["_default"]);

    {
        let key_name = "fullname";
        let head = format!(r#" {key_name}="scp-173" category="*""#);
        let arguments = parse_list_pages_arguments(&head)
            .expect("exact full-slug selector should parse");
        let key = exact_name_list_pages_batch_key(
            &head,
            &default_template,
            &arguments,
            "_default",
        )
        .expect("exact full-slug selector should batch");
        assert!(key.category_all, "unexpected category scope for {key_name}");
        assert!(key.categories.is_empty());
    }
    for inert_alias in ["full_slug", "fullslug"] {
        let head = format!(r#" {inert_alias}="scp-173" category="*""#);
        let arguments =
            parse_list_pages_arguments(&head).expect("inert alias should not abort");
        assert_eq!(arguments.slug, None, "{inert_alias}");
        assert!(
            exact_name_list_pages_batch_key(
                &head,
                &default_template,
                &arguments,
                "_default",
            )
            .is_none(),
            "{inert_alias} must remain inert",
        );
    }

    let categorized =
        parse_list_pages_arguments(r#" category="art" name="ralliston-portrait""#)
            .expect("categorized exact-name selector should parse");
    let categorized_template =
        ListPagesTemplatePlan::compile("%%rating%%").expect("rating body should compile");
    assert!(
        exact_name_list_pages_batch_key(
            r#" category="art" name="ralliston-portrait""#,
            &categorized_template,
            &categorized,
            "_default",
        )
        .is_none(),
        "category-local names require the ordinary query's full-slug projection",
    );

    for (head, body) in [
        (r#" name="scp-173" tags="scp""#, "%%rating%%"),
        (r#" name="scp-173" limit="1""#, "%%rating%%"),
        (r#" name="scp-173" fullname="scp-173""#, "%%rating%%"),
        (r#" fullname="scp-*""#, "%%rating%%"),
        (r#" name="scp-173""#, "%%content%%"),
        (r#" name="scp-173""#, "%%form_data%%"),
        (r#" name="scp-173""#, "%%form_raw%%"),
    ] {
        let arguments = parse_list_pages_arguments(head)
            .expect("supported non-batch selector should still parse");
        assert!(
            ListPagesTemplatePlan::compile(body)
                .and_then(|template| exact_name_list_pages_batch_key(
                    head, &template, &arguments, "_default",
                ))
                .is_none(),
            "unexpectedly batchable: {head} / {body}",
        );
    }
}

#[test]
fn list_pages_batch_display_requirements_union_template_metadata() {
    let mut requirements = ListPagesBatchDisplayRequirements::default();
    requirements.include(
        &ListPagesTemplatePlan::compile("%%title_linked%% %%rating_votes%%")
            .expect("rating-vote template should compile"),
    );
    assert_eq!(
        requirements,
        ListPagesBatchDisplayRequirements {
            users: false,
            snapshots: true,
            runtime: true,
        }
    );

    requirements.include(
        &ListPagesTemplatePlan::compile("%%created_by%% %%comments%%")
            .expect("author and comments template should compile"),
    );
    assert_eq!(
        requirements,
        ListPagesBatchDisplayRequirements {
            users: true,
            snapshots: true,
            runtime: true,
        }
    );
}

#[test]
fn parses_corpus_list_pages_excluded_category_and_tag() {
    let arguments = parse_list_pages_arguments(
        r#" tags="地下東京奇譚 -ハブ" order="created_at" separate="no" category="* -deleted""#,
    )
    .expect("corpus ListPages selector with exclusions should parse");

    assert!(arguments.category_all);
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("deleted")]
    );
    assert_eq!(arguments.default_tags, vec![Cow::Borrowed("地下東京奇譚")]);
    assert_eq!(arguments.no_tags, vec![Cow::Borrowed("ハブ")]);
}

#[test]
fn list_pages_category_and_tag_alias_precedence_matches_live_wikidot() {
    let singular =
        parse_list_pages_arguments(r#" category="_default" tags="+fixture" limit="20" "#)
            .expect("singular category selector should parse");
    let plural = parse_list_pages_arguments(
        r#" categories="_default" tags="+fixture" limit="20" "#,
    )
    .expect("Wikidot categories alias should parse");

    assert_eq!(plural, singular);

    for head in [
        r#" category="_default" categories="fragment" "#,
        r#" categories="fragment" category="_default" "#,
    ] {
        let arguments = parse_list_pages_arguments(head)
            .expect("the modern category spelling takes precedence");
        assert_eq!(arguments.categories, vec![Cow::Borrowed("_default")]);
    }
    for head in [
        r#" tags="fixture" tag="legacy" "#,
        r#" tag="legacy" tags="fixture" "#,
    ] {
        let arguments =
            parse_list_pages_arguments(head).expect("the tag alias takes precedence");
        assert_eq!(arguments.default_tags, vec![Cow::Borrowed("legacy")]);
    }

    let repeated = parse_list_pages_arguments(
        r#" category="doc" category="news" tags="alpha" tags="beta" "#,
    )
    .expect("repeated selectors should parse");
    assert_eq!(repeated.categories, vec![Cow::Borrowed("news")]);
    assert_eq!(repeated.default_tags, vec![Cow::Borrowed("beta")]);
}

#[test]
fn parses_wikidot_list_pages_reverse_yes_only() {
    let arguments = parse_list_pages_arguments(
        r#" tags="+fixture" order="name asc" reverse="yes" limit="20" "#,
    )
    .expect("live-evidenced reverse=yes should parse");

    assert!(arguments.reverse);
    let arguments = parse_list_pages_arguments(r#" tags="+fixture" reverse="no" "#)
        .expect("live-evidenced reverse=no should parse as the default order");
    assert!(!arguments.reverse);
}

#[test]
fn parses_wikidot_list_pages_append_line_without_aliases() {
    let arguments = parse_list_pages_arguments(
        r#" tags="+fixture" separate="no" prependLine="PRE" appendLine="POST" "#,
    )
    .expect("live-evidenced appendLine should parse");

    assert_eq!(arguments.prepend_line.as_deref(), Some("PRE"));
    assert_eq!(arguments.append_line.as_deref(), Some("POST"));
    let alias = parse_list_pages_arguments(r#" tags="+fixture" append_line="POST" "#)
        .expect("unknown argument spellings are ignored");
    assert_eq!(alias.append_line, None);
}

#[test]
fn parses_and_serializes_live_wikidot_list_pages_rss_arguments() {
    let arguments = parse_list_pages_arguments(concat!(
        r#" category="doc" tags="+Documentation -HIDDEN" order="title desc" "#,
        r#"limit="3" rssTitle="First" rss="A&B / C+ D?" "#,
        r#"rssDescription="Description & details" rssHome="blog:_start" "#,
        r#"rssLimit="7" rssOnly="yes" "#,
    ))
    .expect("live-evidenced RSS arguments should parse");

    assert_eq!(arguments.rss_title.as_deref(), Some("A&B / C+ D?"));
    assert_eq!(
        arguments.rss_description.as_deref(),
        Some("Description & details")
    );
    assert_eq!(arguments.rss_home.as_deref(), Some("blog:_start"));
    assert_eq!(arguments.rss_limit.as_deref(), Some("7"));
    assert!(arguments.rss_only);

    let page_info = ftml::data::PageInfo {
        page: Cow::Borrowed("_ajax-module-connector"),
        category: None,
        site: Cow::Borrowed("sandbox-for-codex"),
        title: Cow::Borrowed("Preview"),
        alt_title: None,
        score: ftml::data::ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    assert_eq!(
        list_pages_feed_info_html(&page_info, &arguments).as_deref(),
        Some(concat!(
            "\n\n",
            r#"<div class="feedinfo" data-wikijump-compat-listpages-feed="1">"#,
            r#"<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>"#,
            r#"<a href="/feed/pages/category/doc/"#,
            "tags/%2Bdocumentation%2C-hidden/order/title+desc/limit/7/",
            r#"t/A%26B+%2F+C%2B+D%3F/d/Description+%26+details/h/blog%3A_start">"#,
            "RSS feed</a></div>",
            "\n\n",
        )),
    );

    let invalid_only = parse_list_pages_arguments(
        r#"rss="Feed Title" rssOnly="maybe" rssLimit="0" limit="1""#,
    )
    .expect("live treats invalid rssOnly and zero rssLimit as their defaults");
    assert!(!invalid_only.rss_only);
    assert_eq!(invalid_only.rss_limit, None);
    assert!(
        list_pages_feed_info_html(&page_info, &invalid_only)
            .is_some_and(|feed| feed.contains("/limit/1/t/Feed+Title")),
    );
}

#[test]
fn serializes_live_wikidot_list_pages_rss_selector_paths() {
    let page_info = ftml::data::PageInfo {
        page: Cow::Borrowed("_ajax-module-connector"),
        category: None,
        site: Cow::Borrowed("sandbox-for-codex"),
        title: Cow::Borrowed("Preview"),
        alt_title: None,
        score: ftml::data::ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let arguments = parse_list_pages_arguments(concat!(
        r#"pagetype="*" category="* -deleted" tags="+alpha -beta" "#,
        r#"parent="-" created_by="Some_User" rating=">=1" offset="7" "#,
        r#"range="others" order="dateCreatedDesc" limit="5" perPage="3" "#,
        r#"rss="RSS4" rssDescription="D" rssHome="home" rssOnly="yes""#,
    ))
    .expect("the live RSS path selector combination should parse");
    let feed = list_pages_feed_info_html(&page_info, &arguments)
        .expect("an RSS title should generate a feed link");
    assert!(
        !feed.contains("wikidot.com"),
        "feed links must use the accepted current-origin route"
    );
    assert!(feed.contains(concat!(
        r#"href="/feed/pages/"#,
        "pagetype/%2A/category/%2A%2C-deleted/tags/%2Balpha%2C-beta/",
        "parent/-/created_by/some_user/offset/7/rating/%3E%3D1/",
        "range/others/order/dateCreatedDesc/limit/3/t/RSS4/d/D/h/home",
        r#"">RSS feed</a>"#,
    )));

    let all_categories = parse_list_pages_arguments(
        r#"category="*" rss="Modern" rssTitle="Deprecated" rssOnly="yes""#,
    )
    .expect("the all-category RSS link should parse");
    let feed = list_pages_feed_info_html(&page_info, &all_categories)
        .expect("the preferred RSS title should generate a feed link");
    assert!(feed.contains("/feed/pages/t/Modern"));
    assert!(!feed.contains("/category/%2A/"));
    assert!(!feed.contains("Deprecated"));

    let unusual_limits = parse_list_pages_arguments(
        r#"limit="wat" perPage="huh" rss="RSS4" rssOnly="yes""#,
    )
    .expect("RSS preserves live's nonnumeric inherited limit");
    assert!(
        list_pages_feed_info_html(&page_info, &unusual_limits)
            .is_some_and(|feed| feed.contains("/limit/huh/t/RSS4")),
    );
}

#[test]
fn accepts_live_evidenced_wikidot_list_pages_noops() {
    let baseline = parse_list_pages_arguments(
        r#" category="*" tags="fixture" limit="20" wrapper="no" "#,
    )
    .expect("baseline ListPages module should parse");

    for head in [
        r#" category="*" tags="fixture" limit="20" class="g54-custom" wrapper="no" "#,
        r#" category="*" tags="fixture" limit="20" custom="@URL" wrapper="no" "#,
        r#" category="*" tags="fixture" limit="20" style="margin: 0; width: 100%;" wrapper="no" "#,
        r#" category="*" tags="fixture" limit="20" unknown="kept" wrapper="no" "#,
        r#" category="*" tags="fixture" limit="20" class="" style="" wrapper="no" "#,
        r#" category="*" tags="fixture" limit="20" class="first" class="second" style="color: red" style="display: block" wrapper="no" "#,
    ] {
        assert_eq!(
            parse_list_pages_arguments(head),
            Some(baseline.clone()),
            "Wikidot accepts the live-evidenced no-op ListPages grammar: {head}",
        );
    }

    assert_eq!(
        parse_list_pages_arguments(
            r#" category="*" tags="fixture" limit="20" data-custom="value" wrapper="no" "#,
        ),
        Some(baseline),
        "syntactically valid unknown arguments are ignored",
    );
}

#[test]
fn parses_wikidot_list_pages_no_tags_selector_without_widening_countpages() {
    for source in [
        r#" category="_default" tags="-" limit="20" "#,
        r#" category="_default" tag="-" limit="20" "#,
    ] {
        let arguments =
            parse_list_pages_arguments(source).expect("no-tags selector should parse");

        assert!(arguments.untagged);
        assert!(arguments.default_tags.is_empty());
        assert!(arguments.all_tags.is_empty());
        assert!(arguments.no_tags.is_empty());
        assert!(
            count_pages_should_remain_literal(&arguments),
            "the ListPages no-tags evidence must not widen CountPages behavior",
        );
    }
}

#[test]
fn parses_singular_list_pages_tag_argument_with_exclusions() {
    let arguments = parse_list_pages_arguments(
        r#" tag="+scp -tale -goi-format -co-authored" category="-fragment" perPage="250""#,
    )
    .expect("Wikidot singular tag selector with exclusions should parse");

    assert_eq!(arguments.all_tags, vec![Cow::Borrowed("scp")]);
    assert_eq!(
        arguments.no_tags,
        vec![
            Cow::Borrowed("tale"),
            Cow::Borrowed("goi-format"),
            Cow::Borrowed("co-authored")
        ]
    );
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("fragment")]
    );
    assert_eq!(arguments.count_pages_per_page, Some(250));
}

#[test]
fn parses_corpus_list_pages_table_arguments() {
    let arguments = parse_list_pages_arguments(
        r#" tags="1998" separate="no" category="* -deleted" perPage="100" prependLine="||~ ページ ||~ 投稿者 ||~ 投稿日 ||~ 評価 ||""#,
    )
    .expect("corpus ListPages table selector should parse");

    assert!(arguments.category_all);
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("deleted")]
    );
    assert_eq!(arguments.default_tags, vec![Cow::Borrowed("1998")]);
    assert_eq!(arguments.count_pages_per_page, Some(100));
    assert_eq!(
        arguments.prepend_line.as_deref(),
        Some("||~ ページ ||~ 投稿者 ||~ 投稿日 ||~ 評価 ||"),
    );
}

#[test]
fn parses_corpus_list_pages_url_and_filter_arguments() {
    let arguments = parse_list_pages_arguments(
        r#" range="." limit="@URL|0" offset="@URL|5" urlAttrPrefix="list1""#,
    )
    .expect("URL-driven ListPages arguments should parse");

    assert_eq!(arguments.limit, Some(0));
    assert_eq!(arguments.offset, 5);
    assert_eq!(arguments.url_attr_prefix.as_deref(), Some("list1"));

    let arguments = parse_list_pages_arguments(
        r#" separate="no" category="@URL|*" tags="@URL" created_at="@URL" updated_at="@URL" created_by="@URL" rating="@URL" votes="@URL" link_to="@URL" offset="@URL|0" name="@URL" limit="@URL|0" perPage="@URL|20" parent="*" order="@URL|created_at desc" wrapper="no""#,
    )
    .expect("tag-search ListPages arguments should parse");

    assert!(arguments.category_all);
    assert_eq!(arguments.limit, Some(0));
    assert_eq!(arguments.count_pages_per_page, Some(20));
    assert_eq!(arguments.offset, 0);
    assert!(arguments.slug.is_none());
    assert!(!arguments.author_filter_present);
    assert!(!arguments.unsupported_author_filter);

    let unresolved_parent = parse_list_pages_arguments(r#" parent="@URL""#)
        .expect("an unresolved URL parent selector is omitted");
    assert_eq!(
        unresolved_parent.page_parent,
        crate::services::page_query::PageParentSelector::All,
    );
    assert!(list_pages_has_unsupported_parent_selector(
        r#" parent="@URL""#
    ));
    assert!(!list_pages_has_unsupported_parent_selector(
        r#" parent="other-page""#
    ));
    let named_parent = parse_list_pages_arguments(r#" parent="other-page""#)
        .expect("a named parent should be queried after its existence preflight");
    assert_eq!(
        named_parent.static_parent_fullname.as_deref(),
        Some("other-page"),
    );
    assert!(!list_pages_has_unsupported_parent_selector(
        r#" parent="@URL|.""#
    ));
    let large_offset = parse_list_pages_arguments(r#" offset="1001""#)
        .expect("large live offsets should produce an empty result");
    assert_eq!(large_offset.offset_beyond_render_window, Some(1001));
    for value in ["", "-1", "2.5"] {
        let arguments = parse_list_pages_arguments(&format!(r#"offset="{value}""#))
            .expect("invalid live offsets should use the zero default");
        assert_eq!(arguments.offset, 0);
        assert_eq!(arguments.offset_beyond_render_window, None);
    }
    assert!(list_pages_has_unsupported_page_type_selector(
        r#" pagetype="draft""#
    ));
    assert!(!list_pages_has_unsupported_page_type_selector(
        r#" pagetype="@URL|normal""#
    ));
}

#[test]
fn parses_corpus_list_pages_literal_name_as_slug() {
    let arguments =
        parse_list_pages_arguments(r#" category="_default" name="SCP-655-JP""#)
            .expect("literal ListPages name selector should parse");

    assert_eq!(arguments.slug.as_deref(), Some("scp-655-jp"));
    assert!(!arguments.current_page_only);
}

#[test]
fn parses_site_news_name_date_and_rating_selectors() {
    let arguments = parse_list_pages_arguments(
        r#" name="scp-*" created_at="2026.06" rating=">=-4" perPage="250""#,
    )
    .expect("site-news selectors should parse");

    assert!(arguments.slug.is_none());
    assert_eq!(arguments.name_pattern.as_deref(), Some("scp-*"));
    assert!(matches!(
        arguments.creation_date,
        DateSelector::Span {
            resolution: DateTimeResolution::Month,
            comparison: ComparisonOperation::Equal,
            ..
        }
    ));
    assert_eq!(arguments.score.len(), 1);
    assert_eq!(
        arguments.score[0].comparison,
        ComparisonOperation::GreaterOrEqualThan,
    );
    assert_eq!(
        arguments.score[0].score,
        ftml::data::ScoreValue::Integer(-4),
    );
    assert!(!arguments.unsupported_count_pages_filter);
}

#[test]
fn parses_relative_and_comparison_date_selectors() {
    assert!(matches!(
        parse_list_pages_date_selector("older than 2 month"),
        Some(DateSelector::Span {
            resolution: DateTimeResolution::Second,
            comparison: ComparisonOperation::LessThan,
            ..
        })
    ));
    assert!(matches!(
        parse_list_pages_date_selector(">2022.08"),
        Some(DateSelector::Span {
            resolution: DateTimeResolution::Month,
            comparison: ComparisonOperation::GreaterThan,
            ..
        })
    ));
    assert!(matches!(
        parse_list_pages_date_selector("last 180 days"),
        Some(DateSelector::FromPresent { .. })
    ));
    assert_eq!(
        parse_list_pages_date_selector("last 9223372036854775807 days"),
        None,
    );

    // A bare keyword or an incomplete phrase is not a selector.
    for value in ["older", "newer", "last", "older than", "newer than"] {
        assert_eq!(
            parse_list_pages_date_selector(value),
            None,
            "{value:?} is not a complete relative selector",
        );
    }

    // The relative unit defaults to one day and the direction is preserved.
    for (value, comparison) in [
        ("older than 5", ComparisonOperation::LessThan),
        ("newer than 5", ComparisonOperation::GreaterThan),
        ("older than 2 month", ComparisonOperation::LessThan),
        ("newer than 2 days", ComparisonOperation::GreaterThan),
    ] {
        assert!(
            matches!(
                parse_list_pages_date_selector(value),
                Some(DateSelector::Span {
                    resolution: DateTimeResolution::Second,
                    comparison: actual,
                    ..
                }) if actual == comparison
            ),
            "{value:?} should parse with {comparison:?}",
        );
    }
    assert!(matches!(
        parse_list_pages_date_selector("last 5 days"),
        Some(DateSelector::FromPresent { .. })
    ));
}

#[test]
fn batches_only_simple_unbounded_required_tag_counts() {
    let arguments = parse_list_pages_arguments(
        r#" tags="third-law -hub -artwork -artist" wrapper="no""#,
    )
    .expect("activity-marker CountPages selectors should parse");
    assert_eq!(
        count_pages_required_tag_batch_selector(&arguments),
        Some("third-law"),
    );

    let bounded = parse_list_pages_arguments(r#" tags="+third-law" limit="1""#)
        .expect("bounded selector should parse");
    assert_eq!(count_pages_required_tag_batch_selector(&bounded), None);
}

#[test]
fn required_tag_batches_preserve_the_raw_scan_cap_and_uncertain_permissions() {
    let last_exact = i64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS) - 1;
    assert_eq!(
        count_pages_required_tag_batch_result(last_exact, Some(true)),
        CountPagesRequiredTagBatchResult::Exact(last_exact as usize),
    );
    assert_eq!(
        count_pages_required_tag_batch_result(last_exact, Some(false)),
        CountPagesRequiredTagBatchResult::Exact(0),
    );
    for raw_total in [
        i64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
        i64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS) + 1,
        i64::MAX,
    ] {
        assert_eq!(
            count_pages_required_tag_batch_result(raw_total, Some(true)),
            CountPagesRequiredTagBatchResult::PreserveLiteral,
        );
        assert_eq!(
            count_pages_required_tag_batch_result(raw_total, Some(false)),
            CountPagesRequiredTagBatchResult::Exact(0),
        );
    }
    assert_eq!(
        count_pages_required_tag_batch_result(1, None),
        CountPagesRequiredTagBatchResult::PreserveLiteral,
    );
    assert_eq!(
        count_pages_required_tag_batch_result(-1, Some(true)),
        CountPagesRequiredTagBatchResult::PreserveLiteral,
    );
}

#[test]
fn dense_count_pages_literal_checks_advance_each_region_once() {
    const MODULE_COUNT: usize = 4_096;
    const LITERAL_MODULE: &str =
        "[!-- [[module CountPages tags=\"+literal\"]]L[[/module]] --]\n";
    const ACTIVE_MODULE: &str = "[[module CountPages tags=\"+active\"]]A[[/module]]\n";
    let mut source = String::with_capacity(
        MODULE_COUNT * (LITERAL_MODULE.len() + ACTIVE_MODULE.len()),
    );
    for _ in 0..MODULE_COUNT {
        source.push_str(LITERAL_MODULE);
        source.push_str(ACTIVE_MODULE);
    }

    let literal_regions = LiteralRegionIndex::new_count_pages_syntax(&source);
    let mut literal_regions = literal_regions.monotone_cursor();
    let mut literal_count = 0;
    let mut active_count = 0;
    for captures in COUNTPAGES_MODULE_REGEX.captures_iter(&source) {
        let module = captures
            .get(0)
            .expect("CountPages capture has a full match");
        if advance_literal_cursor_and_check_count_pages_capture_containment(
            &mut literal_regions,
            module.start(),
        ) {
            literal_count += 1;
        } else {
            active_count += 1;
        }
    }

    assert_eq!(literal_count, MODULE_COUNT);
    assert_eq!(active_count, MODULE_COUNT);
    assert_eq!(literal_regions.advances(), MODULE_COUNT);
}

#[test]
fn dense_count_pages_projection_checks_are_linear_per_pass() {
    const MODULE_COUNT: usize = 4_096;
    const MODULE: &str = "[[module CountPages\ttags=\"+active\"]]A[[/module]]\n";
    let source = MODULE.repeat(MODULE_COUNT);
    let projection = ListPagesSourceProjection::new(&source)
        .expect("tab expansion should create a source projection");

    for pass in 0..2 {
        let mut projection_ranges = projection.original_range_cursor();
        let mut checked = 0usize;
        let mut checked_head_bytes = 0usize;
        for captures in COUNTPAGES_MODULE_REGEX.captures_iter(&source) {
            let head = captures
                .name("head")
                .expect("CountPages capture has a head");
            checked += 1;
            checked_head_bytes += head.len();
            assert!(
                !projection_ranges.advance_to_range_and_check_unchanged(
                    &source,
                    head.start()..head.end()
                ),
                "tab-expanded CountPages head must be projection-changed on pass {pass}",
            );
        }

        assert_eq!(checked, MODULE_COUNT);
        assert!(projection_ranges.advances() <= projection.source().len());
        assert!(
            projection_ranges.advances() + checked_head_bytes
                <= projection.source().len() + source.len(),
            "projection cursor and head comparisons must remain linear on pass {pass}",
        );
    }
}

#[test]
fn parses_static_data_form_selectors() {
    let arguments = parse_list_pages_arguments(
        r#" category="codexdftdft01" _codexkind="alpha" _codexflag!="missing" _Status="open" __private="yes" order="titleAsc" limit="20" separate="false""#,
    )
    .expect("static data-form selectors should parse");

    assert_eq!(
        arguments.data_form_fields,
        vec![
            DataFormSelector {
                field: Cow::Borrowed("codexkind"),
                value: Cow::Borrowed("alpha"),
                negated: false,
            },
            DataFormSelector {
                field: Cow::Borrowed("codexflag"),
                value: Cow::Borrowed("missing"),
                negated: true,
            },
            DataFormSelector {
                field: Cow::Borrowed("Status"),
                value: Cow::Borrowed("open"),
                negated: false,
            },
            DataFormSelector {
                field: Cow::Borrowed("_private"),
                value: Cow::Borrowed("yes"),
                negated: false,
            },
        ],
    );
    let unresolved = parse_list_pages_arguments(r#" _status="@URL""#)
        .expect("an unresolved URL data-form selector should drop only that field");
    assert!(unresolved.data_form_fields.is_empty());
    assert!(unresolved.unsupported_count_pages_filter);
}

#[test]
fn parses_corpus_list_pages_created_by_argument() {
    let source = r#"[[module ListPages created_by="[Congy]" separate="no"]]%%title_linked%%[[/module]]"#;
    let modules = find_list_pages_module_matches(source);
    let module = modules
        .first()
        .expect("ListPages module with bracketed quoted author should match");
    assert_eq!(module.head.trim(), r#"created_by="[Congy]" separate="no""#,);

    let arguments = parse_list_pages_arguments(
        r#" created_by="[Congy]" separate="no" tags="+jp" order="created" category="-deleted""#,
    )
    .expect("bracketed Wikidot author selector should parse");

    assert_eq!(arguments.authors, vec![Cow::Borrowed("Congy")]);
    assert!(arguments.author_filter_present);
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("deleted")]
    );
    assert_eq!(arguments.all_tags, vec![Cow::Borrowed("jp")]);

    let not_current = parse_list_pages_arguments(r#" created_by="-=" limit="20""#)
        .expect("not-current author selector should remain identifiable");
    assert!(not_current.authors.is_empty());
    assert!(not_current.author_filter_present);
    assert!(
        not_current.exclude_current_page_author,
        "the sentinel excludes the containing page's author rather than blocking",
    );
    assert!(!not_current.unsupported_author_filter);
    assert!(not_current.unsupported_count_pages_filter);
}

#[test]
fn list_pages_author_cache_key_normalizes_order_case_and_duplicates() {
    let repeated = list_pages_author_cache_key(
        &[
            Cow::Borrowed("Billith"),
            Cow::Borrowed("billith"),
            Cow::Borrowed("BILLITH"),
        ],
        true,
    );
    let single = list_pages_author_cache_key(&[Cow::Borrowed("billith")], true);
    assert_eq!(repeated, single);
    let mut cache = BTreeMap::new();
    cache.insert(repeated, super::super::ResolvedListPagesAuthors::None);
    assert!(matches!(
        cache.get(&single),
        Some(super::super::ResolvedListPagesAuthors::None)
    ));

    let current = list_pages_author_cache_key(&[Cow::Borrowed("=")], true);
    assert_ne!(single, current);
    assert_ne!(single, list_pages_author_cache_key(&[], false));
}

#[test]
fn ignores_blank_wikidot_list_pages_order_argument() {
    let arguments = parse_list_pages_arguments(
        r#" created_by="=" order="" category="-fragment" tag="+scp -co-authored" perPage="250""#,
    )
    .expect("blank ListPages order should fall back to default order");

    assert_eq!(arguments.authors, vec![Cow::Borrowed("=")]);
    assert_eq!(arguments.order, None);
    assert_eq!(arguments.count_pages_per_page, Some(250));
    assert_eq!(arguments.all_tags, vec![Cow::Borrowed("scp")]);
    assert_eq!(arguments.no_tags, vec![Cow::Borrowed("co-authored")]);
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("fragment")]
    );
}

#[test]
fn parses_corpus_list_pages_custom_and_unknown_noops() {
    let arguments = parse_list_pages_arguments(
        r#" created_by="=" tag="+scp" order="" category="-fragment" tag="-co-authored" perPage="250" custom="@URL" unknown="kept""#,
    )
    .expect("live-evidenced custom and unknown ListPages no-ops should parse");

    assert_eq!(arguments.authors, vec![Cow::Borrowed("=")]);
    assert_eq!(arguments.order, None);
    assert_eq!(arguments.count_pages_per_page, Some(250));
    assert!(arguments.all_tags.is_empty());
    assert_eq!(arguments.no_tags, vec![Cow::Borrowed("co-authored")]);
    assert_eq!(
        arguments.excluded_categories,
        vec![Cow::Borrowed("fragment")]
    );
    let typo = parse_list_pages_arguments(r#" tag="+scp" selector_typo="kept""#)
        .expect("unknown arguments are inert");
    assert_eq!(typo.all_tags, vec![Cow::Borrowed("scp")]);
}

#[test]
fn parses_wikidot_camel_case_list_pages_order_argument() {
    let ascending = parse_list_pages_arguments(
        r#" category="*" tags="codex" order="titleAsc" limit="20" wrapper="no""#,
    )
    .expect("Wikidot camel-case order should parse");
    assert_eq!(
        ascending.order,
        Some(OrderBySelector {
            property: OrderProperty::Title,
            ascending: true,
        }),
    );

    let descending = parse_list_pages_arguments(
        r#" category="*" tags="codex" order="createdAtDesc" limit="20""#,
    )
    .expect("Wikidot camel-case descending order should parse");
    assert_eq!(
        descending.order,
        Some(OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: false,
        }),
    );

    for (value, property, ascending) in [
        ("dateCreatedAsc", OrderProperty::CreatedAt, true),
        ("dateEditedDesc", OrderProperty::UpdatedAt, false),
        ("pageLengthDesc", OrderProperty::Size, false),
        ("created_by", OrderProperty::CreatedBy, true),
        ("votes desc", OrderProperty::Votes, false),
        ("revisions desc", OrderProperty::Revisions, false),
        ("comments desc", OrderProperty::Comments, false),
        ("created_at desc desc", OrderProperty::CreatedAt, true),
        // The live preview accepts the ordinary `asc` spelling as ascending;
        // `desc desc` is the legacy alternate spelling for that direction.
        ("created_at asc", OrderProperty::CreatedAt, true),
        ("unknown", OrderProperty::CreatedAt, false),
    ] {
        let arguments = parse_list_pages_arguments(&format!(r#"order="{value}""#))
            .expect("live-supported order should parse");
        assert_eq!(
            arguments.order,
            Some(OrderBySelector {
                property,
                ascending,
            }),
            "{value:?}",
        );
    }

    let data_form_order = parse_list_pages_arguments(r#"order="_mainword""#)
        .expect("data-form ordering should remain representable");
    assert_eq!(
        data_form_order.order,
        Some(OrderBySelector {
            property: OrderProperty::DataFormFieldName {
                field: Cow::Borrowed("mainword"),
                numeric: false,
            },
            ascending: true,
        }),
        "ordinary data-form ordering should retain its field name",
    );

    let integer_data_form_order =
        parse_list_pages_arguments(r#"order="_albums::integer desc""#)
            .expect("integer data-form ordering should parse");
    assert_eq!(
        integer_data_form_order.order,
        Some(OrderBySelector {
            property: OrderProperty::DataFormFieldName {
                field: Cow::Borrowed("albums"),
                numeric: true,
            },
            ascending: false,
        }),
    );
}

#[test]
fn count_pages_default_selector_executes_but_all_categories_remains_literal() {
    let no_filter = parse_list_pages_arguments(r#""#)
        .expect("broad CountPages selector should parse");
    assert!(!count_pages_should_remain_literal(&no_filter));

    let all_categories = parse_list_pages_arguments(r#" category="*" "#)
        .expect("all-category CountPages selector should parse");
    assert!(count_pages_should_remain_literal(&all_categories));

    let exclusion_only = parse_list_pages_arguments(r#" category="* -deleted" "#)
        .expect("exclusion-only CountPages selector should parse");
    assert!(count_pages_should_remain_literal(&exclusion_only));
}

#[test]
fn allows_unbounded_count_pages_with_static_filter() {
    let tagged = parse_list_pages_arguments(r#" category="*" tags="codex" "#)
        .expect("static tag CountPages selector should parse");
    assert!(!count_pages_should_remain_literal(&tagged));

    let named = parse_list_pages_arguments(r#" name="example" "#)
        .expect("static name CountPages selector should parse");
    assert!(!count_pages_should_remain_literal(&named));
}

#[test]
fn count_pages_substitution_resolves_numeric_ifexpr() {
    let output = substitute_count_pages_variables(
        r#"[[div class="activity-container [[#ifexpr %%total%% >= 60 | large-c | not-large-c ]]" data-number="%%total%%"]]x[[/div]]"#,
        0,
    );

    assert!(output.contains(r#"activity-container not-large-c"#));
    assert!(output.contains(r#"data-number="0""#));
    assert!(!output.contains("[[#ifexpr"));
}

#[test]
fn count_pages_substitution_cannot_construct_a_trusted_compat_marker() {
    let output = substitute_count_pages_variables(
        concat!(
            "<div id=\"ml-1\" data-wikijump-compat-members=\"%%total%%\">",
            "<img src=x onerror=\"alert(1)\"></div>",
        ),
        1,
    );
    assert!(output.contains("data-wikijump-authored-compat-members=\"1\""));

    let mut protected = output.clone();
    let fragments = RenderService::protect_generated_wikidot_compat_html(
        &mut protected,
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );
    assert!(fragments.is_empty());
    let rendered = render_wikidot_page_body_after_compat_restore(&output);
    assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
}

#[test]
fn count_pages_exact_count_render_diagnostics_allow_matching_explicit_sql_window() {
    let diagnostics = count_pages_exact_count_render_diagnostics(
        PageQueryResultMetadata {
            candidate_count: Some(10),
            sql_limit_offset_applied: true,
            exact_count_safe: true,
            ..PageQueryResultMetadata::default()
        },
        false,
        false,
        false,
        Some(10),
        10,
    );

    assert!(diagnostics.allowed);
    assert_eq!(diagnostics.denied_reason_code, None);
    assert_eq!(diagnostics.denied_reason_detail, None);
}

#[test]
fn count_pages_exact_count_render_diagnostics_denies_unbounded_sql_window() {
    let diagnostics = count_pages_exact_count_render_diagnostics(
        PageQueryResultMetadata {
            candidate_count: Some(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize),
            sql_limit_offset_applied: true,
            exact_count_safe: true,
            ..PageQueryResultMetadata::default()
        },
        false,
        false,
        false,
        None,
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
    );

    assert!(!diagnostics.allowed);
    assert_eq!(diagnostics.denied_reason_code, Some("unsafe_sql_window"));
    assert_eq!(diagnostics.denied_reason_detail, None);
}

#[test]
fn count_pages_exact_count_render_diagnostics_denies_post_query_exclusion_before_offset()
{
    let diagnostics = count_pages_exact_count_render_diagnostics(
        PageQueryResultMetadata {
            candidate_count: Some(10),
            exact_count_safe: true,
            ..PageQueryResultMetadata::default()
        },
        false,
        true,
        true,
        Some(10),
        11,
    );

    assert!(!diagnostics.allowed);
    assert_eq!(diagnostics.denied_reason_code, Some("post_query_exclusion"));
    assert_eq!(diagnostics.denied_reason_detail, None);
}

#[test]
fn unbounded_count_pages_remains_literal_when_scan_cap_is_reached() {
    assert_eq!(
        count_pages_unbounded_total(CountPagesRawScanCompletion::Capped, 1_000),
        None,
    );
    assert_eq!(
        count_pages_unbounded_total(CountPagesRawScanCompletion::Complete, 17),
        Some(17),
    );
    assert_eq!(
        count_pages_unbounded_total(CountPagesRawScanCompletion::RandomSampleCapped, 17,),
        None,
        "a capped random CountPages scan must stay literal instead of exposing a sampled hidden-page ratio",
    );
}

#[test]
fn data_form_candidate_cap_requires_original_listpages_and_countpages_modules() {
    assert!(page_query_cap_requires_original_module(
        &PageQueryResultMetadata {
            candidate_count: Some(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize),
            cap_exceeded: true,
            filtering_deferred_to_rust: true,
            ..PageQueryResultMetadata::default()
        },
    ));
    assert!(!page_query_cap_requires_original_module(
        &PageQueryResultMetadata::default(),
    ));
}

#[test]
fn capped_random_scan_remains_literal_for_privacy() {
    assert_eq!(
        count_pages_raw_scan_completion(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize - 1,),
        CountPagesRawScanCompletion::Complete,
    );
    let raw_scan_completion =
        count_pages_raw_scan_completion(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize);

    assert_eq!(
        raw_scan_completion,
        CountPagesRawScanCompletion::RandomSampleCapped,
    );
    assert!(count_pages_scan_requires_preservation(
        raw_scan_completion,
        99,
        100,
    ));
    assert!(
        count_pages_scan_requires_preservation(raw_scan_completion, 100, 100,),
        "a capped random scan remains literal even when the visible sample fills the requested count"
    );
    assert!(!count_pages_scan_requires_preservation(
        CountPagesRawScanCompletion::Complete,
        99,
        100,
    ));
}

#[test]
fn list_pages_scan_target_covers_requested_page_within_the_safety_ceiling() {
    assert_eq!(list_pages_row_scan_target(Some(100), 0, false), 100);
    assert_eq!(list_pages_row_scan_target(Some(100), 25, true), 126);
    assert_eq!(
        list_pages_row_scan_target(None, 0, false),
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
        "an unbounded paginated ListPages module needs the bounded safety window to determine pager totals",
    );
    assert_eq!(
        list_pages_row_scan_target(Some(1_000), 0, false),
        1_000,
        "an explicit overall limit must remain available when computing the pager total",
    );
    assert_eq!(
        list_pages_row_scan_target(
            Some(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS).saturating_add(1)),
            0,
            false,
        ),
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
        "an explicit overall limit must still obey the render safety ceiling",
    );
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn list_pages_scan_envelope_covers_the_synchronized_live_inventory() {
    const SYNCHRONIZED_SANDBOX_PAGE_COUNT: u32 = 6_566;

    assert!(
        MAX_LISTPAGES_RENDER_SCAN_ROWS > SYNCHRONIZED_SANDBOX_PAGE_COUNT,
        "the bounded scan must classify the complete live inventory instead of truncating it",
    );
}

#[test]
fn list_pages_content_budget_limits_modules_and_rows() {
    let mut budget = ListPagesExpansionBudget::new();

    assert!(budget.try_start_content_module());
    assert!(budget.try_start_content_module());
    assert!(budget.try_start_content_module());
    assert!(!budget.try_start_content_module());
    assert!(budget.can_expand_content_rows(40));
    budget.consume_content_rows(40);
    assert!(budget.can_expand_content_rows(MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER - 40));
    assert!(!budget.can_expand_content_rows(MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER - 39));

    budget.consume_content_rows(MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER - 40);
    assert!(budget.can_expand_content_rows(0));
    assert!(!budget.can_expand_content_rows(1));
}

#[test]
fn list_pages_content_query_target_probes_only_enough_rows_to_decide_the_budget() {
    assert_eq!(
        list_pages_content_query_target(5_000, 250, 100, 0, false, false),
        101
    );
    assert_eq!(
        list_pages_content_query_target(5_000, 100, 100, 0, false, false),
        100
    );
    assert_eq!(
        list_pages_content_query_target(5_000, 250, 82, 0, false, false),
        83
    );
    assert_eq!(
        list_pages_content_query_target(5_000, 50, 100, 10, true, false),
        61
    );
    assert_eq!(
        list_pages_content_query_target(40, 250, 100, 10, true, false),
        40
    );
    assert_eq!(
        list_pages_content_query_target(5_000, 50, 100, 10, true, true),
        5_000
    );
}

#[test]
fn render_page_query_batches_match_the_remaining_scan_window() {
    assert_eq!(render_page_query_batch_limit(100, 0, 0), 250);
    assert_eq!(render_page_query_batch_limit(5_000, 0, 0), 5_000);
    assert_eq!(render_page_query_batch_limit(300, 250, 250), 250);
    assert_eq!(render_page_query_batch_limit(5_000, 4_000, 4_000), 1_000);
}

#[test]
fn random_page_query_rendering_uses_one_capped_scan() {
    assert!(render_page_query_uses_single_scan(Some(OrderBySelector {
        property: OrderProperty::Random,
        ascending: false,
    })));
    assert!(render_page_query_uses_single_scan(Some(OrderBySelector {
        property: OrderProperty::SeededRandom(Cow::Borrowed("seed")),
        ascending: false,
    })));
    assert!(!render_page_query_uses_single_scan(Some(OrderBySelector {
        property: OrderProperty::CreatedAt,
        ascending: false,
    })));
    assert!(!render_page_query_uses_single_scan(None));
}

#[test]
fn module_opening_preflight_is_ascii_case_insensitive_and_syntax_aware() {
    assert!(has_list_pages_module_opening_candidate(
        "[[MoDuLe ListPages]]",
    ));
    assert!(has_list_pages_module_opening_candidate(
        "[[ module654_\nLISTPAGES limit=1]]",
    ));
    assert!(has_count_pages_module_opening_candidate(
        "[[module COUNTPAGES]]",
    ));
    assert!(!has_list_pages_module_opening_candidate(
        "ListPages documentation without a module opening",
    ));
    assert!(!has_list_pages_module_opening_candidate(
        "[[module CSS]] /* ListPages */",
    ));
    assert!(!has_count_pages_module_opening_candidate(
        "[[module654 CountPages]]",
    ));

    let source = "prefix [[MoDuLe ListPages]]first[[/module]] [[module ListPages]]second[[/module]]";
    assert_eq!(
        first_list_pages_module_opening_candidate(source),
        source.find("[[MoDuLe ListPages]]")
    );
}

#[test]
fn random_page_query_scan_uses_the_full_render_cap() {
    assert_eq!(
        random_page_query_scan_limit(1),
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
    );
    assert_eq!(
        random_page_query_scan_limit(100),
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
    );
    assert_eq!(
        random_page_query_scan_limit(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize + 1),
        u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
    );
}

#[test]
fn repeated_rating_selectors_keep_only_the_effective_final_value() {
    let impossible_alias = r#" score=">=0""#.repeat(MAX_PAGE_QUERY_SCORE_SELECTORS);
    let arguments = parse_list_pages_arguments(&impossible_alias)
        .expect("the impossible score alias should remain inert");
    assert!(arguments.score.is_empty());
    assert!(!arguments.unsupported_score_filter);

    let repeated_rating = r#" rating=">=0""#.repeat(MAX_PAGE_QUERY_SCORE_SELECTORS + 1);
    let arguments = parse_list_pages_arguments(&repeated_rating)
        .expect("duplicate canonical rating selectors should remain representable");
    assert_eq!(arguments.score.len(), 1);
    assert!(arguments.unsupported_score_filter);
}

#[test]
fn does_not_protect_forgeable_pager_html_before_parsing() {
    let original = r#"<div class="pager" data-wikijump-compat-pager="1"><img src=x onerror="alert(1)"></div>"#;
    let mut wikitext = original.to_owned();
    let fragments = RenderService::protect_generated_wikidot_compat_html(
        &mut wikitext,
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(fragments.is_empty());
    assert_eq!(wikitext, original);
}

#[test]
fn forged_pager_html_is_not_restored_as_trusted_html_after_render() {
    let rendered = render_wikidot_page_body_after_compat_restore(
        r#"<div class="pager" data-wikijump-compat-pager="1"><img src=x onerror="alert(1)"></div>"#,
    );

    assert!(rendered.contains("&lt;div"));
    assert!(rendered.contains("&lt;img"));
    assert!(rendered.contains("onerror=&quot;alert(1)&quot;"));
    assert!(!rendered.contains(r#"<div class="pager""#));
    assert!(!rendered.contains("<img"));
    assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
    assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
}

#[test]
fn list_pages_compat_registry_ignores_code_block_fragments() {
    let source = concat!(
        "[[code]]\n",
        "<table class=\"wiki-content-table\" data-wikijump-compat-listpages=\"1\">",
        "<tr><td><img src=x onerror=\"alert(document.domain)\"></td></tr>",
        "</table>\n",
        "<span class=\"odate time_1 format_%25e\" ",
        "data-wikijump-compat-date=\"1\">1 Jan 1970</span>\n",
        "[[/code]]",
    );
    let mut fragments = CompatHtmlFragments::new(source);

    let protected = register_generated_list_pages_html(source.to_owned(), &mut fragments);

    assert_eq!(protected, source);
    assert_eq!(fragments.restore(&protected), protected);
    let rendered = render_wikidot_page_body_after_compat_restore(&protected);
    assert!(!rendered.contains(r#"<img src=x onerror="alert(document.domain)">"#));
    assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
}

#[test]
fn expanded_list_pages_wrapper_preserves_the_following_paragraph_boundary() {
    let mut replacement = "[[div class=\"list-pages-box\"]]\nrow\n[[/div]]".to_owned();

    repair_list_pages_block_boundaries(&mut replacement, ("", "\nafter"));

    let rendered =
        render_wikidot_page_body_after_compat_restore(&format!("{replacement}\nafter"));
    assert_eq!(
        rendered,
        "<div class=\"list-pages-box\"><p>row</p></div><p>after</p>",
    );

    let mut ordinary_div = "[[div]]\nrow\n[[/div]]".to_owned();
    repair_list_pages_block_boundaries(&mut ordinary_div, ("", "\nafter"));
    assert_eq!(ordinary_div, "[[div]]\nrow\n[[/div]]");

    let mut existing_blank_line =
        "[[div class=\"list-pages-box\"]]\nrow\n[[/div]]".to_owned();
    repair_list_pages_block_boundaries(&mut existing_blank_line, ("", "\n\nafter"));
    assert_eq!(
        existing_blank_line,
        "[[div class=\"list-pages-box\"]]\nrow\n[[/div]]",
    );
}

#[test]
fn neutralizes_compat_markers_composed_by_list_pages_substitution() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some(
            r#"compat-members="1"><img src=x onerror="alert(1)"></div>"#.to_owned(),
        ),
        alt_title: None,
        slug: Some("forged-title".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let mut substituted = substitute_list_pages_variables(
        r#"<div id="ml-1" data-wikijump-%%title%%"#,
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &BTreeMap::new(), None, &BTreeMap::new()),
    );
    assert!(substituted.contains(r#"data-wikijump-compat-members="1""#));

    RenderService::neutralize_authored_wikidot_compat_markers(&mut substituted);
    let fragments = RenderService::protect_generated_wikidot_compat_html(
        &mut substituted,
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(fragments.is_empty());
    assert!(substituted.contains("data-wikijump-authored-compat-members"));
}

#[test]
fn generated_list_pages_date_is_registered_before_authored_marker_neutralization() {
    let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
        .expect("fixture timestamp should be valid");
    let generated =
        format_list_pages_created_at(Some(created_at), Some("%d %b %Y"), false, true);
    let mut fragments = CompatHtmlFragments::new("");
    let mut protected = register_generated_list_pages_html(generated, &mut fragments);

    RenderService::neutralize_authored_wikidot_compat_markers(&mut protected);
    assert!(!protected.contains("data-wikijump-compat-date"));
    assert!(!protected.contains("data-wikijump-authored-compat-date"));
    let restored = fragments.restore(&protected);
    assert!(restored.contains(r#"<span class="odate time_1782003564"#));
    assert!(!restored.contains("data-wikijump-compat-date"));
}

#[test]
fn generated_list_pages_user_is_registered_without_internal_marker() {
    let generated = r#"<span class="printuser avatarhover" data-wikijump-compat-listpages-user="1"><a href="http://www.wikidot.com/user:info/example">example</a></span>"#;
    let mut fragments = CompatHtmlFragments::new("");
    let protected =
        register_generated_list_pages_html(generated.to_owned(), &mut fragments);
    let restored = fragments.restore(&protected);

    assert!(restored.contains(r#"<span class="printuser avatarhover">"#));
    assert!(restored.contains("user:info/example"));
    assert!(!restored.contains("data-wikijump-compat-listpages-user"));
}

#[test]
fn generated_list_pages_feed_is_restored_as_block_html() {
    let generated = concat!(
        "\n\n",
        r#"<div class="feedinfo" data-wikijump-compat-listpages-feed="1">"#,
        r#"<a href="/feed/pages/t/Test">RSS feed</a></div>"#,
        "\n\n",
    );
    let mut fragments = CompatHtmlFragments::new("");
    let mut protected =
        register_generated_list_pages_html(generated.to_owned(), &mut fragments);
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    ftml::preprocess_for_layout(&mut protected, settings.layout);
    let tokens = ftml::tokenize(&protected);
    let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    let rendered = fragments.restore(&rendered);

    assert_eq!(
        rendered,
        r#"<div class="feedinfo"><a href="/feed/pages/t/Test">RSS feed</a></div>"#,
    );
}

#[test]
fn authored_list_pages_feed_is_not_registered_as_trusted_html() {
    let mut forged = r#"<div class="feedinfo" data-wikijump-compat-listpages-feed="1"><img src=x onerror="alert(1)"></div>"#.to_owned();
    RenderService::neutralize_authored_wikidot_compat_markers(&mut forged);
    assert!(forged.contains("data-wikijump-authored-compat-listpages-feed"));

    let mut fragments = CompatHtmlFragments::new("");
    let protected = register_generated_list_pages_html(forged, &mut fragments);

    assert!(!protected.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    assert_eq!(fragments.restore(&protected), protected);
}

#[test]
fn defers_wikidot_list_pages_custom_date_format_to_odate_class() {
    let created_at = time::Date::from_calendar_date(2024, time::Month::August, 8)
        .expect("fixture date should be valid")
        .with_hms(19, 44, 0)
        .expect("fixture time should be valid")
        .assume_utc();

    let rendered = format_list_pages_created_at(
        Some(created_at),
        Some("%Y-%m-%d %R|agohover"),
        true,
        true,
    );

    assert!(rendered.contains("format_%25Y-%25m-%25d%20%25R%7Cagohover"));
    // ListPages' anonymous PagePreview response carries the UTC server text;
    // the requested format is carried by the ODate class.
    assert!(rendered.ends_with(">08 Aug 2024 19:44</span>"));
}

#[test]
fn list_pages_custom_date_format_collapses_repeated_ascii_spaces() {
    let created_at = time::Date::from_calendar_date(2024, time::Month::August, 8)
        .expect("fixture date should be valid")
        .with_hms(19, 44, 0)
        .expect("fixture time should be valid")
        .assume_utc();

    let rendered = format_list_pages_created_at(
        Some(created_at),
        Some("%Y年 %m月%d日  %H:%M"),
        true,
        true,
    );

    assert!(
        rendered
            .contains("format_%25Y%E5%B9%B4%20%25m%E6%9C%88%25d%E6%97%A5%20%25H%3A%25M",)
    );
    assert!(!rendered.contains("%20%20"));
}

#[test]
fn generated_list_pages_pager_still_renders_without_forgeable_marker() {
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments::default(),
        None,
        0,
        2,
        5,
    );

    assert!(wikitext.starts_with(r#"<div class="pager">"#));
    assert!(!wikitext.contains("data-wikijump-compat-pager"));

    let rendered = render_trusted_wikidot_block_html(wikitext);

    assert!(
        rendered
            .contains(r#"<div class="pager"><span class="pager-no">page 1 of 3</span>"#,),
        "generated pager should preserve the expected block structure",
    );
    assert!(
        !rendered.contains(r#"<div class="pager"><p>"#),
        "generated pager should not gain an extra paragraph wrapper"
    );
    assert!(rendered.contains(r#"<span class="pager-no">page 1 of 3</span>"#));
    assert!(
        rendered.contains(r#"<a href="/scp-7243/p/2">2</a>"#),
        "generated pager should preserve the expected saved-page route"
    );
    assert!(!rendered.contains("data-wikijump-compat-pager"));
}

#[test]
fn generated_list_pages_pager_keeps_untrusted_slug_inside_href() {
    let page_info = fallback_test_page_info(
        "日本語/already%2Fencoded] [[span class=\"owned\"]]OWNED[[/span]] [",
        "Missing page",
    );
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments::default(),
        None,
        0,
        2,
        5,
    );

    let encoded_slug = concat!(
        "%E6%97%A5%E6%9C%AC%E8%AA%9E%2Falready%252Fencoded%5D%20",
        "%5B%5Bspan%20class%3D%22owned%22%5D%5DOWNED",
        "%5B%5B%2Fspan%5D%5D%20%5B",
    );
    assert!(wikitext.contains(&format!("/{encoded_slug}/p/2")));
    assert!(!wikitext.contains(r#"[[span class="owned"]]"#));

    let rendered = render_trusted_wikidot_block_html(wikitext);

    assert!(
        rendered.contains(&format!(r#"<a href="/{encoded_slug}/p/2">2</a>"#)),
        "generated pager should keep the encoded slug inside the href"
    );
    assert_eq!(rendered.matches(r#"class="owned""#).count(), 0);
    assert_eq!(rendered.matches("<a href=").count(), 3);
}

#[test]
fn generated_list_pages_pager_preserves_live_url_argument_shape() {
    let page_info = fallback_test_page_info("listpages-url-shape", "ListPages URL shape");
    let path_arguments = vec![
        UrlArgumentPair {
            name: "tag".to_owned(),
            value: Some("alpha".to_owned()),
        },
        UrlArgumentPair {
            name: "p".to_owned(),
            value: Some("2".to_owned()),
        },
        UrlArgumentPair {
            name: "p".to_owned(),
            value: Some("3".to_owned()),
        },
    ];
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        },
        None,
        4,
        2,
        10,
    );

    assert!(
        wikitext.contains("/listpages-url-shape/tag/alpha/p/1/p/3"),
        "{wikitext}",
    );
    assert!(
        wikitext.contains("/listpages-url-shape/tag/alpha/p/4/p/3"),
        "{wikitext}",
    );
}

#[test]
fn generated_list_pages_pager_appends_after_malformed_page_arguments() {
    let page_info =
        fallback_test_page_info("listpages-malformed-pager", "ListPages pager");
    let path_arguments = vec![UrlArgumentPair {
        name: "p".to_owned(),
        value: Some("nope".to_owned()),
    }];
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        },
        None,
        0,
        2,
        5,
    );

    assert!(
        wikitext.contains("/listpages-malformed-pager/p/nope/p/2"),
        "{wikitext}",
    );
}

#[test]
fn generated_list_pages_pager_replaces_zero_page_arguments() {
    let page_info = fallback_test_page_info("listpages-zero-pager", "ListPages pager");
    let path_arguments = vec![UrlArgumentPair {
        name: "p".to_owned(),
        value: Some("0".to_owned()),
    }];
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        },
        None,
        0,
        2,
        5,
    );

    assert!(wikitext.contains("/listpages-zero-pager/p/2"), "{wikitext}");
    assert!(!wikitext.contains("/listpages-zero-pager/p/0/p/2"));
}

#[test]
fn generated_list_pages_pager_uses_url_attr_prefix() {
    let page_info =
        fallback_test_page_info("listpages-prefixed-pager", "ListPages pager");
    let path_arguments = vec![UrlArgumentPair {
        name: "p".to_owned(),
        value: Some("2".to_owned()),
    }];
    let mut wikitext = String::new();

    push_list_pages_pager(
        &mut wikitext,
        &page_info,
        ListPagesPagerRoute::SavedPage,
        UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        },
        Some("a"),
        0,
        2,
        5,
    );

    assert!(
        wikitext.contains("/listpages-prefixed-pager/p/2/a_p/2"),
        "{wikitext}",
    );
}

#[test]
fn generated_list_pages_pager_uses_ajax_module_route_and_prefix() {
    let page_info = fallback_test_page_info("", "ListPages preview");
    let mut html = String::new();

    push_list_pages_pager(
        &mut html,
        &page_info,
        ListPagesPagerRoute::AjaxModuleConnector,
        UrlArguments::default(),
        Some("x"),
        0,
        2,
        5,
    );

    assert!(
        html.contains(r#"href="/ajax-module-connector.php/x_p/2""#),
        "{html}",
    );
    assert!(!html.contains("//ajax-module-connector.php"), "{html}");
    assert!(!html.contains(r#"href="//"#), "{html}");
}

#[test]
fn accepts_corpus_list_pages_comments_placeholder() {
    let body = "# (%%created_at|%y/%m/%d%%) %%title_linked%% (評価: %%rating%% コメント: %%comments%% 最終コメント: %%commented_at%%)";

    assert!(list_pages_body_variables_supported(body));
    let substituted = substitute_list_pages_variables(
        body,
        &FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("SCP-655-JP".to_owned()),
            alt_title: None,
            slug: Some("scp-655-jp".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: Some(12.0),
            revision_count: None,
        },
        1,
        1,
        &list_pages_substitution_context(20, &BTreeMap::new(), None, &BTreeMap::new()),
    );

    assert_eq!(
        substituted,
        "# () [/scp-655-jp SCP-655-JP] (評価: 12 コメント: 0 最終コメント: )",
    );
    assert_eq!(
        render_list_pages_numbered_rows_with_titles(&substituted, None),
        "<ol data-wikijump-compat-listpages=\"1\">\n<li>() <a href=\"/scp-655-jp\">SCP-655-JP</a> (評価: 12 コメント: 0 最終コメント: )</li>\n</ol>\n",
    );
}

#[test]
fn substitutes_wikidot_list_pages_size_from_saved_source_scalar_values() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Unicode fixture".to_owned()),
        alt_title: None,
        slug: Some("unicode-fixture".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();

    for (source, expected) in [("😀", "1"), ("e\u{301}", "2"), ("a\nb", "3")] {
        assert_eq!(
            substitute_list_pages_variables(
                "%%size%%",
                &page,
                1,
                1,
                &list_pages_substitution_context(
                    20,
                    &user_displays,
                    Some(source),
                    &data_form_values,
                ),
            ),
            expected,
        );
    }
    assert_eq!(
        substitute_list_pages_variables(
            "%%size%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &user_displays, None, &data_form_values,),
        ),
        "%%size%%",
    );
}

#[test]
fn substitutes_wikidot_list_pages_content_sections() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("SCP-2693".to_owned()),
        alt_title: None,
        slug: Some("scp-2693".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let wikitext = concat!(
        "=====\n",
        "[[include component:preview text=Wherein an adorable kitten gets up to no good.]]\n",
        "=====\n",
        "Main page body\n",
        "=====\n",
        "Hidden title text\n",
        "=====\n",
        "License notes\n",
    );

    assert!(list_pages_body_variables_supported(
        "%%title%% -- %%content{4}%%"
    ));
    assert!(list_pages_body_uses_content_variable(
        "%%title%% -- %%content{4}%%"
    ));
    assert_eq!(
        wikidot_content_section(wikitext, Some(4)),
        "Hidden title text",
    );

    let rendered = substitute_list_pages_variables(
        "**%%title%% -- %%content{4}%%**",
        &page,
        1,
        1,
        &list_pages_substitution_context(
            20,
            &BTreeMap::new(),
            Some(wikitext),
            &BTreeMap::new(),
        ),
    );

    assert_eq!(rendered, "**SCP-2693 -- Hidden title text**");

    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let context = list_pages_substitution_context(
        20,
        &user_displays,
        Some("RAW_CONTENT"),
        &data_form_values,
    );
    assert_eq!(
        substitute_list_pages_variables(
            "%%content%%|%%content{1}%%",
            &page,
            1,
            1,
            &context,
        ),
        "RAW_CONTENT|RAW_CONTENT",
        "row-local content must preserve the authored child source phase",
    );
}

#[test]
fn substitutes_static_wikidot_data_form_variables() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Codex data form fixture".to_owned()),
        alt_title: None,
        slug: Some("codex-data-form-fixture".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let values = parse_static_wikidot_data_form_values(
        "codexkind: alpha\ncodexflag: 'df-red'\ncodex-hyphen: ok\n",
    );

    assert!(list_pages_body_variables_supported(
        "%%title%% %%form_raw{codexkind}%% %%form_data{codexflag}%% %%form_raw{codex-hyphen}%%"
    ));
    assert_eq!(
        substitute_list_pages_variables(
            "%%title%% %%form_raw{codexkind}%% %%form_data{codexflag}%% %%form_raw{codex-hyphen}%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &BTreeMap::new(), None, &values),
        ),
        "Codex data form fixture alpha df-red ok",
    );
}

#[test]
fn static_wikidot_data_form_values_do_not_scan_ordinary_body_text() {
    let values = parse_static_wikidot_data_form_values(
        "This is regular page text.\nstatus: published\nowner: codex\n",
    );

    assert!(values.is_empty());

    let values = parse_static_wikidot_data_form_values(
        "\nstatus: published\nowner: codex\nBody text starts here.\nignored: yes\n",
    );

    assert_eq!(values.get("status").map(String::as_str), Some("published"));
    assert_eq!(values.get("owner").map(String::as_str), Some("codex"));
    assert!(!values.contains_key("ignored"));

    assert!(!static_wikidot_data_form_matches(
        &values,
        &[DataFormSelector {
            field: Cow::Borrowed("missing"),
            value: Cow::Borrowed(""),
            negated: false,
        }],
    ));
    assert!(!static_wikidot_data_form_matches(
        &values,
        &[DataFormSelector {
            field: Cow::Borrowed("missing"),
            value: Cow::Borrowed("closed"),
            negated: true,
        }],
    ));
}

#[test]
fn form_list_pages_variables_require_field_arguments() {
    assert!(list_pages_body_variables_supported(
        "%%form_raw{codexkind}%% %%form_data{codexflag}%%"
    ));
    assert!(!list_pages_body_variables_supported("%%form_raw%%"));
    assert!(!list_pages_body_variables_supported("%%form_data%%"));
}

#[test]
fn renders_wikidot_list_pages_table_rows_as_raw_html() {
    let source = concat!(
        "||~ [en] Flopstyle: LITE ||\n",
        "||= **By:** <span class=\"printuser\"><a href=\"http://www.wikidot.com/user:info/stormbreath\">stormbreath</a></span> ||\n",
        "||~ Published on <span class=\"odate time_1782003564 format_%25d%20%25b%20%25Y\">21 Jun 2026</span> ||",
    );
    let rendered = render_list_pages_table_rows(source)
        .expect("authorbox ListPages body should render as raw table HTML");

    assert!(rendered.contains(
        "<table class=\"wiki-content-table\" data-wikijump-compat-listpages=\"1\">"
    ));
    assert!(rendered.contains("<strong>By:</strong>"));
    assert!(rendered.contains("<span class=\"printuser\">"));
    assert!(rendered.contains("<span class=\"odate time_1782003564"));
    assert!(!rendered.contains("&lt;span"));

    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut protected = rendered.clone();
    let fragments =
        RenderService::protect_generated_wikidot_compat_html(&mut protected, &settings);
    assert_eq!(fragments.len(), 1);
    assert!(!protected.contains("<table"));
    let restored = RenderService::restore_protected_generated_wikidot_compat_html(
        protected, &fragments,
    );
    assert!(restored.contains("<table class=\"wiki-content-table\">"));
    assert!(!restored.contains("data-wikijump-compat-listpages"));
}

#[test]
fn registered_list_pages_tables_remain_block_html_in_the_outer_page() {
    let generated = render_list_pages_table_rows(concat!(
        "||~ Today's date is: ||\n",
        "||= <span class=\"odate time_1785291693 format_%25d%20%25B%20%25Y\">",
        "29 Jul 2026 02:21</span> ||",
    ))
    .expect("complete generated table rows");
    let mut fragments = CompatHtmlFragments::new("");
    let mut protected = register_generated_list_pages_html(generated, &mut fragments);
    let page_info = fallback_test_page_info("table-probe", "Table Probe");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);

    ftml::preprocess_for_layout(&mut protected, settings.layout);
    let tokens = ftml::tokenize(&protected);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    let restored = fragments.restore(&rendered);

    assert!(
        restored.starts_with(r#"<table class="wiki-content-table">"#),
        "{restored:?}",
    );
    assert!(!restored.contains("<p><table"));
    assert!(!restored.contains("data-wikijump-compat-listpages"));
}

#[test]
fn unsupported_numbered_list_pages_body_does_not_leak_to_ftml() {
    let module_source = concat!(
        "[[module ListPages unsupported=\"yes\"]]\n",
        "# %%unsupported_variable%%\n",
        "[[/module]]",
    );

    assert_eq!(
        unsupported_list_pages_replacement(module_source, "# %%unsupported_variable%%\n"),
        "[[div class=\"list-pages-box\"]][[/div]]",
    );
}

#[test]
fn unsupported_tracking_list_pages_body_does_not_leak_to_ftml() {
    let body = concat!(
        "[[%%content{0}%%module listusers users=\".\"]]\n",
        "[[image https://manage.scp-jp.com/api/public/assets/layoutSupporter.png?s_id=578002&s_name=scp-jp&m_id=%%number%%&m_name=%%ti%%content{0}%%tle%%&fn=%%fullname%%]]\n",
        "[[%%content{0}%%/module]]\n",
        "[[image https://manage.scp-jp.com/api/public/assets/analytics.png?s_id=578002&fn=%%fullname%%]]\n",
    );
    let module_source = format!(
        "[[module ListPages category=\"*\" pagetype=\"*\" range=\".\" wrapper=\"no\" separate=\"no\"]]\n{body}[[/module]]"
    );

    assert!(list_pages_body_is_no_visible_tracking_markup(body));
    assert_eq!(
        unsupported_list_pages_replacement(&module_source, body),
        "[[div class=\"list-pages-box\"]][[/div]]",
    );
}

fn render_list_pages_title_variables_through_outer_pipeline(
    title: &str,
    parent: Option<&ListPagesParentDisplay>,
) -> String {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some(title.to_owned()),
        alt_title: None,
        slug: Some("title-matrix".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let template = concat!(
        "TITLE=%%title%%\n",
        "LINKED=%%title_linked%%\n",
        "PARENT=%%parent_title%%\n",
        "PARENT_LINKED=%%parent_title_linked%%",
    );
    let mut compat_html = CompatHtmlFragments::new(template);
    let mut compat_text = CompatTextFragments::new(template);
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);
    context.page_parent_display = parent;
    let substituted = substitute_list_pages_variables_with_fragments(
        template,
        &page,
        1,
        1,
        &context,
        &mut compat_html,
        &mut compat_text,
    );

    let page_info = fallback_test_page_info("listing", "Listing");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let outer = RenderService::prepare_outer_render_wikitext(
        super::super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext: substituted,
            included_pages: Vec::new(),
            expanded_include_count: 0,
            wikidot_compat_html: compat_html,
            wikidot_compat_text: compat_text,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
        },
        &page_info,
        &settings,
    );
    let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&inner.wikitext);
    let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    inner.protection.restore(|protection| {
        let rendered = RenderService::restore_protected_wikidot_color_spans(
            rendered,
            protection.color_spans(),
        );
        let rendered = RenderService::restore_protected_wikidot_inline_html(
            rendered,
            protection.inline_html(),
        );
        let rendered = protection.compat_html().restore(&rendered);
        protection.compat_text().restore(&rendered)
    })
}

#[test]
fn list_pages_plain_tag_variables_keep_visible_and_hidden_tags_disjoint() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Tag matrix".to_owned()),
        alt_title: None,
        slug: Some("tag-matrix".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: Some(vec![
            "visible-one".to_owned(),
            "_hidden-one".to_owned(),
            "visible-two".to_owned(),
            "_hidden-two".to_owned(),
        ]),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);

    assert_eq!(
        substitute_list_pages_variables(
            "VISIBLE=%%tags%%|HIDDEN=%%_tags%%",
            &page,
            1,
            1,
            &context,
        ),
        "VISIBLE=visible-one visible-two|HIDDEN=_hidden-one _hidden-two",
    );
}

#[test]
fn list_pages_title_variables_match_live_sanitization_and_context() {
    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "**BOLD** //ITALIC// __UNDER__ --STRIKE--",
        None,
    );
    assert!(rendered.contains("<strong>BOLD</strong>"), "{rendered}");
    assert!(rendered.contains("<em>ITALIC</em>"), "{rendered}");
    assert!(
        rendered.contains(">**BOLD** //ITALIC// __UNDER__ --STRIKE--</a>"),
        "{rendered}",
    );

    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "[[div class=\"owned\"]]DIV[[/div]]",
        None,
    );
    assert!(
        rendered.contains("TITLE=div class=&quot;owned&quot;DIV/div"),
        "{rendered}"
    );
    assert!(
        rendered.contains(">div class=\"owned\"DIV/div</a>"),
        "{rendered}"
    );
    assert!(!rendered.contains("<div class=\"owned\">"));
    assert!(!rendered.contains("[["));

    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "x...x x....x x. . .x",
        None,
    );
    assert!(rendered.contains("TITLE=x…x x….x x…x"), "{rendered}");
    assert!(rendered.contains(">x...x x....x x. . .x</a>"), "{rendered}",);

    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "``DOUBLE'' `SINGLE' <<ANGLE>> 10 000 5 kg",
        None,
    );
    assert!(
        rendered.contains("TITLE=“DOUBLE” ‘SINGLE’ «ANGLE» 10\u{a0}000\u{a0}5\u{a0}kg"),
        "{rendered}",
    );
    assert!(
        rendered.contains(">``DOUBLE'' `SINGLE' &lt;&lt;ANGLE&gt;&gt; 10 000 5 kg</a>"),
        "{rendered}",
    );

    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "##red|COLOR## ^^SUP^^ ,,SUB,, {{CODE}}",
        None,
    );
    assert!(
        rendered.contains(">##red|COLOR## ^^SUP^^ ,,SUB,, {{CODE}}</a>",),
        "{rendered}",
    );

    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "##red|COLOR## ^^SUP^^ ,,SUB,, {{CODE}} @@ESCAPED@@",
        None,
    );
    assert!(
        rendered.contains(
            "LINKED=[[[title-matrix | <span style=\"color: red\">COLOR</span> \
             <sup>SUP</sup> <sub>SUB</sub> <tt>CODE</tt> \
             <span style=\"white-space: pre-wrap;\">ESCAPED</span>]]]",
        ),
        "{rendered}",
    );
    assert!(
        !rendered.contains(r#"<a href="/title-matrix">"#),
        "{rendered}",
    );

    let parent = ListPagesParentDisplay {
        fullname: "run-owned:parent".to_owned(),
        name: "parent".to_owned(),
        category: "run-owned".to_owned(),
        title: "[[div]]**PARENT**... @@ESCAPED@@[[/div]]".to_owned(),
    };
    let rendered = render_list_pages_title_variables_through_outer_pipeline(
        "CHILD @@UNCLOSED",
        Some(&parent),
    );
    assert!(
        rendered.contains("LINKED=<a href=\"/title-matrix\">CHILD @@UNCLOSED</a>",),
        "{rendered}",
    );
    assert!(
        rendered.contains(
            "PARENT=div<strong>PARENT</strong>… \
             <span style=\"white-space: pre-wrap;\">ESCAPED</span>/div",
        ),
        "{rendered}",
    );
    assert!(
        rendered.contains(
            "PARENT_LINKED=[[[run-owned:parent | div<strong>PARENT</strong>… \
             <span style=\"white-space: pre-wrap;\">ESCAPED</span>/div]]]",
        ),
        "{rendered}",
    );
}

#[test]
fn substitutes_wikidot_list_pages_author_and_created_at_variables() {
    let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
        .expect("fixture timestamp should be valid");
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Codex virtual Wikidot DOM 001".to_owned()),
        alt_title: None,
        slug: Some("dom-001".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: Some(created_at),
        created_by: Some(8_955_132),
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let mut users = BTreeMap::new();
    users.insert(
        8_955_132,
        WikidotUserDisplay {
            user_id: 8_955_132,
            name: "scpaiueouiuiuiui".to_owned(),
            slug: Some("scpaiueouiuiuiui".to_owned()),
            wikidot_profile: true,
        },
    );

    let rendered = substitute_list_pages_variables(
        "||~ %%title%% ||\n||= **By:** %%created_by_linked%% ||\n||~ Published on %%created_at|%d %b %Y%% ||",
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );

    assert!(rendered.contains("Codex virtual Wikidot DOM 001"));
    assert!(rendered.contains("user:info/scpaiueouiuiuiui"));
    assert!(rendered.contains("WIKIDOT.page.listeners.userInfo(8955132)"));
    assert!(
        rendered
            .contains("avatar.php?userid=8955132&amp;amp;size=small&amp;amp;timestamp=")
    );
    assert!(rendered.contains(
        r#"style="background-image:url(http://www.wikidot.com/userkarma.php?u=8955132)""#
    ));
    assert!(rendered.contains(
        r#"<span class="odate time_1782003564 format_%25d%20%25b%20%25Y" data-wikijump-compat-date="1">21 Jun 2026 00:59</span>"#
    ));

    let rendered = substitute_list_pages_variables(
        "%%created_at%%",
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );
    assert_eq!(
        rendered,
        r#"<span class="odate time_1782003564 format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover" data-wikijump-compat-date="1">21 Jun 2026 00:59</span>"#
    );

    let rendered = substitute_list_pages_variables(
        "%%date|%r|agohover%%",
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );
    assert!(rendered.contains(r#"class="odate time_1782003564 format_%25r%7Cagohover""#));
    assert!(!rendered.contains("agohover[[/span]]"));

    let rendered = substitute_list_pages_variables(
        "%%linked_title%%",
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );
    assert_eq!(rendered, "[/dom-001 Codex virtual Wikidot DOM 001]");

    let ellipsis_title_page = FoundPageRow {
        title: Some("Now watch and learn, here's the deal...".to_owned()),
        ..page.clone()
    };
    let rendered = substitute_list_pages_variables(
        "%%title_linked%%",
        &ellipsis_title_page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );
    assert_eq!(
        rendered,
        "[/dom-001 Now watch and learn, here's the deal...]"
    );

    let rendered = substitute_list_pages_variables(
        "%%author%%",
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );
    assert!(rendered.contains("printuser avatarhover"));
    assert!(rendered.contains("user:info/scpaiueouiuiuiui"));

    let local_author = FoundPageRow {
        created_by: Some(ADMIN_USER_ID),
        ..page
    };
    let rendered = substitute_list_pages_variables(
        "%%author%%",
        &local_author,
        1,
        1,
        &list_pages_substitution_context(20, &BTreeMap::new(), None, &BTreeMap::new()),
    );
    assert_eq!(rendered, ADMIN_USER_ID.to_string());
    assert!(!rendered.contains("wikidot.com/user:info"));

    let mut local_users = BTreeMap::new();
    local_users.insert(
        -20,
        WikidotUserDisplay {
            user_id: -20,
            name: "SeekGull".to_owned(),
            slug: Some("seekgull".to_owned()),
            wikidot_profile: false,
        },
    );
    let local_mirror_author = FoundPageRow {
        created_by: Some(-20),
        ..local_author
    };
    let rendered = substitute_list_pages_variables(
        "%%created_by%% / %%author%%",
        &local_mirror_author,
        1,
        1,
        &list_pages_substitution_context(20, &local_users, None, &BTreeMap::new()),
    );
    assert_eq!(rendered, "SeekGull / SeekGull");
    assert!(!rendered.contains("wikidot.com/user:info"));
}

#[test]
fn substitutes_wikidot_list_pages_site_domain_and_parent_fullname() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Offset 0".to_owned()),
        alt_title: None,
        slug: Some("fragment:component:offset-timeline-0".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);
    context.page_parent_fullname = Some("component:offset-timeline");

    assert_eq!(
        substitute_list_pages_variables(
            "https://%%site_domain%%/%%parent_fullname%%/offset/",
            &page,
            1,
            2,
            &context,
        ),
        "https://scp-wiki.wikidot.com/component:offset-timeline/offset/",
    );

    context.page_parent_fullname = None;
    assert_eq!(
        substitute_list_pages_variables("%%parent_fullname%%", &page, 1, 2, &context),
        "",
    );
}

#[test]
fn substitutes_wikidot_list_pages_child_count_and_leaves_rating_percent_literal() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Offset timeline".to_owned()),
        alt_title: None,
        slug: Some("component:offset-timeline".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);

    context.page_child_count = Some(2);
    assert_eq!(
        substitute_list_pages_variables(
            "%%children%% %%rating_percent%%",
            &page,
            1,
            1,
            &context,
        ),
        "2 %%rating_percent%%",
    );

    context.page_child_count = Some(0);
    assert_eq!(
        substitute_list_pages_variables("%%children%%", &page, 1, 1, &context),
        "0",
    );

    context.page_child_count = None;
    assert_eq!(
        substitute_list_pages_variables("%%children%%", &page, 1, 1, &context),
        "%%children%%",
    );
}

#[test]
fn substitutes_wikidot_list_pages_revision_count() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Devereaux".to_owned()),
        alt_title: None,
        slug: Some("devereaux".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);

    context.page_revision_count = Some(2);
    assert_eq!(
        substitute_list_pages_variables("%%revisions%%", &page, 1, 1, &context),
        "2",
    );

    context.page_revision_count = None;
    assert_eq!(
        substitute_list_pages_variables("%%revisions%%", &page, 1, 1, &context),
        "%%revisions%%",
    );
}

#[test]
fn resolves_wikidot_list_pages_parent_fullname_from_import_before_relations() {
    let page = FoundPageRow {
        page_id: 101,
        site_id: 1,
        title: Some("Offset 0".to_owned()),
        alt_title: None,
        slug: Some("fragment:component:offset-timeline-0".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let source_created_at = time::OffsetDateTime::UNIX_EPOCH;
    let snapshot = ListPagesSnapshotDisplay {
        title_shown: None,
        created_at: source_created_at,
        updated_at: source_created_at,
        created_by_user_id: None,
        created_by_name: None,
        created_by_slug: None,
        updated_by_user_id: None,
        updated_by_name: None,
        updated_by_slug: None,
        comments: 0,
        commented_at: None,
        commented_by_name: None,
        rating_votes: None,
        parent_fullname: Some("component:offset-timeline".to_owned()),
        source_revision_count: 2,
    };
    let imported = BTreeMap::from([(101, snapshot.clone())]);
    let relational = BTreeMap::from([(
        101,
        ListPagesParentDisplay {
            fullname: "component:local-parent".to_owned(),
            name: "local-parent".to_owned(),
            category: "component".to_owned(),
            title: "Local Parent".to_owned(),
        },
    )]);
    let empty_snapshots = BTreeMap::new();
    let empty_relations = BTreeMap::new();

    assert_eq!(
        list_pages_parent_fullname(&page, &imported, &relational),
        Some("component:offset-timeline"),
    );
    assert_eq!(
        list_pages_parent_fullname(&page, &empty_snapshots, &relational),
        Some("component:local-parent"),
    );
    assert_eq!(
        list_pages_parent_fullname(&page, &empty_snapshots, &empty_relations),
        None,
    );

    let parentless_import = BTreeMap::from([(
        101,
        ListPagesSnapshotDisplay {
            parent_fullname: None,
            ..snapshot
        },
    )]);
    assert_eq!(
        list_pages_parent_fullname(&page, &parentless_import, &relational),
        None,
    );
}

#[test]
fn substitutes_wikidot_list_pages_created_by_unix_from_account_unix_name() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Identity fixture".to_owned()),
        alt_title: None,
        slug: Some("identity-fixture".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: Some(8_955_132),
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let data_form_values = BTreeMap::new();
    let user_displays = BTreeMap::from([(
        8_955_132,
        WikidotUserDisplay {
            user_id: 8_955_132,
            name: "Dr Wondertainment".to_owned(),
            slug: Some("dr-wondertainment".to_owned()),
            wikidot_profile: true,
        },
    )]);

    assert_eq!(
        substitute_list_pages_variables(
            "%%created_by_unix%% %%created_by%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &user_displays, None, &data_form_values,),
        ),
        "dr-wondertainment Dr Wondertainment",
    );

    let slugless_displays = BTreeMap::from([(
        8_955_132,
        WikidotUserDisplay {
            user_id: 8_955_132,
            name: "Dr Wondertainment".to_owned(),
            slug: None,
            wikidot_profile: true,
        },
    )]);
    assert_eq!(
        substitute_list_pages_variables(
            "%%created_by_unix%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &slugless_displays,
                None,
                &data_form_values,
            ),
        ),
        "%%created_by_unix%%",
    );

    let importer_displays = BTreeMap::from([(
        -1,
        WikidotUserDisplay {
            user_id: -1,
            name: "Administrator".to_owned(),
            slug: Some("administrator".to_owned()),
            wikidot_profile: false,
        },
    )]);
    let imported_page = FoundPageRow {
        created_by: Some(-1),
        ..page
    };
    let imported_snapshots = BTreeMap::from([(
        1,
        ListPagesSnapshotDisplay {
            title_shown: None,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
            created_by_user_id: None,
            created_by_name: Some("INT_Translator".to_owned()),
            created_by_slug: None,
            updated_by_user_id: None,
            updated_by_name: None,
            updated_by_slug: None,
            comments: 0,
            commented_at: None,
            commented_by_name: None,
            rating_votes: None,
            parent_fullname: None,
            source_revision_count: 1,
        },
    )]);
    assert_eq!(
        substitute_list_pages_variables(
            "%%created_by%% %%created_by_unix%%",
            &imported_page,
            1,
            1,
            &list_pages_substitution_context_with_mode(
                20,
                &importer_displays,
                &imported_snapshots,
                None,
                &data_form_values,
                false,
            ),
        ),
        "INT_Translator %%created_by_unix%%",
    );
}

#[test]
fn substitutes_wikidot_list_pages_limit_variable() {
    let body = "%%index%%/%%total%% limit=%%limit%% %%title%%";
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Codex fixture".to_owned()),
        alt_title: None,
        slug: Some("codex-fixture".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };

    assert!(list_pages_body_variables_supported(body));
    assert_eq!(
        substitute_list_pages_variables(
            body,
            &page,
            2,
            7,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new()
            ),
        ),
        "2/7 limit=20 Codex fixture",
    );

    let user_displays = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &user_displays, None, &data_form_values);
    context.authored_limit = None;
    assert_eq!(
        substitute_list_pages_variables(body, &page, 2, 7, &context),
        "2/7 limit= Codex fixture",
    );
}

#[test]
fn distinguishes_wikidot_list_pages_link_and_fullname() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Fixture component".to_owned()),
        alt_title: None,
        slug: Some("component:black-highlighter-theme-dev".to_owned()),
        page_category_id: Some(1),
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let users = BTreeMap::new();
    let data_form_values = BTreeMap::new();
    let mut context =
        list_pages_substitution_context(20, &users, None, &data_form_values);
    context.category = "component";

    assert_eq!(
        substitute_list_pages_variables(
            concat!(
                "%%name%%|%%slug%%|%%page_unix_name%%|",
                "%%fullname%%|%%full_slug%%|%%link%%|%%title_linked%%",
            ),
            &page,
            1,
            1,
            &context,
        ),
        concat!(
            "black-highlighter-theme-dev|",
            "black-highlighter-theme-dev|",
            "component:black-highlighter-theme-dev|",
            "component:black-highlighter-theme-dev|",
            "component:black-highlighter-theme-dev|",
            "http://scp-wiki.wikidot.com/component:black-highlighter-theme-dev",
            "|[/component:black-highlighter-theme-dev Fixture component]",
        ),
    );
}

#[test]
fn substitutes_wikidot_list_pages_author_tool_variables() {
    let updated_at = time::OffsetDateTime::from_unix_timestamp(1_782_005_400)
        .expect("fixture timestamp should be valid");
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("SCP-2693".to_owned()),
        alt_title: None,
        slug: Some("scp-2693".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: Some(vec![
            "_image".to_owned(),
            "scp".to_owned(),
            "safe".to_owned(),
        ]),
        created_at: None,
        created_by: None,
        updated_at: Some(updated_at),
        updated_by: Some(954_000_337),
        score: Some(42.0),
        revision_count: None,
    };
    let mut users = BTreeMap::new();
    users.insert(
        954_000_337,
        WikidotUserDisplay {
            user_id: 954_000_337,
            name: "Calibold".to_owned(),
            slug: Some("calibold".to_owned()),
            wikidot_profile: true,
        },
    );

    let body = concat!(
        "**%%title_linked%%**\n",
        "**Rating:** +%%rating%%\n",
        "**Comments:** %%comments%%\n",
        "**Last Comment:** %%commented_by%% (//%%commented_at|%D %H:%M|agohover%%//)\n",
        "**Last Edit:** %%updated_by%% (//%%updated_at|%D %H:%M|agohover%%//)\n",
        "**Edited date:** //%%date_edited|%D %H:%M|agohover%%//\n",
        "%%tags_linked%%\n",
        "%%link%%",
    );
    assert!(list_pages_body_variables_supported(body));

    let rendered = substitute_list_pages_variables(
        body,
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
    );

    assert!(rendered.contains("[/scp-2693 SCP-2693]"));
    assert!(rendered.contains("**Rating:** +42"));
    assert!(rendered.contains("**Last Edit:** Calibold"));
    assert_eq!(
        rendered
            .matches(r#"<span class="odate time_1782005400"#)
            .count(),
        2,
    );
    assert!(rendered.contains(r#"data-wikijump-compat-date="1""#));
    assert!(rendered.contains("[/system:page-tags/tag/scp scp]"));
    assert!(rendered.contains("[/system:page-tags/tag/safe safe]"));
    assert!(rendered.ends_with("http://scp-wiki.wikidot.com/scp-2693"));
    assert!(!rendered.contains("%%updated_by%%"));
    assert!(!rendered.contains("%%tags_linked%%"));
}

#[test]
fn substitutes_wikidot_list_pages_hidden_tags_as_links() {
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Hidden tags".to_owned()),
        alt_title: None,
        slug: Some("hidden-tags".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: Some(vec![
            "_image".to_owned(),
            "scp".to_owned(),
            "_licensebox".to_owned(),
            "safe".to_owned(),
        ]),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };

    let rendered = substitute_list_pages_variables(
        "%%_tags_linked%%",
        &page,
        1,
        1,
        &list_pages_substitution_context_with_mode(
            20,
            &BTreeMap::new(),
            empty_list_pages_snapshot_displays(),
            None,
            &BTreeMap::new(),
            true,
        ),
    );

    assert_eq!(
        rendered,
        r#"<a href="/system:page-tags/tag/_image">_image</a> <a href="/system:page-tags/tag/_licensebox">_licensebox</a>"#,
    );
    assert!(!rendered.contains(">scp<"));
    assert!(!rendered.contains(">safe<"));

    let no_hidden_tags = FoundPageRow {
        tags: Some(vec!["scp".to_owned(), "safe".to_owned()]),
        ..page
    };
    assert_eq!(
        substitute_list_pages_variables(
            "%%_tags_linked%%",
            &no_hidden_tags,
            1,
            1,
            &list_pages_substitution_context_with_mode(
                20,
                &BTreeMap::new(),
                empty_list_pages_snapshot_displays(),
                None,
                &BTreeMap::new(),
                true,
            ),
        ),
        "",
    );
}

#[test]
fn protects_list_pages_tag_labels_and_encodes_href_segments_independently() {
    let tags = vec![r#"safe] [[span class="owned"]]<img onerror='x'> 日本"#.to_owned()];
    let mut fragments = CompatHtmlFragments::new("");
    let protected = render_list_pages_tags(
        &tags,
        Some("/system:page-tags/tag/"),
        false,
        &mut fragments,
    );

    assert!(protected.contains("safe%5D%20%5B%5Bspan%20class%3D%22owned%22%5D%5D%3Cimg%20onerror%3D%27x%27%3E%20%E6%97%A5%E6%9C%AC"));
    assert!(!protected.contains("<img"));
    let restored = fragments.restore(&protected);
    assert!(restored.contains("safe&#x5D;&#x20;&#x5B;&#x5B;span"));
    assert!(!restored.contains("<img"));
    assert_eq!(
        fragments.restore_plain(&protected),
        format!(
            "[/system:page-tags/tag/safe%5D%20%5B%5Bspan%20class%3D%22owned%22%5D%5D%3Cimg%20onerror%3D%27x%27%3E%20%E6%97%A5%E6%9C%AC {}]",
            tags[0]
        )
    );
    assert_eq!(
        list_pages_tag_link_href("/tag/] [[span/", "safe tag"),
        "/tag/%5D%20%5B%5Bspan/safe%20tag",
    );
}

#[test]
fn restores_dense_list_pages_labels_once_inside_registered_table_html() {
    let tags = (0..10_000)
        .map(|index| format!("tag-{index}]<"))
        .collect::<Vec<_>>();
    let mut fragments = CompatHtmlFragments::new("");
    let links = render_list_pages_tags(&tags, None, true, &mut fragments);
    let table = format!(
        r#"<table class="wiki-content-table" data-wikijump-compat-listpages="1"><tr><td>{links}</td></tr></table>"#
    );
    let protected = register_generated_list_pages_html(table, &mut fragments);
    let restored = fragments.restore(&protected);

    assert_eq!(restored.matches("<a href=").count(), 10_000);
    assert!(restored.contains(r#"href="/system:page-tags/tag/tag-9999%5D%3C""#));
    assert!(restored.contains("tag-9999&#x5D;&#x3C;"));
    assert!(!restored.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
}

#[test]
fn substitutes_imported_wikidot_snapshot_metadata_for_list_pages_rows() {
    let local_created_at = time::OffsetDateTime::from_unix_timestamp(1_600_000_000)
        .expect("fixture timestamp should be valid");
    let source_created_at = time::OffsetDateTime::from_unix_timestamp(1_781_900_521)
        .expect("fixture timestamp should be valid");
    let source_commented_at = time::OffsetDateTime::from_unix_timestamp(1_781_934_132)
        .expect("fixture timestamp should be valid");
    let page = FoundPageRow {
        page_id: 101,
        site_id: 1,
        title: Some("Aspenq Pride Art 2026".to_owned()),
        alt_title: None,
        slug: Some("aspenq-pride-art-2026".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: Some(local_created_at),
        created_by: Some(ADMIN_USER_ID),
        updated_at: Some(local_created_at),
        updated_by: Some(ADMIN_USER_ID),
        score: Some(28.0),
        revision_count: None,
    };
    let mut users = BTreeMap::new();
    users.insert(
        ADMIN_USER_ID,
        WikidotUserDisplay {
            user_id: ADMIN_USER_ID,
            name: "Administrator".to_owned(),
            slug: Some("admin".to_owned()),
            wikidot_profile: false,
        },
    );
    let mut snapshots = BTreeMap::new();
    snapshots.insert(
        101,
        ListPagesSnapshotDisplay {
            title_shown: None,
            created_at: source_created_at,
            updated_at: source_created_at,
            created_by_user_id: None,
            created_by_name: Some("Aspenq".to_owned()),
            created_by_slug: None,
            updated_by_user_id: None,
            updated_by_name: Some("Aspenq".to_owned()),
            updated_by_slug: None,
            comments: 10,
            commented_at: Some(source_commented_at),
            commented_by_name: Some("Aspenq".to_owned()),
            rating_votes: Some(31),
            parent_fullname: None,
            source_revision_count: 37,
        },
    );

    let rendered = substitute_list_pages_variables(
        "%%title%% by %%author%% on %%created_at|%Y %b %e|agohover%% -- %%comments%% Comments -- %%commented_by%% %%commented_at|%Y %b %e%% -- %%rating_votes%% votes",
        &page,
        1,
        1,
        &list_pages_substitution_context_with_mode(
            20,
            &users,
            &snapshots,
            None,
            &BTreeMap::new(),
            false,
        ),
    );

    assert!(rendered.contains("Aspenq Pride Art 2026 by "));
    assert!(rendered.contains("by Aspenq on "));
    assert!(rendered.contains("19 Jun 2026 20:22"));
    assert!(rendered.contains("10 Comments"));
    assert!(rendered.contains("-- Aspenq "));
    assert!(rendered.contains("-- 31 votes"));
    assert!(!rendered.contains(r#"style="cursor: help; display: inline;""#));
    assert!(!rendered.contains("Administrator"));
    assert_eq!(
        rendered.matches("data-wikijump-compat-date=\"1\"").count(),
        2
    );
    assert!(!rendered.contains("user:info"));
    assert!(!rendered.contains("2020 Sep"));
    assert!(!rendered.contains("%%comments%%"));
}

#[test]
fn missing_snapshot_vote_count_uses_zero_vote_ratio_state() {
    let page = FoundPageRow {
        page_id: 101,
        site_id: 1,
        title: Some("Ratio page".to_owned()),
        alt_title: None,
        slug: Some("ratio-page".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: Some(49.0),
        revision_count: None,
    };
    let body = concat!(
        "[[#ifexpr %%rating_votes%% == 0 | zero-vote | has-votes]] ",
        "[[#expr (%%rating%%+%%rating_votes%%)/2]] ",
        "[[#expr (%%rating_votes%%-%%rating%%)/2/%%rating_votes%%*(-180)]]",
    );

    let rendered = substitute_list_pages_variables(
        body,
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &BTreeMap::new(), None, &BTreeMap::new()),
    );

    // The pinned FTML parser-functions implementation reports division by
    // zero instead of manufacturing a numeric result.
    assert_eq!(rendered, "zero-vote 24.5 run-time error: division by zero");
    assert!(!rendered.contains("[[#"));
}

#[test]
fn substitutes_wikidot_list_pages_table_body_generated_variables_as_html() {
    let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
        .expect("fixture timestamp should be valid");
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Codex virtual Wikidot DOM 001".to_owned()),
        alt_title: None,
        slug: Some("dom-001".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: Some(vec![
            "_image".to_owned(),
            "scp".to_owned(),
            "safe".to_owned(),
            "preview".to_owned(),
        ]),
        created_at: Some(created_at),
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: None,
        revision_count: None,
    };
    let body = concat!(
        "||~ Published on %%created_at|%d %b %Y%% ||\n",
        "||= %%tags_linked%% ||",
    );

    let empty_users = BTreeMap::new();
    let empty_data_form = BTreeMap::new();
    let mut preview_context = list_pages_substitution_context_with_mode(
        20,
        &empty_users,
        empty_list_pages_snapshot_displays(),
        None,
        &empty_data_form,
        true,
    );
    preview_context.page_preview = true;
    let substituted =
        substitute_list_pages_variables(body, &page, 1, 1, &preview_context);

    assert!(substituted.contains(
        r#"<span class="odate time_1782003564 format_%25d%20%25b%20%25Y">21 Jun 2026 00:59</span>"#
    ));
    assert!(substituted.contains(r#"<a href="/system:page-tags/tag/scp">scp</a>"#));
    assert!(
        substituted.contains(r#"<a href="/system:page-tags/tag/preview">preview</a>"#)
    );
    assert!(!substituted.contains("_image"));
    assert!(!substituted.contains("[[span"));
    assert!(!substituted.contains("[/system:page-tags/tag/scp scp]"));

    let rendered = render_list_pages_table_rows(&substituted)
        .expect("table-shaped ListPages body should render as raw table HTML");

    assert!(rendered.contains("<table class=\"wiki-content-table\""));
    assert!(rendered.contains(r#"<span class="odate time_1782003564"#));
    assert!(rendered.contains(r#"<a href="/system:page-tags/tag/scp">scp</a>"#));
    assert!(rendered.contains(r#"<a href="/system:page-tags/tag/preview">preview</a>"#));
    assert!(!rendered.contains("&lt;span"));
    assert!(!rendered.contains("&lt;a href"));
}

#[test]
fn substitutes_artwork_hub_listpages_body_without_visible_html_or_parser_functions() {
    let created_at = time::OffsetDateTime::from_unix_timestamp(1_781_900_521)
        .expect("fixture timestamp should be valid");
    let page = FoundPageRow {
        page_id: 1,
        site_id: 1,
        title: Some("Aspenq Pride Art 2026".to_owned()),
        alt_title: None,
        slug: Some("aspenq-pride-art-2026".to_owned()),
        page_category_id: None,
        page_revision_id: None,
        tags: Some(vec![
            "_image".to_owned(),
            "_licensebox".to_owned(),
            "artwork".to_owned(),
            "preview".to_owned(),
            "colored-pencil".to_owned(),
            "pridefest2026".to_owned(),
        ]),
        created_at: Some(created_at),
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: Some(28.0),
        revision_count: None,
    };
    let body = concat!(
        "[[div class=\"tale-block %%tags%%\"]]\n",
        "[[div_ class=\"title\"]]%%linked_title%%[[/div]]\n",
        "[[div_ class=\"date\"]]%%created_at|%Y %b %e|agohover%%[[/div]]\n",
        "[[span class=\"tag-list\"]]%%tags_linked|artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,%%[[/span]]\n",
        "[[span class=\"rating\"]][[#ifexpr %%rating%% > -1 | + | - ]][[#expr abs(%%rating%%)]][[/span]]\n",
        "[[/div]]",
    );

    let rendered = substitute_list_pages_variables(
        body,
        &page,
        1,
        1,
        &list_pages_substitution_context(20, &BTreeMap::new(), None, &BTreeMap::new()),
    );

    assert!(rendered.contains(
        "[[div class=\"tale-block artwork preview colored-pencil pridefest2026\"]]"
    ));
    assert!(!rendered.contains("_image"));
    assert!(!rendered.contains("_licensebox"));
    assert!(rendered.contains("[/aspenq-pride-art-2026 Aspenq Pride Art 2026]"));
    assert!(rendered.contains(r#"<span class="odate time_1781900521 format_%25Y%20%25b%20%25e%7Cagohover" data-wikijump-compat-date="1">19 Jun 2026 20:22</span>"#));
    assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,artwork artwork]"));
    assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,preview preview]"));
    assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,colored-pencil colored-pencil]"));
    assert!(rendered.contains(r#"[[span class="rating"]]+28[[/span]]"#));
    assert_eq!(
        rendered.matches("data-wikijump-compat-date=\"1\"").count(),
        1
    );
    assert!(!rendered.contains("<a href"));
    assert!(!rendered.contains("&lt;span"));
    assert!(!rendered.contains("[[#ifexpr"));
    assert!(!rendered.contains("[[#expr"));
    assert!(!rendered.contains("agohover[[/span]]"));
}

#[test]
fn parses_negative_singular_list_pages_tag_as_exclusion() {
    let arguments =
        parse_list_pages_arguments(r#"tag="-excluded" limit="10" order="name""#)
            .expect("negative singular tag selector should parse as exclusion");

    assert_eq!(arguments.no_tags, vec![Cow::Borrowed("excluded")]);
    assert_eq!(arguments.limit, Some(10));
}

#[test]
fn list_pages_inline_tag_preservation_sanitizes_attributes() {
    let html = super::super::render_list_pages_table_inline_html(concat!(
        r#"<span class="safe" onclick="alert(1)">ok</span> "#,
        r#"<a href="/safe" onclick="alert(2)">link</a> "#,
        r#"<img src="javascript:alert(3)" onerror="alert(4)">"#,
    ));

    assert!(html.contains(r#"<span class="safe">ok</span>"#));
    assert!(html.contains(r#"<a href="/safe">link</a>"#));
    assert!(html.contains("<img>"));
    assert!(!html.contains("onclick"));
    assert!(!html.contains("onerror"));
    assert!(!html.contains("javascript:"));
}

#[test]
fn substantial_url_offset_listpages_content_gets_a_bounded_deadline_floor() {
    let mut config = Config::integration_testing();
    config.preprocess_timeout = Duration::from_millis(500);
    config.render_timeout = Duration::from_millis(2_000);
    let source = "ordinary prose";

    assert_eq!(
        RenderService::ftml_compat_render_timeout(
            &config,
            source,
            MIN_URL_OFFSET_LISTPAGES_CONTENT_BYTES - 1,
        ),
        Duration::from_millis(2_500),
    );
    assert_eq!(
        RenderService::ftml_compat_render_timeout(
            &config,
            source,
            MIN_URL_OFFSET_LISTPAGES_CONTENT_BYTES,
        ),
        Duration::from_secs(MIN_URL_OFFSET_LISTPAGES_RENDER_TIMEOUT_SECS),
    );
}

#[test]
fn url_offset_listpages_deadline_does_not_shorten_configured_timeout() {
    let mut config = Config::integration_testing();
    config.preprocess_timeout = Duration::from_secs(3);
    config.render_timeout = Duration::from_secs(9);

    assert_eq!(
        RenderService::ftml_compat_render_timeout(
            &config,
            "ordinary prose",
            MIN_URL_OFFSET_LISTPAGES_CONTENT_BYTES,
        ),
        Duration::from_secs(12),
    );
}
