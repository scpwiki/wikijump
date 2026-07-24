import assert from "node:assert/strict";
import test from "node:test";

import {
  parseArgs as parseApplyManifestArgs,
  usage as applyManifestUsage,
} from "../scripts/apply-corpus-import-manifest.mjs";
import {
  parseArgs as parseOracleArgs,
  usage as oracleUsage,
} from "../scripts/oracle-fixture-check.mjs";
import {
  parseArgs as parseRenderHealthArgs,
  usage as renderHealthUsage,
} from "../scripts/render-health-sweep.mjs";
import {
  main as validateArtifactMain,
  parseArguments as parseValidateArtifactArguments,
  usage as validateArtifactUsage,
} from "../scripts/validate-artifact.mjs";
import {
  parseArgs as parseBuildManifestArgs,
  usage as buildManifestUsage,
} from "../scripts/build-corpus-import-manifest.mjs";
import {
  parseArgs as parseCompareArgs,
  usage as compareUsage,
} from "../scripts/compare-render-evidence.mjs";
import {
  parseArgs as parseDependencyArgs,
  usage as dependencyUsage,
} from "../scripts/dependency-closure-report.mjs";
import {
  parseArgs as parseImportHealthArgs,
  usage as importHealthUsage,
} from "../scripts/import-health-report.mjs";
import {
  parseArgs as parseMergeReadinessArgs,
  usage as mergeReadinessUsage,
  verdictExitCode,
} from "../scripts/merge-readiness-report.mjs";
import {
  parseArgs as parseReservationArgs,
  usage as reservationUsage,
} from "../scripts/extract-rokurokubi-reservations.mjs";
import {
  parseArgs as parseLayoutArgs,
  usage as layoutUsage,
} from "../scripts/layout-diagnostics.mjs";
import {
  main as measureMain,
  parseArgs as parseMeasureArgs,
} from "../scripts/measure.mjs";
import {
  parseArgs as parseDispatchArgs,
  usage as dispatchUsage,
} from "../scripts/wj-grid-dispatch.mjs";
import {
  parseArgs as parseWorkerArgs,
  usage as workerUsage,
} from "../scripts/wj-grid-worker-once.mjs";

test("reservation CLI parser preserves mirror and source provenance options", () => {
  assert.deepEqual(parseReservationArgs([
    "--source", "source.csv",
    "--output", "output.csv",
    "--manifest", "manifest.json",
    "--source-role", "active",
    "--source-gid", "123",
  ]), {
    mirrorOrigin: "https://scp-wiki.wikijump.localhost",
    sourceLabel: null,
    sourceRole: "active",
    sourceName: null,
    sourceGid: "123",
    sheetManifest: null,
    source: "source.csv",
    output: "output.csv",
    manifest: "manifest.json",
  });
  assert.match(reservationUsage(), /sheet-manifest/u);
});

test("measure CLI parser and main preserve wrapped command exit status", async () => {
  assert.equal(parseMeasureArgs(["--family", "node-mjs", "--", "node", "test.mjs"]).options.command, "node");
  const calls = [];
  const code = await measureMain(["--family", "node-mjs", "--", "node", "test.mjs"], {
    cwd: () => "/tmp/repo",
    run: async (options) => {
      calls.push(options);
      return {exitCode: 7};
    },
  });
  assert.equal(code, 7);
  assert.equal(calls[0].cwd, "/tmp/repo");
  assert.deepEqual(calls[0].args, ["test.mjs"]);
});

test("grid CLI parsers expose stable lane and tmux contracts", () => {
  assert.deepEqual(parseDispatchArgs(["attach", "--session", "session-a"]), {
    command: "attach",
    sessionName: "session-a",
    window: "0",
    pane: "0",
    dryRun: false,
    confirmReset: false,
  });
  assert.deepEqual(parseWorkerArgs([
    "--state-root", "/tmp/state",
    "--campaign-id", "campaign",
    "--lane", "3",
    "--executor", "loopback",
  ]), {
    stateRoot: "/tmp/state",
    campaignId: "campaign",
    lane: 3,
    executor: "loopback",
  });
  assert.match(dispatchUsage(), /confirm-reset/u);
  assert.match(workerUsage(), /--executor/u);
});

test("layout diagnostics CLI parser defaults a single canonical viewport", () => {
  const parsed = parseLayoutArgs([
    "--url", "https://scp-wiki.wikijump.localhost/scp-9506",
    "--output-dir", "/tmp/layout",
  ]);
  assert.deepEqual(parsed.viewports, [{width: 1366, height: 900}]);
  assert.equal(parsed.fixtureId, "EN:scp-9506");
  assert.match(layoutUsage(), /adjunct evidence/u);
});


test("render comparison CLI validates mode-specific evidence inputs", () => {
  assert.deepEqual(parseCompareArgs([
    "--pairs", "pairs.json",
    "--output-dir", "out",
    "--mode", "records",
    "--records", "records.json",
    "--run-id", "run-1",
    "--channel", "whitespace_collapse=off",
  ]), {
    pairs: "pairs.json",
    outputDir: "out",
    mode: "records",
    records: "records.json",
    ledger: null,
    runId: "run-1",
    channels: {whitespace_collapse: false},
  });
  assert.throws(
    () => parseCompareArgs(["--pairs", "pairs.json", "--output-dir", "out", "--mode", "records"]),
    /requires --records/u,
  );
  assert.match(compareUsage(), /--pairs/u);
});

