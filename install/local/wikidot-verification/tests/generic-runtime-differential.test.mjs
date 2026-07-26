import assert from "node:assert/strict";
import test from "node:test";

import {
  DeepwellRpcAdapter,
  RuntimeCleanupError,
  compareRuntimeFragment,
  externalStateReasons,
  runGenericRuntimeDifferential,
  selectLatestSuccessfulCaptures,
} from "../src/generic-runtime-differential.mjs";
import {sha256} from "../src/syntax-differential.mjs";
import {parseArgs} from "../scripts/run-generic-runtime-differential.mjs";
import {
  composeDocument,
  parseArgs as parseStackArgs,
  runtimeIdentity as stackRuntimeIdentity,
} from "../scripts/run-generic-runtime-differential-stack.mjs";

const runtimeIdentity = {
  schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1",
  wikijump_sha: "1".repeat(40),
  ftml_sha: "2".repeat(40),
  dependency_lock_sha256: "3".repeat(64),
  executable_sha256: "4".repeat(64),
  runtime_config_sha256: "5".repeat(64),
};

function runtimeCase(caseId, source = "alpha") {
  return {
    schema: "wikijump_syntax_differential.live_case.v1",
    case_id: caseId,
    source,
    source_sha256: sha256(source),
    execution_class: "wikijump-runtime",
  };
}

function capture(caseValue, {
  capturedAt = "2026-07-26T00:00:00Z",
  fragment = "<p>alpha</p>",
  status = "captured",
  slug = "run-owned:ftml-diff-20260726-001",
} = {}) {
  const marker = {
    case_id: caseValue.case_id,
    source_sha256: caseValue.source_sha256,
    marker_begin: `WJDIFF_BEGIN_${caseValue.case_id}`,
    marker_end: `WJDIFF_END_${caseValue.case_id}`,
  };
  const source = `${marker.marker_begin}\n${caseValue.source}\n${marker.marker_end}`;
  const value = {
    schema: "wikijump_syntax_differential.wikidot_saved_page_capture.v1",
    captured_at: capturedAt,
    capture_status: status,
    site: "sandbox-for-codex",
    domain: "sandbox-for-codex.wikidot.com",
    authenticated_capture: false,
    mutated: true,
    page_identity: 42,
    saved_source: source,
    saved_source_sha256: sha256(source),
    source_normalized: false,
    page_plan: {
      schema: "wikijump_syntax_differential.wikidot_page_plan.v1",
      slug,
      title: slug,
      source,
      source_sha256: sha256(source),
      cases: [marker],
    },
    ...(status === "captured"
      ? {
          page_content_html: `<div id="page-content"><p>${marker.marker_begin}</p>${fragment}<p>${marker.marker_end}</p></div>`,
        }
      : {}),
  };
  if (value.capture_status === "captured") {
    value.page_content_html_sha256 = sha256(value.page_content_html);
  }
  return value;
}

function combinedCapture(caseValues) {
  const markers = caseValues.map((caseValue) => ({
    case_id: caseValue.case_id,
    source_sha256: caseValue.source_sha256,
    marker_begin: `WJDIFF_BEGIN_${caseValue.case_id}`,
    marker_end: `WJDIFF_END_${caseValue.case_id}`,
  }));
  const source = markers.map((marker, index) =>
    `${marker.marker_begin}\n${caseValues[index].source}\n${marker.marker_end}`
  ).join("\n");
  const pageContentHtml = `<div id="page-content">${markers.map((marker, index) =>
    `<p>${marker.marker_begin}</p><p>${caseValues[index].source}</p><p>${marker.marker_end}</p>`
  ).join("")}</div>`;
  const value = capture(caseValues[0]);
  value.saved_source = source;
  value.saved_source_sha256 = sha256(source);
  value.page_plan.source = source;
  value.page_plan.source_sha256 = sha256(source);
  value.page_plan.cases = markers;
  value.page_content_html = pageContentHtml;
  value.page_content_html_sha256 = sha256(pageContentHtml);
  return value;
}

