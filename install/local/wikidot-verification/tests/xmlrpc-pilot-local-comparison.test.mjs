import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { promisify } from "node:util";
import test from "node:test";

import { stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  LocalPageReadClient,
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
  runXmlrpcPilotLocalComparison,
} from "../src/xmlrpc-pilot-local-comparison.mjs";
import { publishWikidotXmlrpcAcquisitionVerdict } from "../src/wikidot-xmlrpc-acquisition-verdict.mjs";
import {
  completeDeletedXmlrpcOrdinal,
  completeXmlrpcOrdinal,
  createAcquisitionFixture,
  createXmlrpcCampaignFixture,
  responseFor,
} from "./wikidot-xmlrpc-acquisition-fixtures.mjs";
import { parseArgs } from "../scripts/compare-xmlrpc-pilot-local.mjs";

const execFileAsync = promisify(execFile);

function digest(bytes) {
  return {
    bytes: bytes.byteLength,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  };
}

async function writeJson(filePath, value) {
  await fs.writeFile(filePath, `${stableStringify(value)}\n`, "utf8");
}

function sealedManifestRecord(semantic, row) {
  const common = {
    fixture_id: row.fixtureId,
    fullname: row.fullname,
    inventory_sha256: semantic.target.inventory.sha256,
    ordinal: row.ordinal,
    schema: XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
    semantic_row_sha256: row.semanticRowSha256,
    source_entity_id: row.sourceEntityId,
    work_identity_sha256: semantic.target.work_identity.sha256,
  };
  if (semantic.kind === "deleted") {
    return {
      ...common,
      reference: {
        kind: "wikidot_deleted",
        tombstone_sha256: digest(
          Buffer.from(stableStringify(semantic.tombstone), "utf8"),
        ).sha256,
      },
    };
  }
  return {
    ...common,
    reference: {
      content_sha256: semantic.observation.observed.content_sha256,
      html_sha256: semantic.observation.observed.html_sha256,
      kind: "live",
      response: semantic.observation.response,
      revisions: semantic.observation.observed.revisions,
      updated_at: semantic.observation.observed.updated_at,
    },
  };
}

function sealedManifestIdentity(semantics, rows) {
  const bytes = Buffer.from(
    `${semantics
      .map((semantic, ordinal) =>
        stableStringify(sealedManifestRecord(semantic, rows[ordinal])),
      )
      .join("\n")}\n`,
    "utf8",
  );
  return digest(bytes);
}

function runtimeIdentity() {
  return {
    artifact_key: "candidate-local-pilot-v1",
    deepwell_binary_or_image_sha256: "a".repeat(64),
    features: [],
    ftml_sha: "b".repeat(40),
    framerail_assets_sha256: "c".repeat(64),
    profile: "dev",
    render_run_id: "xmlrpc-pilot-test",
    runtime_config_sha256: "d".repeat(64),
    rustc_vv:
      "rustc 1.0\nbinary: rustc\ncommit-hash: x\ncommit-date: x\nhost: x\nrelease: x\nLLVM version: x",
    schema: "wikijump_full_parity.local_browser_runtime_identity.v1",
    wikijump_sha: "e".repeat(40),
  };
}

