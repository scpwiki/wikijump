use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn syntax_renderer_streams_rendered_and_invalid_cases_as_json_lines() {
    let valid_case = serde_json::json!({
        "schema": "wikijump_syntax_differential.syntax_case.v1",
        "case_id": "renderer-valid-case",
        "source": "**bold**",
        "title": "Renderer fixture",
        "page_context": {"site": "fixture-site", "page": "fixture-page"}
    });
    let unsupported_case = serde_json::json!({
        "schema": "unsupported-schema",
        "case_id": "renderer-unsupported-schema",
        "source": "plain text",
        "title": "Unsupported fixture"
    });
    let default_context_case = serde_json::json!({
        "schema": "wikijump_syntax_differential.syntax_case.v1",
        "case_id": "renderer-default-context",
        "source": "**fallback**",
        "title": "Default context fixture"
    });
    let anchor_whitespace_case = serde_json::json!({
        "schema": "wikijump_syntax_differential.syntax_case.v1",
        "case_id": "renderer-anchor-whitespace",
        "source": "[[a href=\"#\" class=\"home\"]]Front Page[[/a]][[a href=\"#u-target-1\"]] About Fog [[/a]]",
        "title": "Inline anchor whitespace fixture",
        "page_context": {"site": "fixture-site", "page": "fixture-page"}
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_wikidot_syntax_renderer"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("syntax renderer binary starts");
    let mut stdin = child.stdin.take().expect("syntax renderer stdin is piped");
    writeln!(stdin, "{valid_case}").expect("valid case is written");
    writeln!(stdin, "{anchor_whitespace_case}").expect("anchor case is written");
    writeln!(stdin, "{default_context_case}").expect("default-context case is written");
    writeln!(stdin, "{unsupported_case}").expect("unsupported case is written");
    writeln!(stdin, "{{invalid json").expect("malformed case is written");
    stdin
        .write_all(b"\xff\n")
        .expect("invalid UTF-8 case is written");
    drop(stdin);

    let output = child.wait_with_output().expect("syntax renderer exits");
    assert!(
        output.status.success(),
        "syntax renderer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let records: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .expect("syntax renderer emits UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("each output line is JSON"))
        .collect();
    assert_eq!(records.len(), 6);

    let rendered = &records[0];
    assert_eq!(
        rendered["schema"],
        "wikijump_syntax_differential.ftml_render_result.v1"
    );
    assert_eq!(rendered["case_id"], "renderer-valid-case");
    assert_eq!(rendered["status"], "rendered");
    assert!(
        rendered["html"]
            .as_str()
            .unwrap()
            .contains("<strong>bold</strong>")
    );
    assert_eq!(rendered["engine"]["layout"], "wikidot");
    assert!(rendered.get("error").is_none());

    let anchor_whitespace = &records[1];
    assert_eq!(anchor_whitespace["case_id"], "renderer-anchor-whitespace");
    assert_eq!(anchor_whitespace["status"], "rendered");
    assert!(anchor_whitespace["html"]
        .as_str()
        .unwrap()
        .contains(r##"<a class="home" href="#">Front Page</a> <a href="#u-target-1">About Fog </a>"##));

    let default_context = &records[2];
    assert_eq!(default_context["case_id"], "renderer-default-context");
    assert_eq!(default_context["status"], "rendered");
    assert!(
        default_context["html"]
            .as_str()
            .unwrap()
            .contains("<strong>fallback</strong>")
    );

    let unsupported = &records[3];
    assert_eq!(unsupported["case_id"], "renderer-unsupported-schema");
    assert_eq!(unsupported["status"], "input-error");
    assert_eq!(unsupported["html"], serde_json::Value::Null);
    assert!(
        unsupported["error"]
            .as_str()
            .unwrap()
            .contains("unsupported schema")
    );

    let malformed = &records[4];
    assert_eq!(malformed["case_id"], serde_json::Value::Null);
    assert_eq!(malformed["status"], "input-error");
    assert_eq!(malformed["html"], serde_json::Value::Null);
    assert!(
        malformed["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty())
    );

    let invalid_utf8 = &records[5];
    assert_eq!(invalid_utf8["case_id"], serde_json::Value::Null);
    assert_eq!(invalid_utf8["status"], "input-error");
    assert_eq!(invalid_utf8["html"], serde_json::Value::Null);
    assert!(
        invalid_utf8["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty())
    );
}
