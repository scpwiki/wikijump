use super::*;

fn prepare_test_nested_include_conditionals(
    source: &str,
    variables: &[(&'static str, &'static str)],
    tags: &[&'static str],
) -> String {
    let include = IncludeRef::new(
        PageRef::page_only("component:test"),
        variables
            .iter()
            .map(|&(name, value)| (Cow::Borrowed(name), Cow::Borrowed(value)))
            .collect(),
    );
    let mut page_info = fallback_test_page_info("consumer", "Consumer");
    page_info.tags = tags.iter().map(|&tag| Cow::Borrowed(tag)).collect();
    let mut source = source.to_owned();
    let mut compat_text = CompatTextFragments::new(&source);
    super::super::prepare_include_source_variables_and_comment_branches(
        &mut source,
        &include,
        &page_info,
        &mut compat_text,
    )
    .expect("test include source should remain within the byte budget");
    let mut preserved = CompatTextFragments::new(&source);
    RenderService::prepare_wikidot_conditionals_before_include_expansion(
        &mut source,
        &page_info,
        &mut preserved,
        1,
    );
    compat_text.restore(&preserved.restore(&source))
}

fn resolve_test_included_variable_iftags(
    source: &str,
    variables: &[(&'static str, &'static str)],
    tags: &[&'static str],
) -> String {
    let include = IncludeRef::new(
        PageRef::page_only("component:test"),
        variables
            .iter()
            .map(|&(name, value)| (Cow::Borrowed(name), Cow::Borrowed(value)))
            .collect(),
    );
    let mut page_info = fallback_test_page_info("consumer", "Consumer");
    page_info.tags = tags.iter().map(|&tag| Cow::Borrowed(tag)).collect();
    let mut source = source.to_owned();

    super::super::apply_include_variables_before_resolving_iftags(
        &mut source,
        &include,
        &page_info,
    )
    .expect("test include variables should remain within the byte budget");
    resolve_test_wikidot_iftags(&mut source, &page_info);
    source
}

#[tokio::test]
async fn include_source_cache_loads_each_canonical_page_once() {
    let first_ref = PageRef::page_only("Component:Sybadge#first");
    let second_ref = PageRef::page_only("component:sybadge/second");
    assert_eq!(first_ref.page(), second_ref.page());

    let mut cache = IncludeSourceCache::default();
    let mut load_count = 0;
    let mut first = cache
        .get_or_try_insert_with(1, first_ref.page(), || {
            load_count += 1;
            async { Ok(Some("CACHE-{$label}-END".to_owned())) }
        })
        .await
        .expect("the first include source load should succeed")
        .expect("the first include source should be available");
    let first_include = IncludeRef::new(
        first_ref,
        VariableMap::from([(Cow::Borrowed("label"), Cow::Borrowed("first"))]),
    );
    super::super::apply_include_variables(&mut first, &first_include)
        .expect("first cached source should remain within the byte budget");

    let mut second = cache
        .get_or_try_insert_with(1, second_ref.page(), || {
            load_count += 1;
            async { Ok(Some("CHANGED".to_owned())) }
        })
        .await
        .expect("the cached include source load should succeed")
        .expect("the cached include source should be available");
    let second_include = IncludeRef::new(
        second_ref,
        VariableMap::from([(Cow::Borrowed("label"), Cow::Borrowed("second"))]),
    );
    super::super::apply_include_variables(&mut second, &second_include)
        .expect("second cached source should remain within the byte budget");

    assert_eq!(load_count, 1);
    assert_eq!(first, "CACHE-first-END");
    assert_eq!(second, "CACHE-second-END");

    let explicit_default = cache
        .get_or_try_insert_with(1, "_default:home", || {
            load_count += 1;
            async { Ok(Some("DEFAULT-PAGE".to_owned())) }
        })
        .await
        .expect("the explicit default-category source load should succeed");
    let implicit_default = cache
        .get_or_try_insert_with(1, "home", || {
            load_count += 1;
            async { Ok(Some("CHANGED-DEFAULT-PAGE".to_owned())) }
        })
        .await
        .expect("the canonical default-category source load should succeed");
    assert_eq!(explicit_default, implicit_default);

    let other_site = cache
        .get_or_try_insert_with(2, "component:sybadge", || {
            load_count += 1;
            async { Ok(Some("OTHER-SITE".to_owned())) }
        })
        .await
        .expect("the other-site include source load should succeed");
    assert_eq!(other_site.as_deref(), Some("OTHER-SITE"));

    let unavailable = cache
        .get_or_try_insert_with(1, "component:private", || {
            load_count += 1;
            async { Ok(None) }
        })
        .await
        .expect("the unavailable include source load should succeed");
    assert_eq!(unavailable, None);
    let unavailable_again = cache
        .get_or_try_insert_with(1, "component:private", || {
            load_count += 1;
            async { Ok(Some("PRIVATE".to_owned())) }
        })
        .await
        .expect("the cached unavailable source load should succeed");
    assert_eq!(unavailable_again, None);
    assert_eq!(load_count, 4);
}

#[tokio::test]
async fn include_source_cache_reuses_found_and_missing_sites_but_not_errors() {
    let mut cache = IncludeSourceCache::default();
    let mut site = wikidot_site("scpwiki", Some("scp-wiki.wikidot.com"));
    site.site_id = 7;
    let mut load_count = 0;

    let first_by_id = cache
        .get_site_by_id_or_try_insert_with(7, || {
            load_count += 1;
            async { Ok(Some(site.clone())) }
        })
        .await
        .expect("the first site ID lookup should succeed");
    let second_by_id = cache
        .get_site_by_id_or_try_insert_with(7, || {
            load_count += 1;
            async { Ok(None) }
        })
        .await
        .expect("the cached site ID lookup should succeed");
    assert_eq!(first_by_id, second_by_id);

    let canonical_slug = cache
        .get_site_by_slug_or_try_insert_with("scpwiki", || {
            load_count += 1;
            async { Ok(Some(site.clone())) }
        })
        .await
        .expect("the canonical slug lookup should succeed independently");
    assert_eq!(canonical_slug, first_by_id);

    let missing = cache
        .get_site_by_slug_or_try_insert_with("scp-int", || {
            load_count += 1;
            async { Ok(None) }
        })
        .await
        .expect("the first missing-site lookup should succeed");
    let missing_again = cache
        .get_site_by_slug_or_try_insert_with("scp-int", || {
            load_count += 1;
            async { Ok(Some(site.clone())) }
        })
        .await
        .expect("the cached missing-site lookup should succeed");
    assert_eq!(missing, None);
    assert_eq!(missing_again, None);

    let failed = cache
        .get_site_by_slug_or_try_insert_with("retryable", || {
            load_count += 1;
            async { Err(include_error()) }
        })
        .await;
    assert!(failed.is_err());
    let recovered = cache
        .get_site_by_slug_or_try_insert_with("retryable", || {
            load_count += 1;
            async { Ok(Some(site.clone())) }
        })
        .await
        .expect("a failed lookup should be retried");
    assert_eq!(recovered, Some(site));
    assert_eq!(load_count, 5);
}

#[test]
fn expands_wikidot_image_block_includes_with_defaults_and_arguments() {
    let mut wikitext = concat!(
        "[[include component:image-block\n",
        "    name=theend.jpg|\n",
        "    caption=The end title card.\n",
        "]]\n",
        "[[include component:image-block name=steel.png|align=center|width=100%|caption=Steel frame.|alt=alt|alt-text=A steel frame.]]\n",
        "[[include component:image-block-base name=raw.jpg]]\n",
    )
    .to_owned();

    let page_info = fallback_test_page_info("scp-3922", "SCP-3922");
    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(
        wikitext.contains(
            r#"[[div class="scp-image-block block-right" style="width:300px;"]]"#
        )
    );
    assert!(wikitext.contains(
        "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/theend.jpg]]"
    ));
    assert!(wikitext.contains("The end title card."));
    assert!(
        wikitext.contains(
            r#"[[div class="scp-image-block block-center" style="width:100%;"]]"#
        )
    );
    assert!(wikitext.contains(
        r#"[[image http://scp-wiki.wikidot.com/local--files/scp-3922/steel.png alt="A steel frame."]]"#
    ));
    assert!(wikitext.contains("[[include component:image-block-base name=raw.jpg]]"));
    assert!(!wikitext.contains("[[include component:image-block\n"));
    assert_eq!(
        included_pages,
        vec![
            PageRef::page_only("component:image-block"),
            PageRef::page_only("component:image-block-base"),
            PageRef::page_only("component:image-block"),
            PageRef::page_only("component:image-block-base"),
        ],
    );

    let mut category_page_info = fallback_test_page_info("basalt", "Basalt Theme");
    category_page_info.category = Some(Cow::Borrowed("theme"));
    assert_eq!(
        RenderService::wikidot_image_block_source("logo.svg", &category_page_info, None,),
        "http://scp-wiki.wikidot.com/local--files/theme:basalt/logo.svg"
    );
}

#[test]
fn expands_multiline_image_block_parameters_before_following_article_text() {
    let mut wikitext = concat!(
        "[[include component:image-block\n",
        "    name=Replication.jpg|\n",
        "    caption=Researcher O'Conner and Dr. Wettle.|\n",
        "    align=center|\n",
        "    width=100%|\n",
        "]]\n",
        "\n",
        "**Researcher O'Conner:** The following dialogue is outside the image block.\n",
        "\n",
        "[[include component:image-block\n",
        "    name=Egg.jpg|\n",
        "    caption=Dr. Dan's breakfast for 16 July 2022.|\n",
        "    align=center|\n",
        "    width=700px|\n",
        "]]\n",
        "\n",
        "|+ A later table\n",
        "|| a || b ||\n",
    )
    .to_owned();
    let page_info = fallback_test_page_info("scp-7000", "SCP-7000");

    RenderService::normalize_wikidot_multiline_includes(&mut wikitext);
    assert!(
        wikitext.contains(
            "[[include component:image-block name=Replication.jpg| caption=Researcher O'Conner and Dr. Wettle.| align=center| width=100%|]]"
        ),
        "normalized include source: {wikitext}"
    );
    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(
        wikitext.contains(
            r#"[[div class="scp-image-block block-center" style="width:100%;"]]"#
        )
    );
    assert!(wikitext.contains(
        r#"[[div class="scp-image-block block-center" style="width:700px;"]]"#
    ));
    assert!(wikitext.contains("Researcher O'Conner and Dr. Wettle.\n[[/div]]\n[[/div]]"));
    assert!(wikitext.contains(
        "[[/div]]\n[[/div]]\n\n**Researcher O'Conner:** The following dialogue is outside the image block."
    ));
    assert!(
        wikitext.contains("Dr. Dan's breakfast for 16 July 2022.\n[[/div]]\n[[/div]]")
    );
    assert!(wikitext.contains("|+ A later table\n|| a || b ||"));
    assert!(!wikitext.contains("align=center|"));
    assert!(!wikitext.contains("width=100%|"));
    assert!(!wikitext.contains("width=700px|"));
    assert_eq!(included_pages.len(), 4);
}

#[test]
fn expands_wikidot_image_block_includes_with_nested_caption_markup() {
    let mut wikitext = concat!(
        "[[include :scp-wiki:component:image-block ",
        "name=linked.jpg|caption=See [[[SCP-173|the statue]]] for details.]]\n",
    )
    .to_owned();
    let page_info = fallback_test_page_info("scp-3922", "SCP-3922");

    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(wikitext.contains("See [[[SCP-173|the statue]]] for details."));
    assert!(wikitext.contains(
        "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/linked.jpg]]"
    ));
    assert_eq!(
        included_pages,
        vec![
            PageRef::page_and_site("scp-wiki", "component:image-block"),
            PageRef::page_and_site("scp-wiki", "component:image-block-base"),
        ],
    );
}

#[test]
fn image_block_prepass_keeps_extensionless_root_attachments_implicit() {
    let mut wikitext =
        "[[include component:image-block name=Lobster|caption=An attachment.]]"
            .to_owned();
    let page_info = fallback_test_page_info("scp-5516", "SCP-5516");

    RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info, None);

    assert!(wikitext.contains("[[image Lobster]]"), "{wikitext}");
    assert!(
        !wikitext.contains("local--files/scp-5516/Lobster"),
        "{wikitext}"
    );
}

#[test]
fn expands_scp_2117_shaped_image_block_with_fragment_src_and_absolute_link() {
    let mut wikitext = concat!(
        "[[include component:image-block ",
        "name=2117.png|alt=alt|alt-text=An image|",
        "link=\"https://scp-wiki.wdfiles.com/local--files/fragment:2117-1/2117.png\"]]",
    )
    .to_owned();
    let page_info = fallback_test_page_info("scp-2117", "SCP-2117");

    RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        Some(("scp-wiki", "fragment:2117-1")),
    );

    assert!(wikitext.contains(
        r#"[[image http://scp-wiki.wikidot.com/local--files/fragment:2117-1/2117.png alt="An image" link="https://scp-wiki.wdfiles.com/local--files/fragment:2117-1/2117.png"]]"#,
    ), "{wikitext}");
    assert!(!wikitext.contains("%22https%3A"), "{wikitext}");
    assert!(
        !wikitext.contains("/local--files/scp-2117/2117.png"),
        "{wikitext}"
    );
}

