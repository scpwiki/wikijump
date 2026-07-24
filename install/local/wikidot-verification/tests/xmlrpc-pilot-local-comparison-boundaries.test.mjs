import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  LocalPageReadClient,
  runXmlrpcPilotLocalComparison,
} from "../src/xmlrpc-pilot-local-comparison.mjs";
import {parseArgs} from "../scripts/compare-xmlrpc-pilot-local.mjs";
import {
  comparisonOutputDir,
  execFileAsync,
  localPage,
  readJsonl,
  sealPilot,
  startRpcServer,
} from "./support/xmlrpc-pilot-local-comparison-fixture.mjs";

test("a page_get response must identify the requested site and full slug", async (t) => {
  const fixture = await sealPilot(t);
  const responses = [...fixture.responses.values()];
  const pages = new Map([
    [
      responses[0].fullname,
      localPage(responses[0], { slug: "not-the-requested-page" }),
    ],
    [responses[1].fullname, localPage(responses[1])],
  ]);
  const rpc = await startRpcServer(t, pages);
  const result = await runXmlrpcPilotLocalComparison({
    outputDir: comparisonOutputDir(fixture, "identity-mismatch-output"),
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(result.exit_code, 2);
  const rows = readJsonl(await fs.readFile(result.output.rows));
  assert.equal(rows[0].status, "local_error");
  assert.equal(rows[0].error_kind, "returned_page_identity");
  assert.equal(rows[1].status, "matched");
});

test("a page_get response with an invalid RFC3339 calendar date is a local error", async (t) => {
  const fixture = await sealPilot(t);
  const responses = [...fixture.responses.values()];
  const pages = new Map([
    [
      responses[0].fullname,
      localPage(responses[0], { page_updated_at: "2026-02-30T00:00:00Z" }),
    ],
    [responses[1].fullname, localPage(responses[1])],
  ]);
  const rpc = await startRpcServer(t, pages);
  const result = await runXmlrpcPilotLocalComparison({
    outputDir: comparisonOutputDir(fixture, "invalid-local-timestamp-output"),
    pilotRoot: fixture.pilotRoot,
    rpcUrl: rpc.rpcUrl,
    runtimeIdentityPath: fixture.runtimeIdentityPath,
    sourceExpectation: fixture.sourceExpectation,
  });
  assert.equal(result.exit_code, 2);
  const rows = readJsonl(await fs.readFile(result.output.rows));
  assert.equal(rows[0].status, "local_error");
  assert.equal(rows[0].error_kind, "invalid_page_result");
  assert.equal(rows[1].status, "matched");
});

test(
  "receipt FIFOs and existing output FIFOs fail without blocking",
  { timeout: 5000 },
  async (t) => {
    const sourceFixture = await sealPilot(t);
    const resultPath = path.join(
      sourceFixture.pilotRoot,
      "receipts",
      "result.json",
    );
    await fs.unlink(resultPath);
    await execFileAsync("mkfifo", [resultPath]);
    await assert.rejects(
      runXmlrpcPilotLocalComparison({
        outputDir: comparisonOutputDir(sourceFixture, "receipt-fifo-output"),
        pilotRoot: sourceFixture.pilotRoot,
        rpcUrl: "http://127.0.0.1:29999/jsonrpc",
        runtimeIdentityPath: sourceFixture.runtimeIdentityPath,
        sourceExpectation: sourceFixture.sourceExpectation,
      }),
      /bounded regular file/u,
    );

    const outputFixture = await sealPilot(t);
    const pages = new Map(
      [...outputFixture.responses].map(([fullname, response]) => [
        fullname,
        localPage(response),
      ]),
    );
    const rpc = await startRpcServer(t, pages);
    const outputDir = comparisonOutputDir(outputFixture, "output-fifo-output");
    await runXmlrpcPilotLocalComparison({
      outputDir,
      pilotRoot: outputFixture.pilotRoot,
      rpcUrl: rpc.rpcUrl,
      runtimeIdentityPath: outputFixture.runtimeIdentityPath,
      sourceExpectation: outputFixture.sourceExpectation,
    });
    const rowsPath = path.join(outputDir, "local-comparison.jsonl");
    await fs.unlink(rowsPath);
    await execFileAsync("mkfifo", [rowsPath]);
    await assert.rejects(
      runXmlrpcPilotLocalComparison({
        outputDir,
        pilotRoot: outputFixture.pilotRoot,
        rpcUrl: rpc.rpcUrl,
        runtimeIdentityPath: outputFixture.runtimeIdentityPath,
        sourceExpectation: outputFixture.sourceExpectation,
      }),
      /expected private regular file/u,
    );
  },
);

test("an existing comparison output directory must be private to the current user", async (t) => {
  const fixture = await sealPilot(t);
  const pages = new Map(
    [...fixture.responses].map(([fullname, response]) => [
      fullname,
      localPage(response),
    ]),
  );
  const rpc = await startRpcServer(t, pages);
  const outputDir = comparisonOutputDir(fixture, "shared-output");
  await fs.mkdir(outputDir, { mode: 0o700 });
  await fs.chmod(outputDir, 0o777);
  await assert.rejects(
    runXmlrpcPilotLocalComparison({
      outputDir,
      pilotRoot: fixture.pilotRoot,
      rpcUrl: rpc.rpcUrl,
      runtimeIdentityPath: fixture.runtimeIdentityPath,
      sourceExpectation: fixture.sourceExpectation,
    }),
    /must have mode 700/u,
  );
  assert.deepEqual(await fs.readdir(outputDir), []);
  assert.equal(rpc.calls.length, 0);
});

test("CLI and local RPC boundaries reject unsupported paths and endpoints", () => {
  assert.throws(
    () =>
      parseArgs([
        "--pilot-root",
        "pilot",
        "--runtime-identity",
        "/runtime",
        "--rpc-url",
        "https://127.0.0.1/jsonrpc",
        "--output-dir",
        "/out",
      ]),
    /must be an absolute path/u,
  );
  assert.throws(
    () => new LocalPageReadClient({ rpcUrl: "https://127.0.0.1/jsonrpc" }),
    /loopback/u,
  );
  assert.throws(
    () => new LocalPageReadClient({ rpcUrl: "http://example.test/jsonrpc" }),
    /loopback/u,
  );
  assert.deepEqual(
    parseArgs([
      "--pilot-root",
      "/pilot",
      "--runtime-identity",
      "/runtime",
      "--rpc-url",
      "http://127.0.0.1:2747/jsonrpc",
      "--output-dir",
      "/out",
      "--timeout-ms",
      "100",
    ]).timeoutMs,
    100,
  );
});

test("local page reads reject stale JSON-RPC envelopes and bound a body that stalls after headers", async () => {
  const wrongEnvelope = new LocalPageReadClient({
    fetchImpl: async () =>
      new Response(
        JSON.stringify({ id: 99, jsonrpc: "2.0", result: { site_id: 42 } }),
        { headers: { "content-type": "application/json" }, status: 200 },
      ),
    rpcUrl: "http://127.0.0.1:2747/jsonrpc",
  });
  await assert.rejects(
    wrongEnvelope.siteId(),
    (error) => error.code === "rpc_envelope",
  );

  const stalled = new LocalPageReadClient({
    fetchImpl: async (_url, options) => {
      const body = new ReadableStream({
        start(controller) {
          options.signal.addEventListener("abort", () =>
            controller.error(new DOMException("aborted", "AbortError")),
          );
        },
      });
      return new Response(body, {
        headers: { "content-type": "application/json" },
        status: 200,
      });
    },
    rpcUrl: "http://127.0.0.1:2747/jsonrpc",
    timeoutMs: 10,
  });
  await assert.rejects(stalled.siteId(), (error) => error.code === "timeout");
});
