import {execFile} from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import {promisify} from "node:util";

import {stableStringify} from "../../src/corpus-import-manifest.mjs";
import {XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA} from "../../src/xmlrpc-pilot-local-comparison.mjs";
import {publishWikidotXmlrpcAcquisitionVerdict} from "../../src/wikidot-xmlrpc-acquisition-verdict.mjs";
import {
  completeDeletedXmlrpcOrdinal,
  completeXmlrpcOrdinal,
  createAcquisitionFixture,
  createXmlrpcCampaignFixture,
  responseFor,
} from "../wikidot-xmlrpc-acquisition-fixtures.mjs";

export const execFileAsync = promisify(execFile);

export function digest(bytes) {
  return {
    bytes: bytes.byteLength,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  };
}

export async function writeJson(filePath, value) {
  await fs.writeFile(filePath, `${stableStringify(value)}\n`, "utf8");
}

export function sealedManifestRecord(semantic, row) {
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

export function sealedManifestIdentity(semantics, rows) {
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

export function runtimeIdentity() {
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

export async function sealPilot(t, { count = 3, deletedOrdinals = [count - 1] } = {}) {
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

export function localPage(response, overrides = {}) {
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

export async function startRpcServer(t, pages, { siteError = false } = {}) {
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

export function readJsonl(bytes) {
  return bytes
    .toString("utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}

export function comparisonOutputDir(fixture, label) {
  return path.join(
    path.dirname(fixture.pilotRoot),
    `${path.basename(fixture.pilotRoot)}-${label}`,
  );
}
