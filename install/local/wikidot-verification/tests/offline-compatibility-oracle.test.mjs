import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";

import {
  DEFAULT_VISUAL_RMSE_THRESHOLD,
  comparePngRmse,
  loadOfflineBrowserOracle,
  parseImageMagickRmse,
  validateOfflineBrowserOracle,
} from "../src/offline-compatibility-oracle.mjs";
import {browserReplayPayload, loadOfflineBrowserResponseFixture} from "../src/offline-browser-response-fixture.mjs";
import {
  parseArgs,
  requestIsOfflineSafe,
} from "../scripts/run-offline-browser-oracle.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const oraclePath = path.join(
  here,
  "..",
  "fixtures",
  "offline-compatibility",
  "scp-9506-final-zero-oracle.json",
);
const responseFixturePath = path.join(
  here,
  "..",
  "fixtures",
  "offline-compatibility",
  "scp-9506-browser-responses.json.gz",
);

test("SCP-9506 offline oracle is repository-owned and binds final-zero evidence", async () => {
  const {oracle, goldenPath} = await loadOfflineBrowserOracle(oraclePath);
  assert.equal(oracle.fixture_id, "scp-9506-final-zero-20260915");
  assert.equal(oracle.provenance.final_zero_status, "pass");
  assert.equal(
    oracle.provenance.final_zero_wikijump_sha,
    "162b0a5fe340ea126c9d0e5bddd8398be511e55e",
  );
  assert.equal(oracle.pair.live_url, "https://scp-wiki.wikidot.com/scp-9506");
  assert.equal(
    oracle.pair.local_url,
    "https://scp-wiki.wikijump.localhost/scp-9506",
  );
  assert.equal(oracle.accepted_comparison.status, "pass");
  assert.equal(path.basename(goldenPath), oracle.verified_local_golden.file);
});

test("offline browser oracle rejects self-approved or incomplete fixtures", () => {
  const base = {
    schema: "wikijump.offline_compatibility_browser_oracle.v1",
    provenance: {
      final_zero_status: "fail",
      final_zero_receipt_sha256: "a".repeat(64),
      standing_browser_proof_sha256: "b".repeat(64),
      standing_browser_parity_sha256: "c".repeat(64),
      live_reference_sha256: "d".repeat(64),
      accepted_browser: {executable_sha256: "e".repeat(64)},
    },
    pair: {
      live_url: "https://scp-wiki.wikidot.com/scp-9506",
      local_url: "https://scp-wiki.wikijump.localhost/scp-9506",
    },
    live_capture: {
      input_url: "https://scp-wiki.wikidot.com/scp-9506",
      final_url: "https://scp-wiki.wikidot.com/scp-9506",
      navigation_status: 200,
    },
    accepted_comparison: {status: "pass"},
    verified_local_golden: {
      file: "golden.png",
      sha256: "f".repeat(64),
    },
  };
  assert.throws(
    () => validateOfflineBrowserOracle(base),
    /must derive from a passing final-zero/u,
  );
  base.provenance.final_zero_status = "pass";
  base.accepted_comparison.status = "fail";
  assert.throws(
    () => validateOfflineBrowserOracle(base),
    /lacks a passing accepted comparison/u,
  );
});

test("offline request policy admits only loopback and Wikijump localhost origins", () => {
  for (const url of [
    "https://scp-wiki.wikijump.localhost/scp-9506",
    "https://scp-wiki.wjfiles.localhost/local--files/scp-9506/a.png",
    "http://127.0.0.1:2747/jsonrpc",
    "data:text/plain,fixture",
    "about:blank",
  ]) {
    assert.equal(requestIsOfflineSafe(url), true, url);
  }
  for (const url of [
    "https://scp-wiki.wikidot.com/scp-9506",
    "https://scp-wiki.wdfiles.com/local--files/scp-9506/a.png",
    "https://fonts.googleapis.com/css2?family=Inter",
    "https://example.com/",
  ]) {
    assert.equal(requestIsOfflineSafe(url), false, url);
  }
});