test("latest successful capture is selected by capture time, not input order", () => {
  const caseValue = runtimeCase("latest");
  const laterFailure = capture(caseValue, {
    capturedAt: "2026-07-26T03:00:00Z",
    status: "render-failed",
  });
  const laterSuccess = capture(caseValue, {
    capturedAt: "2026-07-26T02:00:00Z",
    fragment: "<p>later</p>",
  });
  const earlierSuccess = capture(caseValue, {
    capturedAt: "2026-07-26T01:00:00Z",
    fragment: "<p>earlier</p>",
  });
  const selection = selectLatestSuccessfulCaptures(
    [caseValue],
    [
      {path: "later.jsonl", captures: [laterFailure, laterSuccess]},
      {path: "earlier.jsonl", captures: [earlierSuccess]},
    ],
  );
  assert.equal(selection.selected.get(caseValue.case_id).wikidot_html, "<p>later</p>");
  assert.equal(selection.acquisitionFailed.length, 0);
});

test("capture validation rejects a changed saved source", () => {
  const caseValue = runtimeCase("source-hash");
  const invalid = capture(caseValue);
  invalid.saved_source += "changed";
  assert.throws(
    () => selectLatestSuccessfulCaptures([caseValue], [{path: "invalid.jsonl", captures: [invalid]}]),
    /saved source hash does not match/u,
  );
});

test("fragment comparison never hides mismatches behind inferred state preconditions", () => {
  const matching = compareRuntimeFragment(runtimeCase("match"), "<p>alpha</p>", "<p>alpha</p>");
  assert.equal(matching.status, "match");
  const stateDependent = compareRuntimeFragment(
    runtimeCase("include", "[[include target]]"),
    "<p>included</p>",
    "<p>missing</p>",
  );
  assert.equal(stateDependent.status, "true-mismatch");
  assert.deepEqual(stateDependent.suspected_state_preconditions, ["include-target-state"]);
  const mismatch = compareRuntimeFragment(
    runtimeCase("literal", "alpha"),
    "<p>alpha</p>",
    "<p>beta</p>",
  );
  assert.equal(mismatch.status, "true-mismatch");
});

test("runtime state diagnostics do not mistake deterministic file and email rendering for state", () => {
  assert.deepEqual(externalStateReasons("[[include component:card]]"), ["include-target-state"]);
  assert.deepEqual(externalStateReasons("[[include :scp-wiki:component:card]]"), [
    "cross-site-include-state",
  ]);
  assert.deepEqual(externalStateReasons("[[file attachment.txt]]"), []);
  assert.deepEqual(externalStateReasons("[[file ../attachment.txt]]"), []);
  assert.deepEqual(externalStateReasons("[[*user Alice]]"), ["user-identity-state"]);
  assert.deepEqual(externalStateReasons("alice@example.com"), []);
});

test("file host normalization keeps page slug differences visible", () => {
  const caseValue = runtimeCase("file", "[[file attachment.txt]]");
  const wikidot =
    '<p><a href="http://sandbox-for-codex.wdfiles.com/local--files/run-owned:fixture/attachment.txt">file</a></p>';
  const samePage =
    '<p><a href="https://sandbox-for-codex.wjfiles.localhost/local--files/run-owned:fixture/attachment.txt">file</a></p>';
  const changedPage =
    '<p><a href="https://sandbox-for-codex.wjfiles.localhost/local--files/fixture/attachment.txt">file</a></p>';
  assert.equal(compareRuntimeFragment(caseValue, wikidot, samePage).status, "match");
  const mismatch = compareRuntimeFragment(caseValue, wikidot, changedPage);
  assert.equal(mismatch.status, "true-mismatch");
  assert.deepEqual(mismatch.suspected_state_preconditions, []);
});

test("runner reports acquisition failures and cleans each page before the next", async () => {
  const capturedCase = runtimeCase("captured");
  const failedCase = runtimeCase("failed");
  const captured = capture(capturedCase);
  const failed = capture(failedCase, {
    status: "render-failed",
    slug: "run-owned:ftml-diff-20260726-002",
  });
  let activePages = 0;
  const adapter = {
    async withCompiledPage(page, inspect) {
      activePages += 1;
      assert.equal(activePages, 1);
      try {
        await inspect(captured.page_content_html);
      } finally {
        activePages -= 1;
      }
      return {slug: page.slug, cleanup: {status: "removed"}};
    },
  };
  const report = await runGenericRuntimeDifferential({
    cases: [capturedCase, failedCase],
    captureFiles: [{path: "captures.jsonl", captures: [captured, failed]}],
    externalReferences: [],
    runtimeIdentity,
    adapter,
  });
  assert.equal(activePages, 0);
  assert.equal(report.status, "incomplete");
  assert.equal(report.summary.match, 1);
  assert.equal(report.summary.acquisition_failed, 1);
  assert.equal(report.page_receipts[0].cleanup.status, "removed");
});