#[test]
fn image_block_prepass_preserves_quoted_hash_link_distinction() {
    let page_info = fallback_test_page_info("consumer", "Consumer");

    for (raw_link, expected) in [
        (
            "#",
            "[[image http://scp-wiki.wikidot.com/local--files/consumer/fixture.png]]",
        ),
        (
            "\"#\"",
            "[[image http://scp-wiki.wikidot.com/local--files/consumer/fixture.png link=\"#\"]]",
        ),
    ] {
        let mut wikitext = format!(
            "[[include component:image-block name=fixture.png|link={raw_link}]]",
        );

        RenderService::expand_wikidot_image_block_includes(
            &mut wikitext,
            &page_info,
            None,
        );

        assert!(
            wikitext.contains(expected),
            "raw_link={raw_link:?}: {wikitext}"
        );
    }
}

#[test]
fn image_block_prepass_consumes_forwarded_name_and_link_provenance() {
    let variables = [
        (Cow::Borrowed("asset"), Cow::Borrowed("source image.png")),
        (Cow::Borrowed("href"), Cow::Borrowed("source full.png")),
    ]
    .into_iter()
    .collect::<VariableMap<'_>>();
    let owners = [
        (
            "asset".to_owned(),
            AttachmentOwner {
                site_slug: "source-site".into(),
                page_slug: "fragment:source".into(),
            },
        ),
        (
            "href".to_owned(),
            AttachmentOwner {
                site_slug: "link-site".into(),
                page_slug: "fragment:link-source".into(),
            },
        ),
    ]
    .into_iter()
    .collect::<AttachmentVariableOwners>();
    let mut provenance = AttachmentProvenanceRegistry::default();
    let mut wikitext = concat!(
        "[[include[!-- opening gap --] component:image-block ",
        "[!-- argument gap [x] | still comment --] name [!-- key gap --] = ",
        "[!-- value gap --] \"{$asset}\"|name=default.png|",
        "link [!-- link gap --] = '{$href}'|link=default-full.png|",
        "caption=\"quoted ]] | caption\"]]\n",
        "after",
    )
    .to_owned();
    protect_forwarded_attachment_variables(
        &mut wikitext,
        &variables,
        &owners,
        &mut provenance,
    );
    assert!(wikitext.contains("__wj_attachment_"), "{wikitext}");

    let page_info = fallback_test_page_info("consumer", "Consumer");
    RenderService::expand_wikidot_image_block_includes_with_provenance(
        &mut wikitext,
        &page_info,
        Some(("intermediate", "component:image-block")),
        Some(&provenance),
    );

    assert!(
        wikitext.contains(concat!(
            "https://source-site.wikidot.com/local--files/",
            "fragment:source/source%20image.png",
        )),
        "{wikitext}",
    );
    assert!(
        wikitext.contains(concat!(
            "link='https://link-site.wikidot.com/local--files/",
            "fragment:link-source/source%20full.png'",
        )),
        "{wikitext}",
    );
    assert!(!wikitext.contains("__wj_attachment_"), "{wikitext}");
    assert!(!wikitext.contains("intermediate.wikidot.com"), "{wikitext}");
    assert!(!wikitext.contains("default.png"), "{wikitext}");
    assert!(!wikitext.contains("default-full.png"), "{wikitext}");
    assert!(
        wikitext.contains("\n\"quoted ]] | caption\"\n[[/div]]\n[[/div]]\nafter"),
        "{wikitext}",
    );
}