test("ImageMagick RMSE parser and threshold classify visual drift", async () => {
  assert.deepEqual(parseImageMagickRmse("123.45 (0.001884)"), {
    absolute: 123.45,
    normalized: 0.001884,
  });
  const pass = await comparePngRmse("expected.png", "actual.png", {
    threshold: 0.01,
    run: async () => ({stderr: "123.45 (0.001884)"}),
  });
  assert.equal(pass.status, "pass");
  const fail = await comparePngRmse("expected.png", "actual.png", {
    threshold: 0.001,
    run: async () => {
      const error = new Error("different");
      error.code = 1;
      error.stderr = "123.45 (0.001884)";
      throw error;
    },
  });
  assert.equal(fail.status, "fail");
  assert.equal(DEFAULT_VISUAL_RMSE_THRESHOLD, 0.015);
});

test("offline browser CLI defaults to the hermetic SCP-9506 oracle", () => {
  const args = parseArgs([]);
  assert.equal(args.localOrigin, "https://scp-wiki.wikijump.localhost");
  assert.equal(args.viewport.width, 1366);
  assert.equal(args.viewport.height, 900);
  assert.match(args.oracle, /scp-9506-final-zero-oracle\.json$/u);
  assert.throws(
    () => parseArgs(["--local-origin", "https://scp-wiki.wikidot.com"]),
    /wikijump\.localhost/u,
  );
});

test("offline response fixture resolves retained redirects without browser egress", async () => {
  const fixture = await loadOfflineBrowserResponseFixture(responseFixturePath);
  const retained = fixture.lookup(
    "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css",
  );
  assert.equal(retained.status, 200);
  assert.deepEqual(retained.redirect_chain, [
    "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css",
    "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com/theme/en/basalt/basalt-bedrock-min.css",
  ]);
  assert.match(retained.headers["content-type"], /text\/css/u);
});

test("browser replay uses the retained CSS redirect destination for relative font URLs", async () => {
  const fixture = await loadOfflineBrowserResponseFixture(responseFixturePath);
  const source = "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css";
  const retained = fixture.lookup(source);
  const finalUrl = retained.resolved_url;
  assert.equal(
    finalUrl,
    "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com/theme/en/basalt/basalt-bedrock-min.css",
  );
  const {body, rebased_urls: rebased} = browserReplayPayload(retained);
  assert.equal(retained.status, 200);
  assert.ok(rebased > 1);
  const correctedFont = new URL("./fonts/Jost-Book.woff2", finalUrl).href;
  assert.ok(body.toString("utf8").includes(correctedFont));
  assert.ok(retained.body.toString("utf8").includes("./fonts/Jost-Book.woff2"));
  assert.equal(fixture.lookup("https://example.invalid/never-recorded.css"), null);
  assert.equal(browserReplayPayload(fixture.lookup(finalUrl)).rebased_urls, 0);
  assert.equal(browserReplayPayload(null), null);
});

test("redirected CSS rewriting preserves absolute references and rebases imports without touching fixture bytes", () => {
  const body = Buffer.from([
    '@import "../site.css";',
    '@font-face{src:url(fonts/Book.woff2),url("../fonts/Second.woff2")}',
    'a{background:url("data:image/svg+xml,body"),url(/icons/icon.svg),url(#filter),url("https://other.test/image.svg")}',
  ].join("\n"));
  const normalized = browserReplayPayload({
    original_url: "https://entry.test/css/site.css",
    resolved_url: "https://cdn.test/build/css/site.css",
    headers: {"Content-Type": "text/css; charset=utf-8"},
    body,
  });
  assert.equal(normalized.rebased_urls, 3);
  const css = normalized.body.toString("utf8");
  assert.ok(css.includes('@import "https://cdn.test/build/site.css"'));
  assert.ok(css.includes('url(https://cdn.test/build/css/fonts/Book.woff2)'));
  assert.ok(css.includes('url("https://cdn.test/build/fonts/Second.woff2")'));
  assert.ok(css.includes('url("data:image/svg+xml,body")'));
  assert.ok(css.includes('url(/icons/icon.svg)'));
  assert.ok(css.includes('url(#filter)'));
  assert.ok(css.includes('url("https://other.test/image.svg")'));
  assert.ok(body.toString("utf8").includes('url(fonts/Book.woff2)'));
});