async function sealPilot(t, { count = 3, deletedOrdinals = [count - 1] } = {}) {
  const state = await createAcquisitionFixture(t, count);
  const { campaign, implementation } = await createXmlrpcCampaignFixture(state);
  const responses = new Map();
  const semantics = [];
  for (let ordinal = 0; ordinal < count; ordinal += 1) {
    if (deletedOrdinals.includes(ordinal)) {
      semantics.push(
        await completeDeletedXmlrpcOrdinal(state, campaign, ordinal),
      );
    } else {
      const response = responseFor(state, ordinal);
      responses.set(response.fullname, response);
      semantics.push(await completeXmlrpcOrdinal(state, campaign, ordinal));
    }
  }
  await state.semantic.close();
  state.semantic = undefined;
  const throttleConfig = (
    await state.store.putBytes(Buffer.from('{"throttle":"sealed"}\n'))
  ).object;
  const verdictPath = path.join(state.receiptDirectory, "verdict.json");
  const publication = await publishWikidotXmlrpcAcquisitionVerdict(
    verdictPath,
    {
      campaignReference: campaign.reference,
      context: state.context,
      store: state.store,
    },
  );
  const artifactKey = `wikidot-xmlrpc-en-${count}-${state.inventory.identity.sha256}-${implementation.object.sha256}`;
  await writeJson(
    path.join(state.receiptDirectory, "inventory.json"),
    state.inventory,
  );
  await writeJson(path.join(state.receiptDirectory, "result.json"), {
    artifact_key: artifactKey,
    campaign: campaign.reference,
    completed: count,
    failure: null,
    implementation: implementation.object,
    inventory: { row_count: count, sha256: state.inventory.identity.sha256 },
    outcome: "pass",
    schema: "wikijump_full_parity.wikidot_xmlrpc_acquisition_run.v1",
    throttle: throttleConfig,
    verdict: digest(publication.bytes),
  });
  await writeJson(path.join(state.receiptDirectory, "throttle.json"), {
    artifact_key: artifactKey,
    campaign: campaign.reference,
    implementation: implementation.object,
    inventory_sha256: state.inventory.identity.sha256,
    schema: "wikijump_full_parity.wikidot_xmlrpc_throttle_receipt.v1",
    status: "sealed",
    throttle_config: throttleConfig,
  });
  const inputReceipts = Object.freeze({
    inventory: digest(
      await fs.readFile(path.join(state.receiptDirectory, "inventory.json")),
    ),
    result: digest(
      await fs.readFile(path.join(state.receiptDirectory, "result.json")),
    ),
    throttle: digest(
      await fs.readFile(path.join(state.receiptDirectory, "throttle.json")),
    ),
    verdict: digest(
      await fs.readFile(path.join(state.receiptDirectory, "verdict.json")),
    ),
  });
  const verifiedPilotManifest = sealedManifestIdentity(
    semantics,
    state.context.rows,
  );
  await state.store.close();
  state.store = undefined;
  const pilotRoot = path.dirname(state.root);
  const runtimeIdentityPath = path.join(pilotRoot, "runtime-identity.json");
  await writeJson(runtimeIdentityPath, runtimeIdentity());
  return {
    pilotRoot,
    responses,
    runtimeIdentityPath,
    sourceExpectation: {
      acquisition_artifact_key: artifactKey,
      campaign: campaign.reference,
      implementation: implementation.object,
      input_receipts: inputReceipts,
      inventory_sha256: state.inventory.identity.sha256,
      row_count: count,
      verified_pilot_manifest: verifiedPilotManifest,
    },
  };
}

function localPage(response, overrides = {}) {
  return {
    compiled_body_html: response.html,
    compiled_body_styles: [],
    page_revision_count: response.revisions,
    page_updated_at: response.updated_at,
    site_id: 42,
    slug: response.fullname,
    wikitext: response.content,
    ...overrides,
  };
}

async function startRpcServer(t, pages, { siteError = false } = {}) {
  const calls = [];
  const server = http.createServer((request, response) => {
    let body = "";
    request.setEncoding("utf8");
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const payload = JSON.parse(body);
      calls.push({
        headers: request.headers,
        method: payload.method,
        params: payload.params,
      });
      response.setHeader("content-type", "application/json");
      if (payload.method === "site_get") {
        response.end(
          JSON.stringify(
            siteError
              ? {
                  jsonrpc: "2.0",
                  id: payload.id,
                  error: { code: -1, message: "unavailable" },
                }
              : { jsonrpc: "2.0", id: payload.id, result: { site_id: 42 } },
          ),
        );
        return;
      }
      if (payload.method !== "page_get") {
        response.end(
          JSON.stringify({
            jsonrpc: "2.0",
            id: payload.id,
            error: { code: -1, message: "unexpected" },
          }),
        );
        return;
      }
      response.end(
        JSON.stringify({
          jsonrpc: "2.0",
          id: payload.id,
          result: pages.get(payload.params.page) ?? null,
        }),
      );
    });
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(
    () =>
      new Promise((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  );
  return {
    calls,
    rpcUrl: `http://127.0.0.1:${server.address().port}/jsonrpc`,
  };
}

function readJsonl(bytes) {
  return bytes
    .toString("utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}

function comparisonOutputDir(fixture, label) {
  return path.join(
    path.dirname(fixture.pilotRoot),
    `${path.basename(fixture.pilotRoot)}-${label}`,
  );
}

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