#[test]
fn image_block_prepass_decodes_all_forwarded_arguments_before_semantics() {
    let variables = [
        (Cow::Borrowed("asset"), Cow::Borrowed("source image.png")),
        (Cow::Borrowed("attribute"), Cow::Borrowed("alt")),
        (
            Cow::Borrowed("description"),
            Cow::Borrowed(r#"A "quoted" label"#),
        ),
        (Cow::Borrowed("caption"), Cow::Borrowed("Forwarded caption")),
        (Cow::Borrowed("width"), Cow::Borrowed("225px")),
        (Cow::Borrowed("align"), Cow::Borrowed("left")),
    ]
    .into_iter()
    .collect::<VariableMap<'_>>();
    let owner = AttachmentOwner {
        site_slug: "source-site".into(),
        page_slug: "fragment:source".into(),
    };
    let owners = variables
        .keys()
        .map(|name| (name.to_string(), owner.clone()))
        .collect::<AttachmentVariableOwners>();
    let mut provenance = AttachmentProvenanceRegistry::default();
    let mut wikitext = concat!(
        "[[include component:image-block name={$asset}|alt={$attribute}|",
        "alt-text={$description}|caption={$caption}|width={$width}|align={$align}]]",
    )
    .to_owned();
    protect_forwarded_attachment_variables(
        &mut wikitext,
        &variables,
        &owners,
        &mut provenance,
    );

    let page_info = fallback_test_page_info("consumer", "Consumer");
    RenderService::expand_wikidot_image_block_includes_with_provenance(
        &mut wikitext,
        &page_info,
        Some(("intermediate", "component:image-block")),
        Some(&provenance),
    );

    assert!(
        wikitext.contains(concat!(
            "https://source-site.wikidot.com/local--files/",
            "fragment:source/source%20image.png",
        )),
        "{wikitext}",
    );
    assert!(wikitext.contains("block-left"), "{wikitext}");
    assert!(wikitext.contains("width:225px"), "{wikitext}");
    assert!(
        wikitext.contains(r#" alt="A &quot;quoted&quot; label""#),
        "{wikitext}",
    );
    assert!(wikitext.contains("Forwarded caption"), "{wikitext}");
    assert!(!wikitext.contains("__wj_attachment_"), "{wikitext}");
}

#[test]
fn image_block_prepass_decodes_forwarded_values_before_required_and_duplicate_checks() {
    let page_info = fallback_test_page_info("consumer", "Consumer");
    let owner = AttachmentOwner {
        site_slug: "source-site".into(),
        page_slug: "fragment:source".into(),
    };

    let empty_variables = [(Cow::Borrowed("empty"), Cow::Borrowed(""))]
        .into_iter()
        .collect::<VariableMap<'_>>();
    let empty_owners = [("empty".to_owned(), owner.clone())].into_iter().collect();
    let mut empty_provenance = AttachmentProvenanceRegistry::default();
    let mut empty_name = "[[include component:image-block name={$empty}]]".to_owned();
    protect_forwarded_attachment_variables(
        &mut empty_name,
        &empty_variables,
        &empty_owners,
        &mut empty_provenance,
    );
    let included_pages =
        RenderService::expand_wikidot_image_block_includes_with_provenance(
            &mut empty_name,
            &page_info,
            Some(("intermediate", "component:image-block")),
            Some(&empty_provenance),
        );
    assert!(included_pages.is_empty());
    assert!(empty_name.contains("__wj_attachment_"), "{empty_name}");
    empty_provenance.restore_unresolved(&mut empty_name);
    assert_eq!(empty_name, "[[include component:image-block name=]]",);

    let self_ref_variables = [(Cow::Borrowed("forwarded"), Cow::Borrowed("{$NAME}"))]
        .into_iter()
        .collect::<VariableMap<'_>>();
    let self_ref_owners = [("forwarded".to_owned(), owner)].into_iter().collect();
    let mut self_ref_provenance = AttachmentProvenanceRegistry::default();
    let mut self_ref = concat!(
        "[[include component:image-block ",
        "NAME={$forwarded}|name=default image.png]]",
    )
    .to_owned();
    protect_forwarded_attachment_variables(
        &mut self_ref,
        &self_ref_variables,
        &self_ref_owners,
        &mut self_ref_provenance,
    );
    RenderService::expand_wikidot_image_block_includes_with_provenance(
        &mut self_ref,
        &page_info,
        Some(("scp-wiki", "component:wrapper")),
        Some(&self_ref_provenance),
    );
    assert!(
        self_ref.contains(concat!(
            "http://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/default%20image.png",
        )),
        "{self_ref}",
    );
    assert!(!self_ref.contains("__wj_attachment_"), "{self_ref}");
    assert!(!self_ref.contains("{$NAME}"), "{self_ref}");
}

#[test]
fn image_block_prepass_uses_later_defaults_only_for_self_references() {
    let mut wikitext = concat!(
        "[[include[!-- opening gap --] component:image-block ",
        "[!-- argument gap [x] | still comment --] NAME [!-- key gap --] = ",
        "[!-- value gap --] {$NAME}|NAME=default image.png|",
        "LINK [!-- link gap --] = {$LINK}|LINK=default full.png]]",
    )
    .to_owned();
    let page_info = fallback_test_page_info("consumer", "Consumer");

    RenderService::expand_wikidot_image_block_includes_with_provenance(
        &mut wikitext,
        &page_info,
        Some(("scp-wiki", "component:wrapper")),
        Some(&AttachmentProvenanceRegistry::default()),
    );

    assert!(
        wikitext.contains(concat!(
            "http://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/default%20image.png",
        )),
        "{wikitext}",
    );
    assert!(
        wikitext.contains(concat!(
            "link=https://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/default%20full.png",
        )),
        "{wikitext}",
    );
    assert!(!wikitext.contains("{$NAME}"), "{wikitext}");
    assert!(!wikitext.contains("{$LINK}"), "{wikitext}");
}

#[test]
fn image_block_prepass_rejects_malformed_argument_segments() {
    let page_info = fallback_test_page_info("consumer", "Consumer");

    for source in [
        "[[include component:image-block name=foo.png|link]]",
        "[[include component:image-block name=foo.png|bad segment]]",
        "[[include component:image-block name=foo.png|[!-- unclosed]]",
    ] {
        let mut wikitext = source.to_owned();
        let included_pages = RenderService::expand_wikidot_image_block_includes(
            &mut wikitext,
            &page_info,
            None,
        );

        assert_eq!(wikitext, source);
        assert!(included_pages.is_empty(), "{source}");
    }

    let mut wikitext = concat!(
        "[[include component:image-block ||",
        "[!-- comment [x] | inside --]| name=foo.png | ]]",
    )
    .to_owned();
    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(
        wikitext.contains("/local--files/consumer/foo.png"),
        "{wikitext}",
    );
    assert_eq!(
        included_pages,
        vec![
            PageRef::page_only("component:image-block"),
            PageRef::page_only("component:image-block-base"),
        ],
    );
}

#[test]
fn image_block_prepass_unquotes_composite_names_for_the_authoring_page() {
    let mut wikitext = concat!(
        "[[include component:image-block name=\"thumb-real image.png\"|",
        "link=\"full-real image.png\"]]",
    )
    .to_owned();
    let page_info = fallback_test_page_info("consumer", "Consumer");

    RenderService::expand_wikidot_image_block_includes_with_provenance(
        &mut wikitext,
        &page_info,
        Some(("scp-wiki", "component:wrapper")),
        Some(&AttachmentProvenanceRegistry::default()),
    );

    assert!(
        wikitext.contains(
            "http://scp-wiki.wikidot.com/local--files/component:wrapper/thumb-real%20image.png",
        ),
        "{wikitext}",
    );
    assert!(!wikitext.contains("%22"), "{wikitext}");
    assert!(
        wikitext.contains(concat!(
            "link=\"https://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/full-real%20image.png\"",
        )),
        "{wikitext}",
    );

    ftml::preprocess_for_layout(&mut wikitext, Layout::Wikidot);
    let tokens = ftml::tokenize(&wikitext);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let html = HtmlRender.render(&tree, &page_info, &settings).body;
    assert!(
        html.contains(concat!(
            "src=\"http://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/thumb-real%20image.png\"",
        )),
        "{html}",
    );
    assert!(
        html.contains(concat!(
            "href=\"https://scp-wiki.wikidot.com/local--files/",
            "component:wrapper/full-real%20image.png\"",
        )),
        "{html}",
    );
}

#[test]
fn expands_image_block_caption_with_external_link_without_stealing_its_bracket() {
    // Reduced from EN:ralliston-s-authorpage. The final external-link `]`
    // adjacent to the include `]]` must not be treated as the include end.
    let mut wikitext = concat!(
        "[[include component:image-block ",
        "name=linked.jpg|caption=[https://example.com/path Linked label] by ",
        "[[*user Example User]]]]\n",
    )
    .to_owned();
    let page_info = fallback_test_page_info("author-page", "Author Page");

    RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info, None);

    assert!(wikitext.contains("[https://example.com/path Linked label]"));
    assert!(wikitext.contains("[[*user Example User]]"));
    assert!(wikitext.ends_with("[[/div]]\n"), "{wikitext}");
    assert!(!wikitext.contains("[[/div]]]"), "{wikitext}");
}

#[test]
fn expands_included_fragment_wikidot_image_blocks_before_generic_includes() {
    let page_info = fallback_test_page_info("scp-8382", "SCP-8382");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragment_wikitext = concat!(
        "Before the block.\n",
        "[[include component:image-block\n",
        "  name=Alis.jpg|\n",
        "  caption=Photograph of PoI-6721-1 at Secure Area-219, 2023. |\n",
        "  width=225px|\n",
        "  align=left|\n",
        "]]\n",
        "After the block.\n",
    )
    .to_owned();

    let image_block_includes = RenderService::expand_wikidot_image_block_includes(
        &mut fragment_wikitext,
        &page_info,
        Some(("scp-wiki", "fragment:scp-8382-2")),
    );
    let top_wikitext = "[[include fragment:scp-8382-2]]\n";
    let (mut expanded, direct_included_pages) = ftml::include(
        top_wikitext,
        &settings,
        PreparedIncluder {
            pages: vec![Some(fragment_wikitext)],
            missing_replacements: Default::default(),
        },
        include_error,
    )
    .expect("prepared include should expand");

    ftml::preprocess_for_layout(&mut expanded, settings.layout);
    let tokens = ftml::tokenize(&expanded);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert_eq!(
        direct_included_pages,
        vec![PageRef::page_only("fragment:scp-8382-2")]
    );
    assert_eq!(
        image_block_includes,
        vec![
            PageRef::page_only("component:image-block"),
            PageRef::page_only("component:image-block-base"),
        ]
    );
    assert!(rendered.contains(
        r#"<img src="http://scp-wiki.wikidot.com/local--files/fragment:scp-8382-2/Alis.jpg""#
    ));
    assert!(rendered.contains("Photograph of PoI-6721-1 at Secure Area-219, 2023."));
    assert!(!rendered.contains("[[image"));
    assert!(!rendered.contains("{$alt}"));
    assert!(!rendered.contains("link=#"));
}

#[test]
fn leaves_image_block_includes_on_non_scp_wiki_sites_for_normal_expansion() {
    let mut wikitext = concat!(
        "[[include component:image-block name=custom.jpg|caption=Custom block.]]\n",
        "[[include :sandbox-for-codex:component:image-block name=custom.jpg]]\n",
    )
    .to_owned();
    let page_info = ftml::data::PageInfo {
        site: Cow::Borrowed("sandbox-for-codex"),
        page: Cow::Borrowed("start"),
        title: Cow::Borrowed("Sandbox"),
        alt_title: None,
        tags: Vec::new(),
        category: None,
        score: ftml::data::ScoreValue::Integer(0),
        language: Cow::Borrowed("en"),
    };

    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(included_pages.is_empty());
    assert!(wikitext.contains("[[include component:image-block name=custom.jpg"));
    assert!(wikitext.contains("[[include :sandbox-for-codex:component:image-block"));
}

#[test]
fn leaves_literal_wikidot_image_block_includes_unexpanded() {
    let mut wikitext = concat!(
        "[[code]]\n",
        "[[include component:image-block name=code.jpg]]\n",
        "[[/code]]\n",
        "@@[[include component:image-block name=escaped.jpg]]@@\n",
        "[!-- [[include component:image-block name=comment.jpg]] --]\n",
        "[[include component:image-block name=live.jpg]]\n",
    )
    .to_owned();
    let page_info = fallback_test_page_info("scp-3922", "SCP-3922");

    let included_pages = RenderService::expand_wikidot_image_block_includes(
        &mut wikitext,
        &page_info,
        None,
    );

    assert!(wikitext.contains("[[include component:image-block name=code.jpg]]"));
    assert!(wikitext.contains("[[include component:image-block name=escaped.jpg]]"));
    assert!(wikitext.contains("[[include component:image-block name=comment.jpg]]"));
    assert!(wikitext.contains(
        "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/live.jpg]]"
    ));
    assert_eq!(
        included_pages,
        vec![
            PageRef::page_only("component:image-block"),
            PageRef::page_only("component:image-block-base"),
        ],
    );
}

#[test]
fn skips_generic_includes_inside_wikidot_comments() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[!--\n",
        "Usage:\n",
        "[[include :scp-wiki:component:interwiki-style\n",
        "| priority=1\n",
        "]]\n",
        "--]\n",
        "[[include component:live]]\n",
    )
    .to_owned();

    RenderService::mask_wikidot_literal_include_markers(&mut wikitext);
    let mut includes = Vec::new();
    ftml::include(
        &wikitext,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("include collection should skip comment-hidden usage examples");
    RenderService::unmask_wikidot_literal_include_markers(&mut wikitext);

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].page_ref(),
        &PageRef::page_only("component:live")
    );
    assert!(wikitext.contains("[[include :scp-wiki:component:interwiki-style"));
}

