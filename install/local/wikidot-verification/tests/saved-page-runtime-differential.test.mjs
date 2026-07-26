import assert from "node:assert/strict";
import test from "node:test";

import {
  compiledGeneratorCheck,
  compareSavedPageRuntime,
  extractSelectedHtml,
  selectSavedPageReferences,
} from "../src/saved-page-runtime-differential.mjs";
import {sha256} from "../src/syntax-differential.mjs";

const identity = {
  schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1",
  wikijump_sha: "1".repeat(40),
  ftml_sha: "2".repeat(40),
  dependency_lock_sha256: "3".repeat(64),
  executable_sha256: "4".repeat(64),
  runtime_config_sha256: "5".repeat(64),
};

function reference(selectedHtml) {
  return {
    schema: "wikijump_syntax_differential.wikidot_saved_page_reference.v1",
    case: {
      case_id: "scp-9507-stray-open-include",
      selector: ".anom-bar-container",
      expected: {
        required_class_tokens: ["anom-bar-container", "item-9507-D", "clear-3"],
        forbidden_literals: ["[[include", "[["],
      },
    },
    captured_at: "2026-07-26T00:00:00Z",
    actor: {authenticated: false},
    site: {unix_name: "scp-wiki", domain: "scp-wiki.wikidot.com"},
    page: {
      slug: "scp-9507",
      identity: 1,
      revision_identity: 2,
      revision_number: 3,
      source_sha256: "6".repeat(64),
    },
    selected_html: selectedHtml,
    selected_html_sha256: sha256(selectedHtml),
    provenance: {
      transport: "anonymous-https",
      mutated: false,
      wikidot_py_commit: "7".repeat(40),
      requirements_sha256: "8".repeat(64),
    },
  };
}

function documentWithGenerator(html, generator = "ftml v1.42.0 [22222222]; deepwell-render/v1") {
  return `<main>${html}</main><script type="application/json">{"compiled_generator":"${generator}"}</script>`;
}

test("extractSelectedHtml requires exactly one selected runtime subtree", () => {
  assert.match(
    extractSelectedHtml(
      '<main><div class="anom-bar-container item-9507-D clear-3">ok</div></main>',
      ".anom-bar-container",
    ),
    /item-9507-D/u,
  );
  assert.throws(
    () => extractSelectedHtml("<main></main>", ".anom-bar-container"),
    /returned 0 nodes/u,
  );
});

test("saved-page comparison binds identities and accepts exact runtime behavior", () => {
  const html = '<div class="anom-bar-container item-9507-D clear-3"><span>ok</span></div>';
  const result = compareSavedPageRuntime(reference(html), documentWithGenerator(html), identity);
  assert.equal(result.status, "match");
  assert.equal(result.checks.dom_hierarchy_child_order_and_attributes.status, "match");
  assert.equal(result.checks.compiled_generator.status, "match");
  assert.equal(result.identities.wikijump.ftml_sha, identity.ftml_sha);
});

test("saved-page comparison reports DOM and unexpanded include differences", () => {
  const live = '<div class="anom-bar-container item-9507-D clear-3"><span>ok</span></div>';
  const local =
    '<div class="anom-bar-container item-9507-D clear-3"><span>ok<br></span>[[include bad]]</div>';
  const result = compareSavedPageRuntime(reference(live), documentWithGenerator(local), identity);
  assert.equal(result.status, "mismatch");
  assert.equal(result.checks.dom_hierarchy_child_order_and_attributes.status, "mismatch");
  assert.deepEqual(result.checks.unexpanded_directives.found, ["[[include", "[["]);
});

test("compiled generator check rejects stale, missing, and duplicate page artifacts", () => {
  assert.equal(
    compiledGeneratorCheck(
      '{"compiled_generator":"ftml v1.42.0 [22222222]; deepwell-render/v1"}',
      identity.ftml_sha,
    ).status,
    "match",
  );
  assert.equal(
    compiledGeneratorCheck(
      '{"compiled_generator":"ftml v1.42.0 [11111111]; deepwell-render/v1"}',
      identity.ftml_sha,
    ).status,
    "mismatch",
  );
  assert.equal(compiledGeneratorCheck("<main>no state</main>", identity.ftml_sha).status, "mismatch");
  assert.equal(
    compiledGeneratorCheck(
      '{"compiled_generator":"ftml [22222222]"}{"compiled_generator":"ftml [22222222]"}',
      identity.ftml_sha,
    ).status,
    "mismatch",
  );
});

test("saved-page case selection is explicit and rejects unknown filters", () => {
  const one = reference('<div class="anom-bar-container item-9507-D clear-3">ok</div>');
  assert.deepEqual(selectSavedPageReferences([one], [one.case.case_id]), [one]);
  assert.throws(
    () => selectSavedPageReferences([one], ["missing"]),
    /absent from the references/u,
  );
});
