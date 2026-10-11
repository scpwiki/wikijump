/*
 * tests/media.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::ADMIN_USER_ID;
use deepwell::services::RequestContext;
use deepwell::services::view::GetPageViewOutput;
use deepwell::types::Reference;
use serde_json::json;
use std::borrow::Cow;

const NO_MATCH: &str =
    r#"<div class="error-block">Sorry, no match for the embedded content.</div>"#;

#[tokio::test]
async fn documented_embedding_code_probes_match_frozen_live_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let embed_alias = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "spec-syntax-embed-alias",
            "wikitext": "before\n[[embed]]\n<p id=\"spec-embed-probe\">hello</p>\n[[/embed]]\nafter",
        }),
    );
    assert!(embed_alias.body.starts_with("<p>before<br>"));
    assert!(embed_alias.body.contains(NO_MATCH));
    assert!(embed_alias.body.ends_with("<br>\nafter"));
    assert!(!embed_alias.body.contains("spec-embed-probe"));
    assert!(!embed_alias.body.contains("[[embed]]"));

    for (case_id, source, expected) in [
        (
            "spec-syntax-iframe-allowed",
            r#"[[iframe https://example.com/ frameborder="0" width="50%" height="120" scrolling="yes"]]"#,
            r#"<p><iframe src="https://example.com/" align="" frameborder="0" height="120" scrolling="yes" width="50%" class="" style=""></iframe></p>"#,
        ),
        (
            "spec-syntax-iframe-unknown-attribute",
            r#"[[iframe https://example.com/ width="50%" probe="1"]]"#,
            r#"<p><iframe src="https://example.com/" align="" frameborder="" height="" scrolling="" width="50%" class="" style=""></iframe></p>"#,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert_eq!(preview.body, expected, "{case_id}");
    }
}

#[tokio::test]
async fn documented_embedaudio_probe_matches_frozen_live_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site.site_id,
            "title": "spec-syntax-embedaudio-embed",
            "wikitext": concat!(
                "[[embedaudio]]\n",
                r#"<embed src="https://example.com/audio.swf" width="300" height="52">"#,
                "\n[[/embedaudio]]",
            ),
        }),
    );
    assert_eq!(preview.body, NO_MATCH);
}

#[tokio::test]
async fn arbitrary_embedvideo_host_remains_fail_closed_pending_issue_1042() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site.site_id,
            "title": "spec-syntax-embedvideo-iframe-security-divergence",
            "wikitext": concat!(
                "[[embedvideo]]\n",
                r#"<iframe src="https://example.com/" width="320" height="180"></iframe>"#,
                "\n[[/embedvideo]]",
            ),
        }),
    );

    // Frozen Wikidot PagePreview renders this arbitrary HTTPS iframe. Keep
    // Wikijump's provider allowlist fail-closed until issue #1042 adjudicates
    // that security divergence instead of silently widening iframe authority.
    assert_eq!(preview.body, NO_MATCH);
}

#[tokio::test]
async fn embedvideo_preview_resolves_only_allowlisted_typed_media() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source, expected) in [
        (
            "focused-embedvideo-corpus-3-theresacactusinthecorner",
            concat!(
                "[[embedvideo]]\n",
                r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/4sroHOHlkAk" frameborder="0" allowfullscreen></iframe>"#,
                "\n[[/embedvideo]]",
            ),
            r#"<p><iframe width="560" height="315" src="https://www.youtube.com/embed/4sroHOHlkAk" frameborder="0" allowfullscreen="allowfullscreen"></iframe></p>"#,
        ),
        (
            "focused-embedvideo-corpus-13-echoes-of-the-end-the-last-safe-haven-hub",
            concat!(
                "[[embedvideo]]\n",
                r#"<iframe src="https://embed.acast.com/66d847f49c7d98a7f877df3d?episode-order=asc&feed=true" frameBorder="0" width="100%" height="380px"></iframe>"#,
                "\n[[/embedvideo]]",
            ),
            r#"<p><iframe src="https://embed.acast.com/66d847f49c7d98a7f877df3d?episode-order=asc&amp;feed=true" frameborder="0" width="100%" height="380px"></iframe></p>"#,
        ),
        (
            "focused-embedvideo-corpus-10-the-trolley-solution-hub",
            concat!(
                "[[embedvideo]]\n",
                r#"<iframe src="https://embed.acast.com/624e90f06b1d87001240baa8?episode-order=desc" frameBorder="0" width="100%" height="80px"></iframe>"#,
                "\n[[/embedvideo]]",
            ),
            r#"<p><iframe src="https://embed.acast.com/624e90f06b1d87001240baa8?episode-order=desc" frameborder="0" width="100%" height="80px"></iframe></p>"#,
        ),
        (
            "focused-embedvideo-corpus-11-echoes-of-the-end-the-last-safe-haven-hub",
            concat!(
                "[[embedvideo]]\n",
                r#"<iframe src="https://embed.acast.com/66d847f49c7d98a7f877df3d/66d847f6202da66c8d8c8ca8" frameBorder="0" width="100%" height="80px"></iframe>"#,
                "\n[[/embedvideo]]",
            ),
            r#"<p><iframe src="https://embed.acast.com/66d847f49c7d98a7f877df3d/66d847f6202da66c8d8c8ca8" frameborder="0" width="100%" height="80px"></iframe></p>"#,
        ),
        (
            "focused-embedvideo-corpus-8-fam-radio-hub",
            concat!(
                "[[embedvideo]]\n",
                r#"<iframe src="https://embed.acast.com/620152ffa0b55c00129a3b8d" frameBorder="0" allow="autoplay" width="100%" height="110"></iframe>"#,
                "\n[[/embedvideo]]",
            ),
            r#"<p><iframe src="https://embed.acast.com/620152ffa0b55c00129a3b8d" frameborder="0" allow="autoplay" width="100%" height="110"></iframe></p>"#,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );

        assert_eq!(preview.body, expected, "{case_id}");
    }
}

#[tokio::test]
async fn embedvideo_preview_fails_closed_for_unsupported_media() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, payload) in [
        (
            "focused-embedvideo-basic",
            "https://www.youtube.com/embed/dQw4w9WgXcQ",
        ),
        (
            "focused-embedvideo-youtube-watch",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        ),
        (
            "focused-embedvideo-youtube-short",
            "https://youtu.be/dQw4w9WgXcQ",
        ),
        ("focused-embedvideo-javascript", "javascript:alert(1)"),
        (
            "focused-embedvideo-data",
            "data:text/html,<script>alert(1)</script>",
        ),
        (
            "focused-embedvideo-html-iframe",
            r#"<iframe src="https://example.com"></iframe>"#,
        ),
        (
            "open43-m-media-event-handler",
            r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen onload="alert(1)"></iframe>"#,
        ),
        (
            "open43-m-media-iframe-javascript",
            r#"<iframe width="560" height="315" src="javascript:alert(1)" frameborder="0" allowfullscreen></iframe>"#,
        ),
        (
            "open43-m-media-iframe-data",
            r#"<iframe width="560" height="315" src="data:text/html,unsafe" frameborder="0" allowfullscreen></iframe>"#,
        ),
        (
            "open43-m-media-iframe-credentials",
            r#"<iframe width="560" height="315" src="https://user@www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen></iframe>"#,
        ),
        (
            "open43-m-media-iframe-port",
            r#"<iframe width="560" height="315" src="https://www.youtube.com:444/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen></iframe>"#,
        ),
        (
            "open43-m-media-malformed",
            r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen>nested</iframe>"#,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": format!("[[embedvideo]]\n{payload}\n[[/embedvideo]]"),
            }),
        );

        assert_eq!(preview.body, NO_MATCH, "{case_id}");
        assert!(!preview.body.contains("<iframe"), "{case_id}");
    }
}

#[tokio::test]
async fn authored_embedvideo_marker_cannot_forge_a_typed_requirement() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let foreign = concat!(
        r#"@@<div class="wj-embed-video" "#,
        r#"id="wj-embed-video-ffffffffffffffffffffffffffffffff"></div>@@"#,
    );

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site.site_id,
            "title": "open43-m-media-foreign-marker",
            "wikitext": foreign,
        }),
    );

    assert!(preview.body.contains("wj-embed-video-ffffffff"));
    assert!(!preview.body.contains("<iframe"));
    assert!(preview.legacy_actions.is_empty());
}

#[tokio::test]
async fn literal_and_generated_owners_never_activate_embedvideo() {
    const GENERATED_TARGET: &str = "fixture-open43-m-media-generated-target";
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let iframe = r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen></iframe>"#;

    for (case_id, source) in [
        (
            "focused-embedvideo-code",
            format!("[[code]]\n[[embedvideo]]\n{iframe}\n[[/embedvideo]]\n[[/code]]"),
        ),
        (
            "open43-m-media-raw-owner",
            format!("@@[[embedvideo]]{iframe}[[/embedvideo]]@@"),
        ),
        (
            "open43-m-media-comment-owner",
            format!("[!-- [[embedvideo]]\n{iframe}\n[[/embedvideo]] --]"),
        ),
        (
            "open43-m-media-html-owner",
            format!("[[html]]\n[[embedvideo]]\n{iframe}\n[[/embedvideo]]\n[[/html]]"),
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );

        assert!(!preview.body.contains("<iframe"), "{case_id}");
        assert!(!preview.body.contains(NO_MATCH), "{case_id}");
        assert!(preview.legacy_actions.is_empty(), "{case_id}");
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(GENERATED_TARGET))),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "EmbedVideo generated owner target",
            "title": iframe,
            "alt_title": null,
            "slug": GENERATED_TARGET,
            "layout": "wikidot",
            "revision_comments": "Open43 M-Media generated owner fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let generated = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "open43-m-media-generated-owner",
            "wikitext": format!(
                "[[module ListPages fullname=\"{GENERATED_TARGET}\" limit=\"1\" separate=\"no\" wrapper=\"no\"]][[embedvideo]]%%title%%[[/embedvideo]][[/module]]"
            ),
        }),
    );
    assert!(!generated.body.contains("<iframe"));
    assert!(!generated.body.contains(NO_MATCH));
    assert!(generated.legacy_actions.is_empty());
}

#[tokio::test]
async fn current_ftml_pin_preserves_promoted_image_dom() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source, expected) in [
        (
            "issue-776-g06-f-equals-image",
            r#"[[f=image https://www.wikidot.com/local--files/files/wikidot_logo_100x30.png width="100px" alt="G06_IMAGE_ALT"]]"#,
            concat!(
                r#"<div class="image-container">"#,
                r#"<img src="https://www.wdfiles.com/local--files/files/wikidot_logo_100x30.png" width="100px" class="image" alt="G06_IMAGE_ALT">"#,
                "</div>",
            ),
        ),
        (
            "issue-806-g61-centered-image-one-space",
            r#"[[=image https://scp-wiki.wikidot.com/local--files/theme:magazine/IMG-goat.webp width="150px"]]"#,
            concat!(
                r#"<div class="image-container aligncenter">"#,
                r#"<img src="https://scp-wiki.wjfiles.com/local--files/theme:magazine/IMG-goat.webp" width="150px" class="image" alt="IMG-goat.webp">"#,
                "</div>",
            ),
        ),
        (
            "issue-806-g61-centered-image-multi-space",
            r#"[[=image  https://scp-wiki.wikidot.com/local--files/theme:magazine/IMG-goat.webp  width="200px"]]"#,
            concat!(
                r#"<div class="image-container aligncenter">"#,
                r#"<img src="https://scp-wiki.wjfiles.com/local--files/theme:magazine/IMG-goat.webp" width="200px" class="image" alt="IMG-goat.webp">"#,
                "</div>",
            ),
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );

        assert_eq!(preview.body, expected, "{case_id}");
    }
}

#[tokio::test]
async fn image_head_ascii_whitespace_boundary_matches_live_preview_and_saved_page() {
    struct Case {
        case_id: &'static str,
        source: &'static str,
        expected: &'static str,
        expected_image_container: Option<&'static str>,
        expected_literal_href: Option<&'static str>,
    }

    const PLAIN_IMAGE: &str = concat!(
        r#"<div class="image-container">"#,
        r#"<img src="https://example.com/a.png" width="100px" class="image" alt="a.png">"#,
        "</div>",
    );
    const CENTERED_IMAGE: &str = concat!(
        r#"<div class="image-container aligncenter">"#,
        r#"<img src="https://example.com/a.png" width="100px" class="image" alt="a.png">"#,
        "</div>",
    );
    let cases = [
        Case {
            case_id: "m776-space-one",
            source: r#"[[f=image https://example.com/a.png width="100px"]]"#,
            expected: PLAIN_IMAGE,
            expected_image_container: Some("image-container"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m776-space-two",
            source: r#"[[f=image  https://example.com/a.png  width="100px"]]"#,
            expected: PLAIN_IMAGE,
            expected_image_container: Some("image-container"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m776-tab",
            source: "[[f=image\thttps://example.com/a.png\twidth=\"100px\"]]",
            expected: PLAIN_IMAGE,
            expected_image_container: Some("image-container"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m776-nbsp",
            source: "[[f=image\u{00a0}https://example.com/a.png\u{00a0}width=\"100px\"]]",
            expected: concat!(
                "<p>[[f=image\u{00a0}",
                r#"<a href="https://example.com/a.png%C2%A0width=">"#,
                "https://example.com/a.png\u{00a0}width=</a>",
                "&quot;100px&quot;]]</p>",
            ),
            expected_image_container: None,
            expected_literal_href: Some("https://example.com/a.png%C2%A0width="),
        },
        Case {
            case_id: "m776-float-lookalike",
            source: r#"[[ff=image https://example.com/a.png width="100px"]]"#,
            expected: concat!(
                r#"<p>[[ff=image <a href="https://example.com/a.png">"#,
                "https://example.com/a.png</a> width=&quot;100px&quot;]]</p>",
            ),
            expected_image_container: None,
            expected_literal_href: Some("https://example.com/a.png"),
        },
        Case {
            case_id: "m806-space-one",
            source: r#"[[=image https://example.com/a.png width="100px"]]"#,
            expected: CENTERED_IMAGE,
            expected_image_container: Some("image-container aligncenter"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m806-space-two",
            source: r#"[[=image  https://example.com/a.png  width="100px"]]"#,
            expected: CENTERED_IMAGE,
            expected_image_container: Some("image-container aligncenter"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m806-tab",
            source: "[[=image\thttps://example.com/a.png\twidth=\"100px\"]]",
            expected: CENTERED_IMAGE,
            expected_image_container: Some("image-container aligncenter"),
            expected_literal_href: None,
        },
        Case {
            case_id: "m806-nbsp",
            source: "[[=image\u{00a0}https://example.com/a.png\u{00a0}width=\"100px\"]]",
            expected: concat!(
                "<p>[[=image\u{00a0}",
                r#"<a href="https://example.com/a.png%C2%A0width=">"#,
                "https://example.com/a.png\u{00a0}width=</a>",
                "&quot;100px&quot;]]</p>",
            ),
            expected_image_container: None,
            expected_literal_href: Some("https://example.com/a.png%C2%A0width="),
        },
        Case {
            case_id: "m806-lookalike",
            source: r#"[[==image https://example.com/a.png width="100px"]]"#,
            expected: concat!(
                r#"<p>[[==image <a href="https://example.com/a.png">"#,
                "https://example.com/a.png</a> width=&quot;100px&quot;]]</p>",
            ),
            expected_image_container: None,
            expected_literal_href: Some("https://example.com/a.png"),
        },
    ];

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for case in cases {
        runner.set_request_context(RequestContext {
            site_id: Some(site_id),
            ..Default::default()
        });
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case.case_id,
                "wikitext": case.source,
            }),
        );
        assert_eq!(preview.body, case.expected, "{} preview", case.case_id);
        assert!(preview.styles.is_empty(), "{} preview", case.case_id);
        assert!(
            preview.legacy_actions.is_empty(),
            "{} preview",
            case.case_id,
        );

        let slug = format!("fixture-{}", case.case_id);
        runner.set_request_context(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(Cow::Owned(slug.clone()))),
            ..Default::default()
        });
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": case.source,
                "title": case.case_id,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "Open43 image whitespace boundary fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );

        runner.set_request_context(RequestContext {
            site_id: Some(site_id),
            ..Default::default()
        });
        let saved = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        let saved_body = match saved {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("{} saved page should exist: {other:?}", case.case_id),
        };
        assert_eq!(saved_body, case.expected, "{} saved page", case.case_id);

        for (surface, body) in [("preview", preview.body), ("saved page", saved_body)] {
            assert!(
                !body.contains("wj-") && !body.contains(".wjfiles.com"),
                "{} {surface} leaked Wikijump-owned image identity: {body}",
                case.case_id,
            );
            match (case.expected_image_container, case.expected_literal_href) {
                (Some(container), None) => {
                    assert!(
                        body.starts_with(&format!(r#"<div class="{container}">"#)),
                        "{} {surface}: {body}",
                        case.case_id,
                    );
                    assert!(
                        body.contains(r#"<img src="https://example.com/a.png""#),
                        "{} {surface}: {body}",
                        case.case_id,
                    );
                    assert!(!body.contains("<a "), "{} {surface}: {body}", case.case_id);
                }
                (None, Some(href)) => {
                    assert!(
                        !body.contains("<img "),
                        "{} {surface}: {body}",
                        case.case_id
                    );
                    assert!(
                        body.contains(&format!(r#"<a href="{href}">"#)),
                        "{} {surface}: {body}",
                        case.case_id,
                    );
                }
                _ => unreachable!("every evidence case has exactly one owner"),
            }
        }
    }
}

#[tokio::test]
async fn m776_m806_image_argument_boundary_matches_retained_live_preview() {
    // Expected bodies are the 2026-09-13 anonymous PagePreview observations
    // with only the documented Deepwell projections applied: scp-wiki
    // wjfiles host/scheme, empty preview page for implicit attachments,
    // and FTML's DOM-equivalent attribute order and quote spelling.
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source, expected) in [
        (
            "m776-width-doc-quoted-px",
            "[[f=image https://example.com/a.png width=\"200px\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" width="200px" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-width-unquoted-px",
            "[[f=image https://example.com/a.png width=200px]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-width-quoted-number",
            "[[f=image https://example.com/a.png width=\"200\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" width="200" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-width-unquoted-number",
            "[[f=image https://example.com/a.png width=200]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-width-percent",
            "[[f=image https://example.com/a.png width=\"50%\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" width="50%" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-width-empty",
            "[[f=image https://example.com/a.png width=\"\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" width class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-size-small-quoted",
            "[[f=image https://example.com/a.png size=\"small\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-size-small-unquoted",
            "[[f=image https://example.com/a.png size=small]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-size-thumbnail-quoted",
            "[[f=image https://example.com/a.png size=\"thumbnail\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-size-invalid",
            "[[f=image https://example.com/a.png size=\"tiny\"]]",
            r#"<div class="image-container"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-source-double-quoted",
            "[[f=image \"https://example.com/a.png\"]]",
            r#"<div class="image-container"><img src="&quot;https://example.com/a.png&quot;" class="image" alt="a.png&quot;"></div>"#,
        ),
        (
            "m776-source-single-quoted",
            "[[f=image 'https://example.com/a.png']]",
            r#"<div class="image-container"><img src="&#39;https://example.com/a.png&#39;" class="image" alt="a.png&#39;"></div>"#,
        ),
        (
            "m776-float-left",
            "[[f<image https://example.com/a.png]]",
            r#"<div class="image-container floatleft"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-float-right",
            "[[f>image https://example.com/a.png]]",
            r#"<div class="image-container floatright"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m776-float-left-lookalike",
            "[[ff<image https://example.com/a.png]]",
            r#"<p>[[ff&lt;image <a href="https://example.com/a.png">https://example.com/a.png</a>]]</p>"#,
        ),
        (
            "m776-float-token-separated",
            "[[f >image https://example.com/a.png]]",
            r#"<p>[[f &gt;image <a href="https://example.com/a.png">https://example.com/a.png</a>]]</p>"#,
        ),
        (
            "m806-width-doc-quoted-px",
            "[[=image https://example.com/a.png width=\"200px\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" width="200px" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-width-unquoted-px",
            "[[=image https://example.com/a.png width=200px]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-width-quoted-number",
            "[[=image https://example.com/a.png width=\"200\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" width="200" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-width-unquoted-number",
            "[[=image https://example.com/a.png width=200]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-width-percent",
            "[[=image https://example.com/a.png width=\"50%\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" width="50%" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-width-empty",
            "[[=image https://example.com/a.png width=\"\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" width class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-size-small-quoted",
            "[[=image https://example.com/a.png size=\"small\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-size-small-unquoted",
            "[[=image https://example.com/a.png size=small]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-size-thumbnail-quoted",
            "[[=image https://example.com/a.png size=\"thumbnail\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-size-invalid",
            "[[=image https://example.com/a.png size=\"tiny\"]]",
            r#"<div class="image-container aligncenter"><img src="https://example.com/a.png" class="image" alt="a.png"></div>"#,
        ),
        (
            "m806-source-double-quoted",
            "[[=image \"https://example.com/a.png\"]]",
            r#"<div class="image-container aligncenter"><img src="&quot;https://example.com/a.png&quot;" class="image" alt="a.png&quot;"></div>"#,
        ),
        (
            "m806-source-single-quoted",
            "[[=image 'https://example.com/a.png']]",
            r#"<div class="image-container aligncenter"><img src="&#39;https://example.com/a.png&#39;" class="image" alt="a.png&#39;"></div>"#,
        ),
        (
            "m806-center-token-separated",
            "[[= image https://example.com/a.png]]",
            r#"<p>[[= image <a href="https://example.com/a.png">https://example.com/a.png</a>]]</p>"#,
        ),
        (
            "m806-center-double-token-separated",
            "[[== image https://example.com/a.png]]",
            r#"<p>[[== image <a href="https://example.com/a.png">https://example.com/a.png</a>]]</p>"#,
        ),
        (
            "m776-local-size-square",
            "[[f=image photo.png size=\"square\"]]",
            r#"<div class="image-container"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/square.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m776-local-size-thumbnail",
            "[[f=image photo.png size=\"thumbnail\"]]",
            r#"<div class="image-container"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/thumbnail.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m776-local-size-small",
            "[[f=image photo.png size=\"small\"]]",
            r#"<div class="image-container"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/small.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m776-local-size-invalid",
            "[[f=image photo.png size=\"tiny\"]]",
            r#"<div class="image-container"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
        (
            "m776-local-size-empty",
            "[[f=image photo.png size=\"\"]]",
            r#"<div class="image-container"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
        (
            "m776-local-size-unquoted",
            "[[f=image photo.png size=small]]",
            r#"<div class="image-container"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
        (
            "m806-local-size-square",
            "[[=image photo.png size=\"square\"]]",
            r#"<div class="image-container aligncenter"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/square.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m806-local-size-thumbnail",
            "[[=image photo.png size=\"thumbnail\"]]",
            r#"<div class="image-container aligncenter"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/thumbnail.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m806-local-size-small",
            "[[=image photo.png size=\"small\"]]",
            r#"<div class="image-container aligncenter"><a href="https://scp-wiki.wjfiles.com/local--files//photo.png"><img src="https://scp-wiki.wjfiles.com/local--resized-images//photo.png/small.jpg" class="image" alt="photo.png"></a></div>"#,
        ),
        (
            "m806-local-size-invalid",
            "[[=image photo.png size=\"tiny\"]]",
            r#"<div class="image-container aligncenter"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
        (
            "m806-local-size-empty",
            "[[=image photo.png size=\"\"]]",
            r#"<div class="image-container aligncenter"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
        (
            "m806-local-size-unquoted",
            "[[=image photo.png size=small]]",
            r#"<div class="image-container aligncenter"><img src="https://scp-wiki.wjfiles.com/local--files//photo.png" class="image" alt="photo.png"></div>"#,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert_eq!(preview.body, expected, "{case_id}");
        assert!(preview.styles.is_empty(), "{case_id}");
        assert!(preview.legacy_actions.is_empty(), "{case_id}");
    }
}

#[tokio::test]
async fn source_less_local_images_keep_original_and_resized_asset_identities() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, slug, source, expected_class) in [
        (
            "issue-776-source-less-floating-image",
            "fixture-source-less-image-776",
            r#"[[f<image image-one.png]]"#,
            "image-container floatleft",
        ),
        (
            "issue-806-source-less-centered-image",
            "fixture-source-less-image-806",
            r#"[[=image image-two.png]]"#,
            "image-container aligncenter",
        ),
    ] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(Cow::Borrowed(slug))),
        });
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": source,
                "title": case_id,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "Open43 source-less image fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        let page = run_endpoint!(
            runner,
            page_get,
            json!({
                "site_id": site_id,
                "page": slug,
                "details": {"compiled": true},
            }),
        )
        .expect("source-less local image fixture should exist");
        let rendered = page
            .compiled_body_html
            .expect("source-less local image fixture should have compiled HTML");

        let filename = if case_id.contains("776") {
            "image-one.png"
        } else {
            "image-two.png"
        };
        let original =
            format!("https://scp-wiki.wjfiles.com/local--files/{slug}/{filename}");
        let resized = format!(
            "https://scp-wiki.wjfiles.com/local--resized-images/{slug}/{filename}/medium.jpg"
        );
        assert!(
            rendered.contains(&format!(r#"<div class="{expected_class}">"#)),
            "{case_id}: {rendered}",
        );
        assert!(
            rendered.contains(&format!(r#"<a href="{original}">"#)),
            "{case_id}: {rendered}",
        );
        assert!(
            rendered.contains(&format!(r#"<img src="{resized}""#)),
            "{case_id}: {rendered}",
        );
    }
}

#[tokio::test]
async fn embedvideo_preview_and_saved_page_share_rendered_body() {
    const SLUG: &str = "fixture-open43-m-media-cache-identity";
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "[[embedvideo]]\n",
        r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/4sroHOHlkAk" frameborder="0" allowfullscreen></iframe>"#,
        "\n[[/embedvideo]]",
    );
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "EmbedVideo cache identity",
            "wikitext": source,
        }),
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(SLUG))),
    });
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "EmbedVideo cache identity",
            "alt_title": null,
            "slug": SLUG,
            "layout": "wikidot",
            "revision_comments": "Open43 M-Media cache identity fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(created.parser_errors.is_empty());

    let saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("saved embedvideo fixture should exist");

    assert_eq!(
        saved.compiled_body_html.as_deref(),
        Some(preview.body.as_str()),
    );
    assert!(saved.compiled_generator.ends_with("; deepwell-render/v14"));
}