#[test]
fn skips_generic_includes_inside_wikidot_literal_regions() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[[code]]\n",
        "[[include component:code-example]]\n",
        "[[/code]]\n",
        "[[span class=\"code-cell\"]] @@ [[include component:raw-example]] @@ [[/span]]\n",
        "[!-- [[include component:comment-example]] --]\n",
        "[[include component:live]]\n",
    )
    .to_owned();

    RenderService::mask_wikidot_literal_include_markers(&mut wikitext);
    let mut includes = Vec::new();
    ftml::include(
        &wikitext,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("include collection should skip literal usage examples");
    RenderService::unmask_wikidot_literal_include_markers(&mut wikitext);

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].page_ref(),
        &PageRef::page_only("component:live")
    );
    assert!(wikitext.contains("[[include component:code-example]]"));
    assert!(wikitext.contains("[[include component:raw-example]]"));
    assert!(wikitext.contains("[[include component:comment-example]]"));
}

#[test]
fn strips_included_comment_usage_examples_after_expansion() {
    let page_info = fallback_test_page_info("scp-anthology-2024", "SCP Anthology 2024");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut component_source = concat!(
        "[!--\n",
        "Usage:\n",
        "[[include :scp-wiki:component:interwiki-style\n",
        "| priority=1\n",
        "]]\n",
        "--]\n",
        "[[embed]]\n",
        "<iframe src=\"/-/wikidot-interwiki/styleFrame.html?priority=1\"></iframe>\n",
        "[[/embed]]\n",
    )
    .to_owned();

    RenderService::mask_wikidot_literal_include_markers(&mut component_source);
    let mut nested_includes = Vec::new();
    ftml::include(
        &component_source,
        &settings,
        CollectingIncluder {
            includes: &mut nested_includes,
        },
        include_error,
    )
    .expect("comment usage examples should not request nested pages");
    RenderService::unmask_wikidot_literal_include_markers(&mut component_source);

    let (mut expanded, included_pages) = ftml::include(
        "[[include :scp-wiki:component:interwiki-style]]\n",
        &settings,
        PreparedIncluder {
            pages: vec![Some(component_source)],
            missing_replacements: Default::default(),
        },
        include_error,
    )
    .expect("prepared include should expand the component source");
    ftml::preprocess_for_layout(&mut expanded, settings.layout);
    let tokens = ftml::tokenize(&expanded);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(nested_includes.is_empty());
    assert_eq!(
        included_pages,
        vec![PageRef::page_and_site(
            "scp-wiki",
            "component:interwiki-style"
        )]
    );
    assert!(rendered.contains("styleFrame.html?priority=1"));
    assert!(!rendered.contains("Usage:"));
    assert!(!rendered.contains("[[include :scp-wiki:component:interwiki-style"));
}

