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
import {
  bindLocalHtmlBlockPayloads,
  sha1,
} from "../src/runtime-html-blocks.mjs";
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

function externalReference(caseValue, rawHtml = "<p>alpha</p>") {
  return {
    schema: "wikijump_syntax_differential.wikidot_reference.v1",
    captured_at: "2026-07-26T00:00:00Z",
    provenance: {
      module: "edit/PagePreviewModule",
      authenticated: false,
      mutated: false,
      site: "sandbox-for-codex",
      site_domain: "sandbox-for-codex.wikidot.com",
      wikidot_py_version: "4.4.1",
      wikidot_py_commit: "1".repeat(40),
      requirements_sha256: "2".repeat(64),
    },
    syntax_case: {
      schema: "wikijump_syntax_differential.syntax_case.v1",
      case_id: caseValue.case_id,
      source: caseValue.source,
      title: caseValue.case_id,
      wikidot_observation_tier: "page-preview",
      local_execution_tier: "wikijump-runtime",
    },
    source_sha256: caseValue.source_sha256,
    raw_html: rawHtml,
    raw_html_sha256: sha256(rawHtml),
  };
}

function capture(caseValue, {
  capturedAt = "2026-07-26T00:00:00Z",
  fragment = "<p>alpha</p>",
  status = "captured",
  slug = "run-owned:ftml-diff-20260726-001",
} = {}) {
  const isolated = caseValue.page_scope === "isolated";
  const marker = isolated
    ? {
        case_id: caseValue.case_id,
        source_sha256: caseValue.source_sha256,
        page_scope: "isolated",
      }
    : {
        case_id: caseValue.case_id,
        source_sha256: caseValue.source_sha256,
        marker_begin: `WJDIFF_BEGIN_${caseValue.case_id}`,
        marker_end: `WJDIFF_END_${caseValue.case_id}`,
      };
  const source = isolated
    ? caseValue.source
    : `${marker.marker_begin}\n${caseValue.source}\n${marker.marker_end}`;
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
          page_content_html: isolated
            ? `<div id="page-content">${fragment}</div>`
            : `<div id="page-content"><p>${marker.marker_begin}</p>${fragment}<p>${marker.marker_end}</p></div>`,
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

test("capture validation rejects an isolated runtime case on a shared page", () => {
  const isolated = {...runtimeCase("isolated"), page_scope: "isolated"};
  const companion = runtimeCase("companion");
  const shared = combinedCapture([isolated, companion]);
  assert.throws(
    () => selectLatestSuccessfulCaptures(
      [isolated, companion],
      [{path: "invalid.jsonl", captures: [shared]}],
    ),
    /isolated runtime case shares a page: isolated/u,
  );
});

test("sentinel-free isolated capture uses the whole page content", () => {
  const isolated = {...runtimeCase("isolated", "alpha_"), page_scope: "isolated"};
  const selection = selectLatestSuccessfulCaptures(
    [isolated],
    [{path: "isolated.jsonl", captures: [capture(isolated, {fragment: "<p>EOF</p>"})]}],
  );
  assert.equal(selection.selected.get("isolated").wikidot_html, "<p>EOF</p>");
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

function liveTabview(id, {
  secondClass = "",
  secondDisplay = "display:none",
  secondPanel = "Second panel",
  secondPanelHtml = null,
  initializerId = id,
  initializerSuffix = "",
} = {}) {
  const nonce = initializerId.slice("wiki-tabview-".length);
  return [
    '<script src="http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--javascript/yahooui/tabview-min.js" type="text/javascript"></script>',
    `<div class="yui-navset" id="${id}">`,
    '<ul class="yui-nav">',
    '<li class="selected"><a href="javascript:;"><em>First</em></a></li>',
    `<li${secondClass}><a href="javascript:;"><em>Second</em></a></li>`,
    "</ul>",
    '<div class="yui-content">',
    '<div><p>First panel</p></div>',
    `<div style="${secondDisplay}">${secondPanelHtml ?? `<p>${secondPanel}</p>`}</div>`,
    "</div>",
    "</div>",
    '<script type="text/javascript">',
    "//<![CDATA[",
    "OZONE.dom.onDomReady(function(){",
    `        var tabView${nonce} = new YAHOO.widget.TabView('${initializerId}');`,
    '                }, "dummy-ondomready-block");',
    initializerSuffix,
    "//]]>",
    "</script>",
  ].join("\n");
}

function localTabview(id, {
  secondClass = "",
  secondDisplay = "display:none",
  secondPanel = "Second panel",
  secondPanelHtml = null,
} = {}) {
  return [
    "<!-- Wikidot tabview bootstrap omitted -->",
    `<div id="${id}" class="yui-navset">`,
    '<ul class="yui-nav">',
    '<li class="selected"><a href="javascript:;"><em>First</em></a></li>',
    `<li${secondClass}><a href="javascript:;"><em>Second</em></a></li>`,
    "</ul>",
    '<div class="yui-content">',
    '<div><p>First panel</p></div>',
    `<div style="${secondDisplay}">${secondPanelHtml ?? `<p>${secondPanel}</p>`}</div>`,
    "</div>",
    "</div>",
  ].join("\n");
}

test("tabview projection separates volatile bootstrap transport from static DOM", () => {
  const wikidotId = `wiki-tabview-${"a".repeat(32)}`;
  const wikijumpId = `wiki-tabview-${"b".repeat(32)}`;
  const comparison = compareRuntimeFragment(
    runtimeCase("tabview", "[[tabview]]"),
    liveTabview(wikidotId),
    localTabview(wikijumpId),
  );

  assert.equal(comparison.status, "static-match-browser-required");
  assert.equal(comparison.checks.dom_tree.status, "mismatch");
  assert.equal(comparison.checks.tabview_static_contract.status, "match");
  assert.equal(comparison.checks.tabview_static_contract.tabview_count, 1);
  assert.equal(
    comparison.checks.tabview_bootstrap_transport.status,
    "expected-platform-substitution",
  );
  assert.equal(comparison.checks.tabview_activation_contract.status, "not-run");
});

test("tabview projection keeps selected state, display, content, and identity uniqueness visible", () => {
  const wikidotId = `wiki-tabview-${"a".repeat(32)}`;
  const wikijumpId = `wiki-tabview-${"b".repeat(32)}`;
  const variants = [
    localTabview(wikijumpId, {secondClass: ' class="selected"'}),
    localTabview(wikijumpId, {secondDisplay: "display:block"}),
    localTabview(wikijumpId, {secondPanel: "Changed panel"}),
    `${localTabview(wikijumpId)}${localTabview(wikijumpId)}`,
  ];

  for (const html of variants) {
    const comparison = compareRuntimeFragment(
      runtimeCase("tabview", "[[tabview]]"),
      liveTabview(wikidotId),
      html,
    );
    assert.equal(comparison.status, "true-mismatch");
    assert.equal(comparison.checks.tabview_static_contract.status, "mismatch");
  }
});

test("tabview projection preserves nested ownership", () => {
  const wikidotOuter = `wiki-tabview-${"a".repeat(32)}`;
  const wikidotInner = `wiki-tabview-${"b".repeat(32)}`;
  const wikijumpOuter = `wiki-tabview-${"c".repeat(32)}`;
  const wikijumpInner = `wiki-tabview-${"d".repeat(32)}`;
  const wikidot = liveTabview(wikidotOuter, {
    secondPanelHtml: liveTabview(wikidotInner),
  });
  const localNested = localTabview(wikijumpOuter, {
    secondPanelHtml: localTabview(wikijumpInner),
  });
  const matching = compareRuntimeFragment(
    runtimeCase("nested-tabview", "[[tabview]][[tabview]]"),
    wikidot,
    localNested,
  );
  assert.equal(
    matching.status,
    "static-match-browser-required",
    JSON.stringify(matching, null, 2),
  );
  assert.equal(matching.checks.tabview_static_contract.status, "match");
  assert.equal(matching.checks.tabview_static_contract.tabview_count, 2);

  const moved = compareRuntimeFragment(
    runtimeCase("nested-tabview", "[[tabview]][[tabview]]"),
    wikidot,
    `${localTabview(wikijumpOuter)}${localTabview(wikijumpInner)}`,
  );
  assert.equal(moved.status, "true-mismatch");
  assert.equal(moved.checks.tabview_static_contract.status, "mismatch");
});

test("tabview projection rejects unknown or incorrectly bound initializer scripts", () => {
  const wikidotId = `wiki-tabview-${"a".repeat(32)}`;
  const wikijumpId = `wiki-tabview-${"b".repeat(32)}`;
  const wrongId = `wiki-tabview-${"c".repeat(32)}`;
  const missingLoader = liveTabview(wikidotId).replace(
    /^<script src="[^"]+" type="text\/javascript"><\/script>\n/u,
    "",
  );
  for (const html of [
    liveTabview(wikidotId, {initializerId: wrongId}),
    liveTabview(wikidotId, {initializerSuffix: "alert('unexpected');"}),
    missingLoader,
    `${liveTabview(wikidotId)}<script type="text/javascript"></script>`,
  ]) {
    const comparison = compareRuntimeFragment(
      runtimeCase("tabview", "[[tabview]]"),
      html,
      localTabview(wikijumpId),
    );
    assert.equal(comparison.status, "true-mismatch");
    assert.equal(comparison.checks.tabview_bootstrap_transport.status, "mismatch");
  }
});

test("runner keeps a static tabview match incomplete without browser evidence", async () => {
  const caseValue = runtimeCase("tabview-browser-required", "[[tabview]]");
  const wikidotId = `wiki-tabview-${"a".repeat(32)}`;
  const wikijumpId = `wiki-tabview-${"b".repeat(32)}`;
  const saved = capture(caseValue, {fragment: liveTabview(wikidotId)});
  const compiled = localTabview(wikijumpId);
  const report = await runGenericRuntimeDifferential({
    cases: [caseValue],
    captureFiles: [{path: "captures.jsonl", captures: [saved]}],
    externalReferences: [],
    runtimeIdentity,
    adapter: {
      async withCompiledPage(page, inspect) {
        await inspect(compiled);
        return {slug: page.slug, cleanup: {status: "removed"}};
      },
    },
  });

  assert.equal(report.status, "incomplete");
  assert.equal(report.summary.compared, 1);
  assert.equal(report.summary.static_match_browser_required, 1);
  assert.equal(report.comparisons[0].status, "static-match-browser-required");
});

test("runner compares a runtime PagePreview fallback through a cleaned local page", async () => {
  const caseValue = runtimeCase("preview-fallback");
  let inspectedPage = null;
  const report = await runGenericRuntimeDifferential({
    cases: [caseValue],
    captureFiles: [],
    externalReferences: [externalReference(caseValue)],
    runtimeIdentity,
    adapter: {
      async withCompiledPage(page, inspect) {
        inspectedPage = page;
        await inspect("<p>alpha</p>");
        return {slug: page.slug, cleanup: {status: "removed"}};
      },
    },
  });

  assert.equal(report.status, "pass");
  assert.equal(report.summary.match, 1);
  assert.equal(report.summary.external_reference, 1);
  assert.equal(report.summary.acquisition_failed, 0);
  assert.equal(report.comparisons[0].status, "match");
  assert.equal(report.comparisons[0].identities.observation_tier, "page-preview");
  assert.equal(inspectedPage.source, caseValue.source);
  assert.match(inspectedPage.slug, /^run-owned:ftml-preview-[0-9a-f]{24}$/u);
  assert.deepEqual(report.page_receipts[0].cleanup, {status: "removed"});
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

function htmlBlockFixture(payloads, {
  slug = "run-owned:ftml-diff-20260726-001",
  localWrapper = true,
  liveWrapper = true,
} = {}) {
  const blocks = payloads.map((payload, offset) => {
    const bytes = Buffer.from(payload);
    return {
      index: offset + 1,
      s3_filename: `42_html_${offset + 1}`,
      bytes: bytes.length,
      sha1: sha1(bytes),
      sha256: sha256(bytes),
    };
  });
  const localFrames = blocks.map(() =>
    '<iframe src="https://example.com/" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe>'
  );
  const liveFrames = blocks.map((block, offset) =>
    `<iframe allowtransparency="true" class="html-block-iframe" frameborder="0" src="/${slug}/html/${block.sha1}-${offset + 10}"></iframe>`
  );
  const wrap = (frames, enabled) => frames.map((frame) => enabled ? `<p>${frame}</p>` : frame).join("");
  const local = wrap(localFrames, localWrapper);
  const bound = bindLocalHtmlBlockPayloads(local, blocks);
  return {
    slug,
    blocks,
    local,
    localIdentity: bound.html,
    live: wrap(liveFrames, liveWrapper),
    binding: bound.binding,
  };
}

test("HTML block projection compares strict live URLs with stored local payload identities", () => {
  const fixture = htmlBlockFixture(["\n<b>first</b>\n", "raw-second"]);
  const comparison = compareRuntimeFragment(
    runtimeCase("html", "[[html]]payload[[/html]]"),
    fixture.live,
    fixture.local,
    {
      pageSlug: fixture.slug,
      wikijumpIdentityHtml: fixture.localIdentity,
      htmlBlockBinding: fixture.binding,
    },
  );
  assert.equal(comparison.status, "match", JSON.stringify(comparison, null, 2));
  assert.equal(comparison.checks.dom_tree.status, "mismatch");
  assert.equal(comparison.checks.html_block_contract.status, "match");
  assert.deepEqual(
    comparison.checks.html_block_contract.wikidot.blocks.map((block) => block.sha1),
    fixture.blocks.map((block) => block.sha1),
  );
  assert.deepEqual(
    comparison.checks.html_block_contract.wikijump.blocks.map((block) => block.stored_index),
    [1, 2],
  );
});

test("HTML block projection leaves structure, attributes, order, and invalid identities visible", () => {
  const fixture = htmlBlockFixture(["\n<b>first</b>\n", "raw-second"]);
  const options = {
    pageSlug: fixture.slug,
    wikijumpIdentityHtml: fixture.localIdentity,
    htmlBlockBinding: fixture.binding,
  };
  const changed = [
    fixture.live.replace(`/${fixture.slug}/html/`, "/wrong-slug/html/"),
    fixture.live.replace(/-[0-9]+/u, "-nonce"),
    fixture.live.replace(fixture.blocks[0].sha1, "f".repeat(40)),
    fixture.live.replace(' class="html-block-iframe"', ' class="html-block-iframe extra"'),
    fixture.live.replace(' frameborder="0"', ' frameborder="1"'),
    fixture.live.replace("<p><iframe", '<div><iframe').replace("</iframe></p>", "</iframe></div>"),
    fixture.live.replace(' frameborder="0"', ' data-extra="live" frameborder="0"'),
    fixture.live.replace(
      fixture.blocks[0].sha1,
      "0".repeat(40),
    ).replace(
      fixture.blocks[1].sha1,
      fixture.blocks[0].sha1,
    ).replace("0".repeat(40), fixture.blocks[1].sha1),
  ];
  for (const live of changed) {
    const comparison = compareRuntimeFragment(
      runtimeCase("html", "[[html]]payload[[/html]]"),
      live,
      fixture.local,
      options,
    );
    assert.equal(comparison.status, "true-mismatch", live);
    assert.equal(comparison.checks.html_block_contract.status, "mismatch");
  }
  const untracked = compareRuntimeFragment(
    runtimeCase("html", "[[html]]payload[[/html]]"),
    fixture.live,
    fixture.local,
    {pageSlug: fixture.slug},
  );
  assert.equal(untracked.status, "true-mismatch");
  assert.equal(untracked.checks.html_block_contract.status, "mismatch");
  const copiedLiveUrl = compareRuntimeFragment(
    runtimeCase("html", "[[html]]payload[[/html]]"),
    fixture.live,
    fixture.live,
    {pageSlug: fixture.slug},
  );
  assert.equal(copiedLiveUrl.status, "true-mismatch");
  assert.equal(copiedLiveUrl.checks.dom_tree.status, "match");
  assert.equal(copiedLiveUrl.checks.html_block_contract.status, "mismatch");
});

test("HTML source with no rendered block remains comparable with an empty tracked binding", () => {
  const comparison = compareRuntimeFragment(
    runtimeCase("hidden-html", "[[iftags +missing]][[html]]hidden[[/html]][[/iftags]]"),
    "<p>visible</p>",
    "<p>visible</p>",
    {
      pageSlug: "run-owned:ftml-diff-20260726-001",
      wikijumpIdentityHtml: "<p>visible</p>",
      htmlBlockBinding: {
        status: "tracked",
        iframe_count: 0,
        stored_block_count: 0,
        page_iframe_count: 0,
        page_stored_block_count: 0,
        blocks: [],
      },
    },
  );
  assert.equal(comparison.status, "match");
  assert.equal(comparison.checks.html_block_contract.status, "match");
});

test("runner binds a case fragment to its persisted HTML block payload", async () => {
  const slug = "run-owned:ftml-diff-20260726-001";
  const caseValue = {
    ...runtimeCase("html-runner", "[[html]]\n<b>stored</b>\n[[/html]]"),
    page_scope: "isolated",
  };
  const fixture = htmlBlockFixture(["\n<b>stored</b>\n"], {slug});
  const saved = capture(caseValue, {slug, fragment: fixture.live});
  const compiled = fixture.local;
  const report = await runGenericRuntimeDifferential({
    cases: [caseValue],
    captureFiles: [{path: "captures.jsonl", captures: [saved]}],
    externalReferences: [],
    runtimeIdentity,
    adapter: {
      async withCompiledPage(page, inspect) {
        await inspect(compiled, {iframe_count: 1, blocks: fixture.blocks});
        return {
          slug: page.slug,
          html_blocks: fixture.blocks,
          cleanup: {status: "removed", html_block_objects_removed: 1},
        };
      },
    },
  });
  assert.equal(report.status, "pass", JSON.stringify(report, null, 2));
  assert.equal(report.comparisons[0].status, "match");
  assert.equal(report.comparisons[0].checks.html_block_contract.status, "match");
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
        assert.equal(page.source, capturedCase.source);
        await inspect("<p>alpha</p>");
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

test("runner compiles every saved-page case from its exact source in a cleaned singleton", async () => {
  const brokenCase = runtimeCase("broken-marker", "broken");
  const intactCase = runtimeCase("intact-marker", "intact");
  const captured = combinedCapture([brokenCase, intactCase]);
  const calls = [];
  let activePages = 0;
  const report = await runGenericRuntimeDifferential({
    cases: [brokenCase, intactCase],
    captureFiles: [{path: "captures.jsonl", captures: [captured]}],
    externalReferences: [],
    runtimeIdentity,
    adapter: {
      async withCompiledPage(page, inspect) {
        activePages += 1;
        assert.equal(activePages, 1);
        calls.push(page);
        await inspect(`<p>${page.source}</p>`);
        activePages -= 1;
        return {slug: page.slug, cleanup: {status: "removed"}};
      },
    },
  });
  assert.equal(activePages, 0);
  assert.equal(report.summary.runtime_error, 0);
  assert.equal(report.summary.match, 2);
  assert.deepEqual(calls.map((page) => page.source), [brokenCase.source, intactCase.source]);
  assert.deepEqual(calls.map((page) => page.source_sha256), [
    brokenCase.source_sha256,
    intactCase.source_sha256,
  ]);
  assert.ok(calls.every((page) => page.slug === captured.page_plan.slug));
  assert.equal(report.page_receipts.length, 2);
  assert.ok(report.page_receipts.every((receipt) => receipt.cleanup.status === "removed"));
  assert.equal(report.comparisons.find((value) => value.case_id === brokenCase.case_id).status, "match");
  assert.equal(report.comparisons.find((value) => value.case_id === intactCase.case_id).status, "match");
  assert.ok(report.comparisons.every((value) =>
    value.identities.local_execution === "sentinel-free-singleton"
  ));
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
    } else if (request.method === "text_block_get_index") {
      result = null;
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
    textBlockBaseUrl: "http://127.0.0.1:9000/deepwell-text-blocks/",
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
      textBlockBaseUrl: "http://127.0.0.1:9000/deepwell-text-blocks/",
      siteSlug: "sandbox-for-codex",
      administratorEmail: "admin@example.test",
      administratorPassword: "secret",
    }),
    /loopback/u,
  );
});

test("Deepwell adapter records stored HTML payload identity before verified cleanup", async () => {
  const payload = Buffer.from("\n<b>stored</b>\n");
  let pageExists = false;
  let objectExists = false;
  const fetchImpl = async (_url, options) => {
    const request = JSON.parse(options.body);
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
            compiled_body_html: '<p><iframe src="https://example.com/" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe></p>',
          }
        : null;
    } else if (request.method === "page_create") {
      pageExists = true;
      objectExists = true;
      result = {page_id: 11, revision_id: 12};
    } else if (request.method === "text_block_get_index") {
      result = request.params.index === 1
        ? {index: 1, s3_filename: "11_html_1"}
        : null;
    } else if (request.method === "page_delete") {
      pageExists = false;
      objectExists = false;
      result = null;
    } else {
      throw new Error(`unexpected method: ${request.method}`);
    }
    return {ok: true, json: async () => ({jsonrpc: "2.0", id: request.id, result})};
  };
  const objectRequests = [];
  const textBlockFetchImpl = async (url) => {
    objectRequests.push(url);
    if (!objectExists) return {ok: false, status: 404};
    return {
      ok: true,
      status: 200,
      arrayBuffer: async () =>
        payload.buffer.slice(payload.byteOffset, payload.byteOffset + payload.byteLength),
    };
  };
  const adapter = new DeepwellRpcAdapter({
    rpcUrl: "http://127.0.0.1:2741/jsonrpc",
    textBlockBaseUrl: "http://127.0.0.1:9000/deepwell-text-blocks/",
    siteSlug: "sandbox-for-codex",
    administratorEmail: "admin@example.test",
    administratorPassword: "secret",
    fetchImpl,
    textBlockFetchImpl,
  });
  const receipt = await adapter.withCompiledPage(
    {
      slug: "runtime-001",
      title: "runtime-001",
      source: "fixture",
      source_sha256: sha256("fixture"),
    },
    async (_html, evidence) => {
      assert.equal(pageExists, true);
      assert.equal(objectExists, true);
      assert.equal(evidence.iframe_count, 1);
      assert.equal(evidence.blocks[0].sha1, sha1(payload));
      assert.equal(evidence.blocks[0].sha256, sha256(payload));
    },
  );
  assert.equal(pageExists, false);
  assert.equal(objectExists, false);
  assert.equal(receipt.html_blocks[0].sha1, sha1(payload));
  assert.equal(receipt.cleanup.html_block_objects_removed, 1);
  assert.equal(objectRequests.length, 2);
  assert.equal(
    objectRequests[0],
    "http://127.0.0.1:9000/deepwell-text-blocks/11_html_1",
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
    textBlockBaseUrl: "http://127.0.0.1:9000/deepwell-text-blocks/",
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
    "--text-block-url", "http://127.0.0.1:9000/deepwell-text-blocks/",
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
    rpcPort: 2741,
    textBlockPort: 9000,
    credentials: {databasePassword: "database", filesAccessKey: "access", filesSecretKey: "secret"},
  });
  assert.match(compose, /runtime-diff-test-database/u);
  assert.match(compose, /runtime-diff-test-network/u);
  assert.match(compose, /\/data:size=256m,mode=0700/u);
  assert.match(compose, /127\.0\.0\.1:9000:9000/u);
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