test("runner turns adapter failures into a fail-closed runtime error", async () => {
  const caseValue = runtimeCase("runtime-error");
  const report = await runGenericRuntimeDifferential({
    cases: [caseValue],
    captureFiles: [{path: "captures.jsonl", captures: [capture(caseValue)]}],
    externalReferences: [],
    runtimeIdentity,
    adapter: {
      async withCompiledPage() {
        throw new Error("cleanup failed");
      },
    },
  });
  assert.equal(report.status, "fail");
  assert.equal(report.summary.runtime_error, 1);
  assert.match(report.comparisons[0].diagnostic.error, /cleanup failed/u);
});

test("runner isolates a marker failure to the case that lost its sentinel", async () => {
  const brokenCase = runtimeCase("broken-marker");
  const intactCase = runtimeCase("intact-marker");
  const captured = combinedCapture([brokenCase, intactCase]);
  const intactMarker = captured.page_plan.cases[1];
  const localHtml =
    `<div id="page-content"><p>broken without sentinels</p><p>${intactMarker.marker_begin}</p>` +
    `<p>${intactCase.source}</p><p>${intactMarker.marker_end}</p></div>`;
  const report = await runGenericRuntimeDifferential({
    cases: [brokenCase, intactCase],
    captureFiles: [{path: "captures.jsonl", captures: [captured]}],
    externalReferences: [],
    runtimeIdentity,
    adapter: {
      async withCompiledPage(page, inspect) {
        await inspect(localHtml);
        return {slug: page.slug, cleanup: {status: "removed"}};
      },
    },
  });
  assert.equal(report.summary.runtime_error, 1);
  assert.equal(report.summary.match, 1);
  assert.equal(report.comparisons.find((value) => value.case_id === brokenCase.case_id).status, "runtime-error");
  assert.equal(report.comparisons.find((value) => value.case_id === intactCase.case_id).status, "match");
});

test("Deepwell adapter removes a created page when inspection fails", async () => {
  const methods = [];
  let pageExists = false;
  const fetchImpl = async (_url, options) => {
    const request = JSON.parse(options.body);
    methods.push(request.method);
    let result;
    if (request.method === "ping") result = "pong";
    else if (request.method === "site_get") result = {site_id: 7};
    else if (request.method === "login") result = {session_token: "token"};
    else if (request.method === "user_get") result = {user_id: 9};
    else if (request.method === "page_get") {
      result = pageExists
        ? {
            page_id: 11,
            revision_id: 12,
            wikitext: "fixture",
            compiled_body_html: "<p>fixture</p>",
          }
        : null;
    } else if (request.method === "page_create") {
      pageExists = true;
      result = {page_id: 11, revision_id: 12};
    } else if (request.method === "page_delete") {
      pageExists = false;
      result = null;
    } else {
      throw new Error(`unexpected method: ${request.method}`);
    }
    return {ok: true, json: async () => ({jsonrpc: "2.0", id: request.id, result})};
  };
  const adapter = new DeepwellRpcAdapter({
    rpcUrl: "http://127.0.0.1:2741/jsonrpc",
    siteSlug: "sandbox-for-codex",
    administratorEmail: "admin@example.test",
    administratorPassword: "secret",
    fetchImpl,
  });
  await assert.rejects(
    adapter.withCompiledPage(
      {slug: "runtime-001", title: "runtime-001", source: "fixture", source_sha256: sha256("fixture")},
      async () => {
        throw new Error("inspection failed");
      },
    ),
    /inspection failed/u,
  );
  assert.equal(pageExists, false);
  assert.equal(methods.filter((method) => method === "page_delete").length, 1);
  assert.throws(
    () => new DeepwellRpcAdapter({
      rpcUrl: "http://example.test/jsonrpc",
      siteSlug: "sandbox-for-codex",
      administratorEmail: "admin@example.test",
      administratorPassword: "secret",
    }),
    /loopback/u,
  );
});

test("runner aborts after a cleanup failure instead of contaminating later pages", async () => {
  const first = runtimeCase("cleanup-one");
  const second = runtimeCase("cleanup-two");
  let calls = 0;
  await assert.rejects(
    runGenericRuntimeDifferential({
      cases: [first, second],
      captureFiles: [{
        path: "captures.jsonl",
        captures: [
          capture(first, {slug: "run-owned:ftml-diff-20260726-001"}),
          capture(second, {slug: "run-owned:ftml-diff-20260726-002"}),
        ],
      }],
      externalReferences: [],
      runtimeIdentity,
      adapter: {
        async withCompiledPage() {
          calls += 1;
          throw new RuntimeCleanupError("cleanup failed");
        },
      },
    }),
    /cleanup failed/u,
  );
  assert.equal(calls, 1);
});