#[test]
fn keeps_selected_comment_branch_includes_collectable() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[!-- --]\n",
        "[[include component:selected]]\n",
        "[!----]\n",
        "[!-- {$inc-hidden}\n",
        "[[include component:hidden]]\n",
        "[!----]\n",
    )
    .to_owned();

    RenderService::mask_wikidot_literal_include_markers(&mut wikitext);
    let mut includes = Vec::new();
    ftml::include(
        &wikitext,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("include collection should keep selected branch includes");
    RenderService::unmask_wikidot_literal_include_markers(&mut wikitext);

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].page_ref(),
        &PageRef::page_only("component:selected")
    );
    assert!(wikitext.contains("[[include component:hidden]]"));
}

#[test]
fn collects_single_line_wikidot_include_variables() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut includes = Vec::new();
    ftml::include(
        "[[include :scp-wiki:theme:basalt | wide=a | hidetitle=a]]\n",
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("include collection should parse Wikidot single-line variables");

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].page_ref(),
        &PageRef::page_and_site("scp-wiki", "theme:basalt"),
    );
    assert_eq!(
        includes[0].variables().get("wide").map(Cow::as_ref),
        Some("a "),
    );
    assert_eq!(
        includes[0].variables().get("hidetitle").map(Cow::as_ref),
        Some("a"),
    );

    let mut multiline_includes = Vec::new();
    let mut multiline_include = concat!(
        "[[include :scp-jp:user-component:ta-badge\n",
        "|badge-top=gold-medal\n",
        "|badge-right=photographer\n",
        "|badge-left=high-calibre\n",
        "|frame=\n",
        "|bg-img=background-color: #fff\n",
        "|bg-shadow=false\n",
        "|plate=style2\n",
        "]]\n",
    )
    .to_owned();
    RenderService::normalize_wikidot_multiline_includes(&mut multiline_include);

    ftml::include(
        &multiline_include,
        &settings,
        CollectingIncluder {
            includes: &mut multiline_includes,
        },
        include_error,
    )
    .expect("include collection should parse Wikidot multiline variables");

    assert_eq!(multiline_includes.len(), 1);
    assert_eq!(
        multiline_includes[0].page_ref(),
        &PageRef::page_and_site("scp-jp", "user-component:ta-badge"),
    );
    assert_eq!(
        multiline_includes[0]
            .variables()
            .get("bg-img")
            .map(Cow::as_ref),
        Some("background-color: #fff "),
    );
}

