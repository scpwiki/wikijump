import assert from "node:assert/strict";
import test from "node:test";

import {validateStandingRefreshReceipt} from "../scripts/verify-standing-refresh.mjs";

const sha = (character) => character.repeat(64);
const git = (character) => character.repeat(40);
const ref = (name) => ({path: `/test/${name}.json`, sha256: sha("a")});
const image = (digit) => ({reference: `sha256:${sha(digit)}`, id: `sha256:${sha(digit)}`, repo_digests: [], labels: {}});
const images = {deepwell: image("1"), framerail: image("2"), wws: image("3")};
const rollback = {deepwell: image("4"), framerail: image("5"), wws: image("6")};
const resources = (set) => Object.fromEntries(Object.entries(set).map(([name, entry]) => [name, {
  owner: "standing-runtime",
  keep_until: "2026-10-16T00:00:00+00:00",
  id: entry.id,
}]));

function receipt() {
  return {
    schema_version: 3,
    kind: "standing-promotion",
    status: "pass",
    run_id: sha("e"),
    started_at: "2026-10-15T00:00:00Z",
    completed_at: "2026-10-15T00:01:00Z",
    activation_duration_seconds: 22,
    image_verification_duration_seconds: 1,
    compose_activation_duration_seconds: 9,
    health_duration_seconds: 7,
    canary_duration_seconds: 1,
    wikijump_sha: git("a"),
    wikijump_tree: git("b"),
    ftml_sha: git("c"),
    dependency_lock_sha256: sha("d"),
    promotion_precondition: ref("precondition"),
    prepared_receipt: ref("prepared"),
    runtime_home: "/test/runtime",
    project_name: "wikijump-standing",
    network_name: "wikijump-standing_default",
    images,
    rollback_images: rollback,
    protected_volumes: ["wikijump-standing-postgres-data", "wikijump-standing-files-data", "wikijump-standing-cache-data"],
    saved_page_render_freshness: {
      status: "pass", expected_compiled_generator: "ftml v1.0; deepwell-render/v24", stale_pages_remaining: 0,
    },
    render_ahead: {
      status: "pass", expected_compiled_generator: "ftml v1.0; deepwell-render/v24",
      stale_pages_before: 24573, receipt: ref("ahead"),
    },
    runtime_differential_identity: {
      ...ref("identity"),
      identity: {
        schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1",
        wikijump_sha: git("a"), ftml_sha: git("c"),
        dependency_lock_sha256: sha("d"), executable_sha256: sha("1"),
        runtime_config_sha256: sha("b"),
      },
    },
    health: {deepwell: "healthy", framerail: "healthy", wws: "healthy"},
    canary: {url: "http://scp-wiki.wikijump.localhost/scp-9506", status: "pass", required_markers: ["scp-9506", "page-content"]},
    cleanup: {status: "pass", candidate_receipt: ref("prepared"), receipt: ref("prepared"), superseded_images: []},
    resource_disposition: {
      active: resources(images), rollback: resources(rollback), volumes: "protected-and-untouched",
      worktrees: "none created", target_directories: "none created",
    },
  };
}

test("v3 standing refresh requires identity-bound successful render ahead", () => {
  const valid = receipt();
  assert.equal(validateStandingRefreshReceipt(valid), valid);
  assert.throws(() => validateStandingRefreshReceipt({...valid, render_ahead: undefined}), /render-ahead/u);
  assert.throws(() => validateStandingRefreshReceipt({...valid, render_ahead: {...valid.render_ahead, status: "incomplete"}}), /render-ahead/u);
  assert.throws(() => validateStandingRefreshReceipt({...valid, render_ahead: {...valid.render_ahead, expected_compiled_generator: "old"}}), /render-ahead/u);
  assert.throws(() => validateStandingRefreshReceipt({...valid, render_ahead: {...valid.render_ahead, receipt: {path: "relative", sha256: sha("a")}}}), /absolute/u);
});

test("v3 standing refresh allows current pages without a redundant render", () => {
  const valid = receipt();
  valid.render_ahead = {
    status: "not-needed", expected_compiled_generator: "ftml v1.0; deepwell-render/v24",
    stale_pages_before: 0,
  };
  assert.equal(validateStandingRefreshReceipt(valid), valid);
  assert.throws(() => validateStandingRefreshReceipt({...valid, render_ahead: {...valid.render_ahead, stale_pages_before: 1}}), /inconsistent/u);
});