test("Deepwell adapter cleans a page created before a transport failure", async () => {
  let pageExists = false;
  let deleteCalls = 0;
  const fetchImpl = async (_url, options) => {
    const request = JSON.parse(options.body);
    let result;
    if (request.method === "ping") result = "pong";
    else if (request.method === "site_get") result = {site_id: 7};
    else if (request.method === "login") result = {session_token: "token"};
    else if (request.method === "user_get") result = {user_id: 9};
    else if (request.method === "page_get") {
      result = pageExists
        ? {page_id: 11, revision_id: 12, wikitext: "fixture", compiled_body_html: "<p>fixture</p>"}
        : null;
    } else if (request.method === "page_create") {
      pageExists = true;
      throw new Error("transport failed after save");
    } else if (request.method === "page_delete") {
      pageExists = false;
      deleteCalls += 1;
      result = null;
    } else {
      throw new Error(`unexpected method: ${request.method}`);
    }
    return {ok: true, json: async () => ({jsonrpc: "2.0", id: request.id, result})};
  };
  const adapter = new DeepwellRpcAdapter({
    rpcUrl: "http://127.0.0.1:2741/jsonrpc",
    siteSlug: "sandbox-for-codex",
    administratorEmail: "admin@example.test",
    administratorPassword: "secret",
    fetchImpl,
  });
  await assert.rejects(
    adapter.withCompiledPage(
      {slug: "runtime-001", title: "runtime-001", source: "fixture", source_sha256: sha256("fixture")},
      async () => {},
    ),
    /transport failed after save/u,
  );
  assert.equal(pageExists, false);
  assert.equal(deleteCalls, 1);
});

test("CLI requires explicit artifacts and preserves repeated capture inputs", () => {
  const args = parseArgs([
    "--cases", "cases.jsonl",
    "--captures", "first.jsonl",
    "--captures", "second.jsonl",
    "--runtime-identity", "identity.json",
    "--rpc-url", "http://127.0.0.1:2741/jsonrpc",
    "--output", "report.json",
  ]);
  assert.deepEqual(args.captures, ["first.jsonl", "second.jsonl"]);
  assert.equal(args.site, "sandbox-for-codex");
  assert.throws(() => parseArgs([]), /--cases is required/u);
});

test("disposable stack controller binds resources and candidate identity", () => {
  const args = parseStackArgs([
    "--repository", "/tmp/repository",
    "--cases", "/tmp/cases.jsonl",
    "--captures", "/tmp/first.jsonl",
    "--captures", "/tmp/second.jsonl",
    "--output", "/tmp/report.json",
  ]);
  assert.deepEqual(args.captures, ["/tmp/first.jsonl", "/tmp/second.jsonl"]);
  const labels = {"example.owner": "runtime-diff"};
  const compose = composeDocument({
    project: "runtime-diff-test",
    labels,
    images: {database: "sha256:1", cache: "sha256:2", files: "sha256:3", deepwell: "sha256:4"},
    binary: "/tmp/deepwell",
    config: "/tmp/config",
    migrations: "/tmp/migrations",
    locales: "/tmp/locales",
    seeder: "/tmp/seeder",
    port: 2741,
    credentials: {databasePassword: "database", filesAccessKey: "access", filesSecretKey: "secret"},
  });
  assert.match(compose, /runtime-diff-test-database/u);
  assert.match(compose, /runtime-diff-test-network/u);
  assert.match(compose, /\/data:size=256m,mode=0700/u);
  assert.doesNotMatch(compose, /runtime-diff-test-files/u);
  assert.equal(compose.match(/example\.owner/u)?.[0], "example.owner");

  const identity = stackRuntimeIdentity({
    source: {wikijump_sha: "1".repeat(40), ftml_sha: "2".repeat(40)},
    build: {
      cargo_lock_sha256: "3".repeat(64),
      binary_sha256: "4".repeat(64),
    },
  }, compose, "config");
  assert.equal(identity.wikijump_sha, "1".repeat(40));
  assert.equal(identity.runtime_config_sha256.length, 64);
});