test("corpus manifest CLI requires one source and derives corpus provenance", () => {
  assert.deepEqual(parseBuildManifestArgs([
    "--corpus-root", "/tmp/corpus",
    "--branch", "en",
    "--fullname", "scp-1000",
    "--output", "manifest.jsonl",
    "--summary", "summary.json",
  ]), {
    corpusRoot: "/tmp/corpus",
    sourceBundle: null,
    branch: "en",
    sourceSite: "en",
    sourceBranch: "en",
    output: "manifest.jsonl",
    summary: "summary.json",
    fullnames: ["scp-1000"],
  });
  assert.throws(
    () => parseBuildManifestArgs(["--corpus-root", "a", "--source-bundle", "b", "--output", "o", "--summary", "s"]),
    /exactly one/u,
  );
  assert.match(buildManifestUsage(), /source-bundle/u);
});

test("report CLIs expose structural exit rules without reading artifacts", () => {
  assert.equal(verdictExitCode({exit_code: 4}), 4);
  assert.equal(verdictExitCode({aggregate: {unclassified: 1}}), 2);
  assert.equal(verdictExitCode({aggregate: {regressions: ["x"]}}), 1);
  assert.equal(verdictExitCode({aggregate: {fail: 0}}), 0);
  assert.equal(parseMergeReadinessArgs(["--output", "report.json", "--run-id", "run"]).runId, "run");
  assert.equal(parseImportHealthArgs(["--log", "import.log", "--output", "verdict.json", "--threshold", "0.9"]).threshold, 0.9);
  assert.equal(parseDependencyArgs([
    "--inventory", "inventory.json",
    "--slug-file", "slugs.txt",
    "--output-dir", "out",
    "--max-depth", "4",
  ]).maxDepth, 4);
  assert.match(mergeReadinessUsage(), /validator/u);
  assert.match(importHealthUsage(), /threshold/u);
  assert.match(dependencyUsage(), /max-depth/u);
});


test("corpus apply CLI preserves fail-closed import defaults", () => {
  const parsed = parseApplyManifestArgs([
    "--manifest", "manifest.jsonl",
    "--dry-run",
    "--skip-attachments",
    "--slug", "scp-1000",
    "--limit", "1",
  ]);
  assert.equal(parsed.manifest, "manifest.jsonl");
  assert.equal(parsed.dryRun, true);
  assert.equal(parsed.skipAttachments, true);
  assert.deepEqual(parsed.slug, ["scp-1000"]);
  assert.equal(parsed.limit, 1);
  assert.equal(parsed.createMode, "rpc");
  assert.equal(parsed.attachmentCreateMode, "rpc");
  assert.throws(
    () => parseApplyManifestArgs([
      "--manifest", "manifest.jsonl",
      "--dry-run",
      "--skip-attachments",
      "--attachment-create-mode", "direct",
    ]),
    /cannot be combined/u,
  );
  assert.match(applyManifestUsage(), /operator-only local tool/u);
});

test("artifact validator CLI maps reports to validator exit codes", async () => {
  assert.deepEqual(parseValidateArtifactArguments([
    "/tmp/artifact",
    "--kind", "codex",
    "--expected-task-id", "task-1",
    "--expected-assignment-id", "assignment-1",
    "--require", "report.md",
  ]), {
    help: false,
    options: {
      artifactRoot: "/tmp/artifact",
      kind: "codex",
      expectedTaskId: "task-1",
      expectedAssignmentId: "assignment-1",
      requiredFiles: ["report.md"],
    },
  });
  const output = [];
  const code = await validateArtifactMain(["/tmp/artifact"], {
    validate: async (options) => ({valid: false, options}),
    exitCode: (report) => report.valid ? 0 : 5,
    stdout: (line) => output.push(JSON.parse(line)),
  });
  assert.equal(code, 5);
  assert.equal(output[0].options.artifactRoot, "/tmp/artifact");
  assert.match(validateArtifactUsage(), /expected-assignment-id/u);
});

test("render health CLI exposes local TLS and disposition policy explicitly", () => {
  const parsed = parseRenderHealthArgs([
    "--manifest", "manifest.jsonl",
    "--host", "scp-wiki.wikijump.localhost",
    "--output-dir", "out",
    "--run-id", "run-1",
    "--family", "EN",
    "--threshold", "0.9",
    "--concurrency", "3",
    "--address", "127.0.0.1",
    "--http-port", "8443",
    "--insecure-local-tls",
    "--disposition", "raw_marker=accepted",
  ]);
  assert.equal(parsed.concurrency, 3);
  assert.equal(parsed.port, 8443);
  assert.equal(parsed.insecureLocalTls, true);
  assert.deepEqual(parsed.dispositions, {raw_marker: "accepted"});
  assert.match(renderHealthUsage(), /disposition/u);
});

test("oracle fixture CLI keeps runtime mutation credentials explicit", () => {
  const parsed = parseOracleArgs([
    "--oracle", "oracle.jsonl",
    "--output", "verdict.json",
    "--api-url", "http://127.0.0.1:2747/jsonrpc",
    "--site-id", "6000002",
    "--slug-prefix", "oracle-test-",
    "--run-id", "run-1",
    "--admin-email", "admin@example.test",
    "--admin-pass", "test-password",
    "--user-id", "-1",
  ]);
  assert.equal(parsed.oracle, "oracle.jsonl");
  assert.equal(parsed.output, "verdict.json");
  assert.equal(parsed.siteId, 6000002);
  assert.equal(parsed.slugPrefix, "oracle-test-");
  assert.equal(parsed.adminEmail, "admin@example.test");
  assert.equal(parsed.adminPass, "test-password");
  assert.equal(parsed.userId, -1);
  assert.match(oracleUsage(), /--oracle/u);
});
