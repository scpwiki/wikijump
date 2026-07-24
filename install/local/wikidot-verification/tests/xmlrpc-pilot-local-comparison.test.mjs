import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  runXmlrpcPilotLocalComparison,
} from "../src/xmlrpc-pilot-local-comparison.mjs";
import {
  comparisonOutputDir,
  localPage,
  readJsonl,
  sealPilot,
  startRpcServer,
} from "./support/xmlrpc-pilot-local-comparison-fixture.mjs";

test("verified XML-RPC pilot rows compare exactly, retain tombstones, and replay idempotently", async (t) => {
  const fixture = await sealPilot(t);
  const pages = new Map(
    [...fixture.responses].map(([fullname, response]) => [
      fullname,
      localPage(response),
    ]),
  );
  const firstResponse = fixture.responses.values().next().value;
  pages.set(
    firstResponse.fullname,
    localPage(firstResponse, {
      page_updated_at: firstResponse.updated_at.replace("Z", ".000+00:00"),
    }),
  );
  const rpc = await startRpcServer(t, pages);
  const outputDir = comparisonOutputDir(fixture, "comparison-output");
  const first = await runXmlrpcPilotLocalComparison({
    outputDir,
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(first.exit_code, 0);
  assert.equal(first.verdict.gate.status, "pass");
  assert.deepEqual(first.verdict.gate.status_counts, {
    matched: 2,
    reference_deleted: 1,
  });
  const records = readJsonl(await fs.readFile(first.output.rows));
  assert.deepEqual(
    records.map((record) => record.status),
    ["matched", "matched", "reference_deleted"],
  );
  assert.equal(Object.hasOwn(records[2].reference, "content_sha256"), false);
  assert.equal(Object.hasOwn(records[2], "local"), false);
  assert.equal(
    rpc.calls.filter((call) => call.method === "page_get").length,
    2,
  );
  assert.equal(
    rpc.calls.every(
      (call) => call.headers["x-deepwell-session-token"] === undefined,
    ),
    true,
  );
  for (const call of rpc.calls.filter((entry) => entry.method === "page_get")) {
    assert.deepEqual(call.params.details, { compiled: true, wikitext: true });
    assert.equal(call.params.site_id, 42);
  }
  const manifestRows = readJsonl(await fs.readFile(first.output.manifest));
  assert.equal(Object.hasOwn(manifestRows[2].reference, "response"), false);
  assert.equal(manifestRows[2].reference.kind, "wikidot_deleted");
  assert.equal((await fs.stat(first.output.verdict)).mode & 0o777, 0o400);
  const before = await fs.readFile(first.output.verdict);
  const second = await runXmlrpcPilotLocalComparison({
    outputDir,
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(second.exit_code, 0);
  assert.deepEqual(await fs.readFile(first.output.verdict), before);
});

test("source, HTML, metadata, and missing-page differences form deterministic mismatch clusters", async (t) => {
  const fixture = await sealPilot(t);
  const responses = [...fixture.responses.values()];
  const pages = new Map([
    [
      responses[0].fullname,
      localPage(responses[0], {
        compiled_body_html: "<p>different</p>",
        wikitext: "different\n",
      }),
    ],
    [responses[1].fullname, null],
  ]);
  const rpc = await startRpcServer(t, pages);
  const result = await runXmlrpcPilotLocalComparison({
    outputDir: comparisonOutputDir(fixture, "mismatch-output"),
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(result.exit_code, 1);
  assert.equal(result.verdict.gate.status, "fail");
  const clusters = JSON.parse(
    await fs.readFile(result.output.clusters, "utf8"),
  );
  assert.deepEqual(
    clusters.map((cluster) => cluster.category),
    ["compiled_html", "local_missing", "source_content"],
  );
  const rows = readJsonl(await fs.readFile(result.output.rows));
  assert.deepEqual(rows[0].differences, ["source_content", "compiled_html"]);
  assert.equal(rows[1].status, "local_missing");
});

test("a local RPC error produces a sealed error verdict while deleted pilot rows remain neutral", async (t) => {
  const fixture = await sealPilot(t);
  const rpc = await startRpcServer(t, new Map(), { siteError: true });
  const result = await runXmlrpcPilotLocalComparison({
    outputDir: comparisonOutputDir(fixture, "error-output"),
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(result.exit_code, 2);
  assert.equal(result.verdict.gate.status, "error");
  assert.equal(result.verdict.gate.error_count, 2);
  assert.deepEqual(result.verdict.gate.status_counts, {
    local_error: 2,
    reference_deleted: 1,
  });
  assert.equal(
    rpc.calls.filter((call) => call.method === "page_get").length,
    0,
  );
});

test("input receipt substitution is rejected before the local RPC or output directory are touched", async (t) => {
  const fixture = await sealPilot(t);
  const resultPath = path.join(fixture.pilotRoot, "receipts", "result.json");
  await fs.appendFile(resultPath, "\n", "utf8");
  let calls = 0;
  await assert.rejects(
    runXmlrpcPilotLocalComparison({
      fetchImpl: async () => {
        calls += 1;
        throw new Error("must not run");
      },
      outputDir: comparisonOutputDir(fixture, "rejected-output"),
      pilotRoot: fixture.pilotRoot,
      rpcUrl: "http://127.0.0.1:29999/jsonrpc",
      runtimeIdentityPath: fixture.runtimeIdentityPath,
      sourceExpectation: fixture.sourceExpectation,
    }),
    /receipts do not match the designated source/u,
  );
  assert.equal(calls, 0);
  await assert.rejects(
    fs.access(comparisonOutputDir(fixture, "rejected-output")),
  );
});

test("a derived manifest substitution is rejected before the local RPC or output directory are touched", async (t) => {
  const fixture = await sealPilot(t);
  let calls = 0;
  await assert.rejects(
    runXmlrpcPilotLocalComparison({
      fetchImpl: async () => {
        calls += 1;
        throw new Error("must not run");
      },
      outputDir: comparisonOutputDir(fixture, "wrong-manifest-output"),
      pilotRoot: fixture.pilotRoot,
      rpcUrl: "http://127.0.0.1:29999/jsonrpc",
      runtimeIdentityPath: fixture.runtimeIdentityPath,
      sourceExpectation: {
        ...fixture.sourceExpectation,
        verified_pilot_manifest: {
          ...fixture.sourceExpectation.verified_pilot_manifest,
          sha256: "f".repeat(64),
        },
      },
    }),
    /manifest does not match the designated source/u,
  );
  assert.equal(calls, 0);
  await assert.rejects(
    fs.access(comparisonOutputDir(fixture, "wrong-manifest-output")),
  );
});

test("a self-consistent but different pilot is rejected by the designated source pin", async (t) => {
  const fixture = await sealPilot(t);
  let calls = 0;
  await assert.rejects(
    runXmlrpcPilotLocalComparison({
      fetchImpl: async () => {
        calls += 1;
        throw new Error("must not run");
      },
      outputDir: comparisonOutputDir(fixture, "wrong-designation-output"),
      pilotRoot: fixture.pilotRoot,
      rpcUrl: "http://127.0.0.1:29999/jsonrpc",
      runtimeIdentityPath: fixture.runtimeIdentityPath,
      sourceExpectation: XMLRPC_EN_128_DESIGNATED_SOURCE,
    }),
    /receipts do not match the designated source/u,
  );
  assert.equal(calls, 0);
  await assert.rejects(
    fs.access(comparisonOutputDir(fixture, "wrong-designation-output")),
  );
});