#[test]
fn normalizes_generic_wikidot_multiline_include_arguments() {
    let mut include = concat!(
        "[[include component:generic\n",
        "|first=one\n",
        "|second=two\n",
        "]]\n",
    )
    .to_owned();

    RenderService::normalize_wikidot_multiline_includes(&mut include);

    assert_eq!(
        include,
        "[[include component:generic |first=one |second=two]]\n"
    );
}

#[test]
fn multiline_include_variables_preserve_a_self_referential_class_prefix() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut include = concat!(
        "[[include :scp-wiki:component:earthworm\n",
        "| previous-url = scp-8380\n",
        "| previous-title = Aquaphobia\n",
        "| next-url = scp-8894\n",
        "| next-title = Basophobia\n",
        "| hub-url = scp-anthology-2024\n",
        "| hub-title = Anthology 2024\n",
        "| class = earthworm--anth24 {$class}\n",
        "]]\n",
    )
    .to_owned();
    RenderService::normalize_wikidot_multiline_includes(&mut include);

    let mut includes = Vec::new();
    ftml::include(
        &include,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("multiline include should parse");

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].page_ref(),
        &PageRef::page_and_site("scp-wiki", "component:earthworm"),
    );
    assert_eq!(
        includes[0].variables().get("class").map(Cow::as_ref),
        Some("earthworm--anth24 {$class}"),
    );

    let mut source = r#"class="{$class}""#.to_owned();
    super::super::apply_include_variables(&mut source, &includes[0])
        .expect("self-referential class should remain within the byte budget");
    assert_eq!(source, r#"class="earthworm--anth24 {$class}""#);
}

#[test]
fn leaves_malformed_multiline_include_boundaries_untouched() {
    let mut include = concat!(
        "[[include component:generic\n",
        "not-an-argument\n",
        "]]\n",
        "[[include component:generic\n",
        "first=one|\n",
        "]]\n",
        "[[include component:image-block\n",
        "name=still-valid|\n",
        "not-an-argument|\n",
        "]]\n",
        "[[includex component:not-an-include\n",
        "|first=one\n",
        "]]\n",
    )
    .to_owned();
    let original = include.clone();

    RenderService::normalize_wikidot_multiline_includes(&mut include);

    assert_eq!(include, original);
}

#[test]
fn resolves_nested_wikidot_include_variables() {
    let include = IncludeRef::new(
        PageRef::page_and_site("scp-jp", "user-component:ta-badge-smooth-base-base"),
        VariableMap::from([
            (Cow::Borrowed("name"), Cow::Borrowed("action")),
            (Cow::Borrowed("action"), Cow::Borrowed("true")),
            (Cow::Borrowed("type"), Cow::Borrowed("false")),
        ]),
    );
    let mut source =
        r#"[[div_ class="badges badge-{$name} {$name} a{${$name}} b{$type}"]]"#
            .to_owned();

    super::super::apply_include_variables(&mut source, &include)
        .expect("nested variables should remain within the byte budget");

    assert!(source.contains(r#"class="badges badge-action action atrue bfalse""#));
    assert!(!source.contains("{$action}"));
}

#[test]
fn nested_include_argument_self_reference_uses_later_fallback() {
    let outer = IncludeRef::new(
        PageRef::page_and_site("scp-jp", "component:coltop"),
        VariableMap::from([
            (Cow::Borrowed("show"), Cow::Borrowed("+ New Messages: 1")),
            (Cow::Borrowed("nohide"), Cow::Borrowed("true")),
        ]),
    );
    let mut source = concat!(
        "[[include :scp-jp:component:coltop-deep\n",
        "ifprot={$ifprot}\n",
        "|ifprot=0\n",
        "|nohide={$nohide}\n",
        "|nohide=0\n",
        "|folded={$folded}\n",
        "|folded=1\n",
        "|show={$show}\n",
        "|show=+ show block\n",
        "]]",
    )
    .to_owned();
    super::super::apply_include_variables(&mut source, &outer)
        .expect("nested include arguments should remain within the byte budget");

    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut includes = Vec::new();
    ftml::include(
        &source,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("nested include fallback should parse");

    assert_eq!(includes.len(), 1, "{source}");
    let variables = includes[0].variables();
    assert_eq!(
        variables.get("ifprot").map(|value| value.trim_end()),
        Some("0"),
    );
    assert_eq!(
        variables.get("nohide").map(|value| value.trim_end()),
        Some("true"),
    );
    assert_eq!(
        variables.get("folded").map(|value| value.trim_end()),
        Some("1"),
    );
    assert_eq!(
        variables.get("show").map(|value| value.trim_end()),
        Some("+ New Messages: 1"),
    );
}

#[test]
fn nested_cross_site_includes_keep_the_original_lookup_site() {
    let page_info = fallback_test_page_info("consumer", "Consumer");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let context = IncludeExpansionContext {
        current_site_id: 11,
        current_site_slug: "sandbox-for-codex".to_owned(),
        attachment_owner: None,
        page_info: &page_info,
        settings: &settings,
        page_preview: false,
        expand_wikidot_image_blocks: true,
        max_total_includes: 100,
        render_cost_budget: RenderCostBudget::new_default(),
    };
    let source = IncludeSource {
        site_slug: "scp-wiki".to_owned(),
        page_slug: "main".to_owned(),
        wikitext: "[[include fragment:front-page-features]]".to_owned(),
    };

    let nested = context.for_nested_source(&source);

    assert_eq!(nested.current_site_id, 11);
    assert_eq!(nested.current_site_slug, "sandbox-for-codex");
    assert_eq!(
        nested.attachment_owner,
        Some(AttachmentOwner {
            site_slug: "scp-wiki".to_owned(),
            page_slug: "main".to_owned(),
        }),
    );
}

#[test]
fn include_variable_rebuild_handles_adjacent_growth_shrink_and_same_length() {
    let include = IncludeRef::new(
        PageRef::page_only("component:test"),
        VariableMap::from([
            (Cow::Borrowed("grow"), Cow::Borrowed("expanded")),
            (Cow::Borrowed("shrink"), Cow::Borrowed("")),
            (Cow::Borrowed("same"), Cow::Borrowed("same123")),
        ]),
    );
    let mut source = "before{$grow}{$shrink}{$same}after".to_owned();

    super::super::apply_include_variables(&mut source, &include)
        .expect("adjacent variables should remain within the byte budget");

    assert_eq!(source, "beforeexpandedsame123after");
}

#[test]
fn include_variable_rebuild_preserves_unresolved_and_self_references_with_defaults() {
    let include = IncludeRef::new(
        PageRef::page_only("component:test"),
        VariableMap::from([
            (Cow::Borrowed("self"), Cow::Borrowed("{$self}")),
            (Cow::Borrowed("trimmed"), Cow::Borrowed("value \t\r\n")),
        ]),
    );
    let mut source = "{$author}|{$missing}|{$shadow}|{$self}|{$trimmed}".to_owned();

    super::super::apply_include_variables(&mut source, &include)
        .expect("unresolved and self-referential variables should remain bounded");

    assert_eq!(source, "%%created_by%%|{$missing}|no|{$self}|value");
}

#[test]
fn include_variable_rebuild_does_not_repeat_a_self_referential_prefix() {
    let include = IncludeRef::new(
        PageRef::page_only("component:earthworm"),
        VariableMap::from([(
            Cow::Borrowed("class"),
            Cow::Borrowed("earthworm--anth24 {$class}"),
        )]),
    );
    let mut source = r#"class="{$class}""#.to_owned();

    super::super::apply_include_variables(&mut source, &include)
        .expect("self-referential prefix should remain bounded");

    assert_eq!(source, r#"class="earthworm--anth24 {$class}""#);
}

#[test]
fn include_variable_rebuild_stops_at_the_existing_depth_limit() {
    let variables = (0..=super::super::MAX_INCLUDE_EXPANSION_DEPTH)
        .map(|depth| {
            (
                Cow::Owned(format!("v{depth}")),
                Cow::Owned(format!("{{$v{}}}", depth + 1)),
            )
        })
        .collect();
    let include = IncludeRef::new(PageRef::page_only("component:test"), variables);
    let mut source = "{$v0}".to_owned();

    super::super::apply_include_variables(&mut source, &include)
        .expect("the existing variable pass limit should remain bounded");

    assert_eq!(
        source,
        format!("{{$v{}}}", super::super::MAX_INCLUDE_EXPANSION_DEPTH),
    );
}

#[test]
fn include_variable_rebuild_matches_reverse_replacement_output() {
    fn apply_reverse_replacement_reference(
        content: &mut String,
        include: &IncludeRef<'_>,
    ) {
        for _ in 0..super::super::MAX_INCLUDE_EXPANSION_DEPTH {
            let mut matches = Vec::new();

            for capture in super::super::INCLUDE_VARIABLE_REGEX.captures_iter(content) {
                let mtch = capture.get(0).unwrap();
                let name = &capture["name"];
                if let Some(value) = include
                    .variables()
                    .get(name)
                    .map(|value| {
                        super::super::trim_include_variable_value(value).to_owned()
                    })
                    .or_else(|| super::super::default_include_variable_value(name))
                {
                    let changed = value != mtch.as_str();
                    matches.push((value, mtch.range(), changed));
                }
            }

            if matches.is_empty() {
                break;
            }

            let changed = matches.iter().any(|(_, _, changed)| *changed);
            matches.reverse();
            for (value, range, _) in matches {
                content.replace_range(range, &value);
            }
            if !changed {
                break;
            }
        }
    }

    let cases = [
        (
            "A{$grow}{$shrink}{$same}Z",
            vec![("grow", "expanded"), ("shrink", "x"), ("same", "same123")],
        ),
        (
            "{$missing}|{$author}|{$shadow}|{$self}",
            vec![("self", "{$self}")],
        ),
        (
            "{$outer}",
            vec![("outer", "pre{$inner}post"), ("inner", "done")],
        ),
        (
            "{$left}{$right}",
            vec![("left", "{$right}"), ("right", "{$left}")],
        ),
    ];

    for (source, variables) in cases {
        let include = IncludeRef::new(
            PageRef::page_only("component:test"),
            variables
                .into_iter()
                .map(|(name, value)| {
                    (Cow::Owned(name.to_owned()), Cow::Owned(value.to_owned()))
                })
                .collect(),
        );
        let mut expected = source.to_owned();
        let mut actual = source.to_owned();

        apply_reverse_replacement_reference(&mut expected, &include);
        super::super::apply_include_variables(&mut actual, &include)
            .expect("reference cases should remain within the byte budget");

        assert_eq!(actual, expected, "source: {source}");
    }
}

#[test]
fn included_dynamic_iftags_substitutes_directive_and_spec_before_matching() {
    let source = "[[ift{$mode}gs +{$required_tag}]]selected[[/ift{$mode}gs]]";

    assert_eq!(
        resolve_test_included_variable_iftags(
            source,
            &[("mode", "a"), ("required_tag", "theme")],
            &["theme"],
        ),
        "selected",
    );
}

#[test]
fn included_dynamic_iftags_drops_body_when_substituted_spec_does_not_match() {
    let source = "[[ift{$mode}gs +{$required_tag}]]selected[[/ift{$mode}gs]]";

    assert_eq!(
        resolve_test_included_variable_iftags(
            source,
            &[("mode", "a"), ("required_tag", "theme")],
            &["other"],
        ),
        "",
    );
    assert_eq!(
        resolve_test_included_variable_iftags(source, &[("mode", "a")], &["theme"],),
        "",
    );
}

#[test]
fn included_dynamic_iftags_keeps_absent_mode_transparent() {
    let source = "before[[ift{$mode}gs +{$required_tag}]]selected[[/ift{$mode}gs]]after";

    assert_eq!(
        resolve_test_included_variable_iftags(source, &[("required_tag", "theme")], &[],),
        "beforeselectedafter",
    );
}

#[test]
fn nested_include_preparation_preserves_callsite_dynamic_iftags_outcomes() {
    let source = "before[[ift{$mode}gs +theme]]selected[[/ift{$mode}gs]]after";

    assert_eq!(
        prepare_test_nested_include_conditionals(source, &[("mode", "a")], &["theme"]),
        "beforeselectedafter",
    );
    assert_eq!(
        prepare_test_nested_include_conditionals(source, &[], &[]),
        "beforeselectedafter",
    );
    assert_eq!(
        prepare_test_nested_include_conditionals(source, &[("mode", "other")], &[]),
        "before[[iftothergs +theme]]selected[[/iftothergs]]after",
    );

    let malformed = "before[[ift{$mode}gs +theme]]selected[[/ift{$other}gs]]after";
    assert_eq!(
        prepare_test_nested_include_conditionals(malformed, &[], &[]),
        malformed,
    );
}

#[test]
fn nested_include_preparation_prunes_comment_branches_before_iftags() {
    // Reduced from EN:component:blacklight-box-source. The inactive
    // branches must be removed while each include invocation is still an
    // independent source; otherwise repeated fragments can leave their
    // outer gates to cross-pair after textual assembly.
    let source = concat!(
        "[[iftags -component-backend]]\n",
        "[!-- {$inc-source}\n",
        "[[module css]]source[[/module]]\n",
        "[!----]\n",
        "[!-- {$inc-colors}\n",
        "[[module css]]colors[[/module]]\n",
        "[!----]\n",
        "[!-- {$inc-section-start}\n",
        "[[div class=\"section\"]]\n",
        "[!----]\n",
        "[!-- {$inc-section-end}\n",
        "[[/div]]\n",
        "[!----]\n",
        "[[/iftags]]\n",
        "[[iftags +component-backend]]documentation[[/iftags]]\n",
    );

    let source_fragment =
        prepare_test_nested_include_conditionals(source, &[("inc-source", "--]")], &[]);
    let start_fragment = prepare_test_nested_include_conditionals(
        source,
        &[("inc-section-start", "--]")],
        &[],
    );
    let end_fragment = prepare_test_nested_include_conditionals(
        source,
        &[("inc-section-end", "--]")],
        &[],
    );
    let assembled = format!("{source_fragment}{start_fragment}body\n{end_fragment}");

    assert!(source_fragment.contains("source"), "{source_fragment}");
    assert!(start_fragment.contains("[[div class=\"section\"]]"));
    assert!(end_fragment.contains("[[/div]]"));
    assert!(!assembled.contains("colors"), "{assembled}");
    assert!(!assembled.contains("documentation"), "{assembled}");
    assert!(!assembled.contains("{$"), "{assembled}");
    assert!(!assembled.contains("[!--"), "{assembled}");
    assert!(!assembled.contains("[[iftags"), "{assembled}");
    assert!(!assembled.contains("[[/iftags]]"), "{assembled}");
}

#[test]
fn include_source_preparation_preserves_unbounded_malformed_comment_branch() {
    let original = concat!(
        "before\n",
        "[!-- {$inc-section-end}\n",
        "[[/div]]\n",
        "after\n",
    );
    let include =
        IncludeRef::new(PageRef::page_only("component:test"), VariableMap::new());
    let page_info = fallback_test_page_info("consumer", "Consumer");
    let mut source = original.to_owned();
    let mut compat_text = CompatTextFragments::new(original);

    super::super::prepare_include_source_variables_and_comment_branches(
        &mut source,
        &include,
        &page_info,
        &mut compat_text,
    )
    .expect("malformed branch source should remain within the byte budget");

    assert_ne!(source, original);
    assert!(!source.contains("[!-- {$inc-section-end"), "{source}");
    assert_eq!(compat_text.restore(&source), original);
}

#[test]
fn malformed_include_comment_branch_cannot_claim_sibling_boundary() {
    let page_info = fallback_test_page_info("consumer", "Consumer");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let malformed = concat!(
        "CHILD_BEFORE\n",
        "[!-- {$missing}\n",
        "CHILD_HIDDEN\n",
        "[[include component:must-not-expand]]\n",
    );
    let include = IncludeRef::new(
        PageRef::page_only("component:malformed"),
        VariableMap::new(),
    );
    let mut compat_text = CompatTextFragments::new(malformed);
    let mut malformed = malformed.to_owned();
    super::super::prepare_include_source_variables_and_comment_branches(
        &mut malformed,
        &include,
        &page_info,
        &mut compat_text,
    )
    .expect("malformed branch source should remain within the byte budget");
    let start = prepare_test_nested_include_conditionals(
        concat!(
            "[[iftags -component-backend]]\n",
            "[!-- {$start}\n",
            "[[div class=\"selected-sibling\"]]\n",
            "[!----]\n",
            "[!-- {$end}\n",
            "[[/div]]\n",
            "[!----]\n",
            "[[/iftags]]\n",
        ),
        &[("start", "--]")],
        &[],
    );
    let end = prepare_test_nested_include_conditionals(
        concat!(
            "[[iftags -component-backend]]\n",
            "[!-- {$start}\n",
            "[[div class=\"selected-sibling\"]]\n",
            "[!----]\n",
            "[!-- {$end}\n",
            "[[/div]]\n",
            "[!----]\n",
            "[[/iftags]]\n",
        ),
        &[("end", "--]")],
        &[],
    );
    let caller = concat!(
        "[[include component:start]]\n",
        "[[include component:malformed]]\n",
        "[[include component:end]]\n",
        "ROOT_BEFORE\n",
        "[!----]\n",
        "ROOT_AFTER\n",
    );
    let (mut expanded, _) = ftml::include(
        caller,
        &settings,
        PreparedIncluder {
            pages: vec![Some(start), Some(malformed), Some(end)],
            missing_replacements: Default::default(),
        },
        include_error,
    )
    .expect("prepared sibling includes should expand");

    super::super::remove_unresolved_include_comment_branches(&mut expanded);
    assert!(expanded.contains("CHILD_BEFORE\n"), "{expanded}");
    assert!(!expanded.contains("CHILD_HIDDEN"), "{expanded}");
    assert!(!expanded.contains("must-not-expand"), "{expanded}");
    assert!(expanded.contains("ROOT_BEFORE\n"), "{expanded}");
    assert!(expanded.contains("ROOT_AFTER\n"), "{expanded}");
    assert!(expanded.contains("[[div class=\"selected-sibling\"]]"));
    assert!(expanded.contains("[[/div]]"));

    ftml::preprocess_for_layout(&mut expanded, settings.layout);
    let tokens = ftml::tokenize(&expanded);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, errors) = result.into();
    assert!(errors.is_empty(), "{errors:?}\n{expanded}");
    let html = HtmlRender.render(&tree, &page_info, &settings).body;
    let html = compat_text.restore(&html);

    assert!(html.contains("[!-- {$missing}"), "{html}");
    assert!(html.contains("CHILD_BEFORE"), "{html}");
    assert!(html.contains("CHILD_HIDDEN"), "{html}");
    assert!(html.contains("component:must-not-expand"), "{html}");
    assert!(html.contains("ROOT_BEFORE"), "{html}");
    assert!(html.contains("ROOT_AFTER"), "{html}");
    assert!(html.contains("selected-sibling"), "{html}");
}

#[test]
fn nested_include_preparation_skips_repeated_unbound_dynamic_iftags_resolution() {
    let page_info = fallback_test_page_info("consumer", "Consumer");
    let mut source =
        "before[[ift{$mode}gs +theme]]selected[[/ift{$mode}gs]]after".to_owned();
    let expected = source.clone();
    let mut preserved = CompatTextFragments::new(&source);

    RenderService::prepare_wikidot_conditionals_before_include_expansion(
        &mut source,
        &page_info,
        &mut preserved,
        1,
    );
    source = preserved.restore(&source);

    assert_eq!(source, expected);
}
