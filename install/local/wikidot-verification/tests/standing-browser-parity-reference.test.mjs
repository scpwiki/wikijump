import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { canaryForUrl } from "../src/standing-browser-canaries.mjs";
import {
  buildLiveReferenceLedger,
  loadSealedLiveReference,
} from "../src/standing-browser-parity-reference.mjs";

const viewport = { width: 1366, height: 900 };
const thresholds = {
  geometry_position_px: 8,
  geometry_size_px: 12,
  image_count_delta: 0,
  dom_multiset_distance_ratio: 0.15,
};

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

async function screenshotArtifacts(directory) {
  const entries = [
    ["first.png", "first", false],
    ["viewport.png", "viewport", false],
    ["full.png", "full", true],
  ];
  await Promise.all(
    entries.map(([file, contents]) =>
      fs.writeFile(path.join(directory, file), contents),
    ),
  );
  return Object.fromEntries(
    entries.map(([file, contents, fullPage]) => [
      file,
      { path: file, sha256: sha256(contents), full_page: fullPage },
    ]),
  );
}

function policy() {
  return {
    schema: "wikijump.standing_browser_live_completion_policy.v1",
    status: "sealed",
    policy_version: "2026-07-20.1",
    allowed_external_failures: [],
  };
}

function requestGate() {
  return {
    schema: "wikijump_full_parity.browser_request_gate.v1",
    interval_ms: 4_000,
    enforcement_failed: false,
    public_requests: 1,
    config_sha256: "a".repeat(64),
  };
}

function probesFor(pair) {
  return canaryForUrl(pair.live_url).presence_probes.map((requirement) => {
    const pseudoLayout = requirement.pseudo_layout
      ? {
          status: "captured",
          node_present: true,
          layout_present: true,
          painted_bounds: { x: 0, y: 0, width: 100, height: 20 },
          visible_bounds: { x: 0, y: 0, width: 100, height: 20 },
          visible_area_ratio: 1,
          descendant_text: requirement.pseudo_layout.require_descendant_text
            ? "generated text"
            : "",
          computed_style: {
            content: requirement.pseudo_layout.require_generated_content
              ? '"generated text"'
              : "none",
            "background-image": requirement.pseudo_layout
              .require_background_image
              ? "url(https://cdn.example/logo.png)"
              : "none",
          },
        }
      : undefined;
    return {
      id: requirement.id,
      selector: requirement.selector,
      pseudo: requirement.pseudo ?? null,
      count: requirement.minimum_count ?? 1,
      rendered_count: requirement.require_rendered
        ? (requirement.minimum_count ?? 1)
        : 0,
      style: {
        content: pseudoLayout?.computed_style.content,
        "background-image": pseudoLayout?.computed_style["background-image"],
      },
      ...(pseudoLayout ? { pseudo_layout: pseudoLayout } : {}),
    };
  });
}

function capture(pair, artifacts, overrides = {}) {
  const probes = probesFor(pair);
  return {
    schema: "wikijump_local_lab.standing_browser_parity_capture.v2",
    captured_at: "2026-07-20T00:00:00.000Z",
    input_url: pair.live_url,
    final_url: pair.live_url,
    navigation_status: 200,
    failures: [],
    broken_images: [],
    first_paint: {
      document: {
        phase: "domcontentloaded_immediate_observation",
        custom_properties: {},
        presence_probes: probes,
        geometry: {},
      },
      screenshot: artifacts["first.png"],
    },
    document: {
      phase: "settled",
      presence_probes: probes,
      geometry: {},
      resource_completion: {
        status: "complete",
        load_ready_state: "complete",
        font_status: "loaded",
        image_count: 0,
        incomplete_image_count: 0,
      },
    },
    settled_viewport_screenshot: artifacts["viewport.png"],
    screenshot: artifacts["full.png"],
    ...overrides,
  };
}

async function writeReference(directory, pair, live, rawPolicy = policy()) {
  const policyFile = path.join(directory, "completion-policy.json");
  await fs.writeFile(policyFile, `${JSON.stringify(rawPolicy)}\n`, {
    mode: 0o600,
  });
  const policyBytes = await fs.readFile(policyFile);
  const policySha256 = sha256(policyBytes);
  const reference = buildLiveReferenceLedger({
    records: [{ input: pair, live }],
    viewport,
    thresholds,
    policy: rawPolicy,
    policySha256,
    browserEnvironment: {
      engine: "chromium",
      version: "fixture",
      executable_sha256: "b".repeat(64),
    },
    requestGate: requestGate(),
    generatedAt: "2026-07-20T00:00:00.000Z",
  });
  const file = path.join(directory, "standing-browser-live-reference.json");
  await fs.writeFile(file, `${JSON.stringify(reference)}\n`, { mode: 0o600 });
  return {
    file,
    policyFile,
    policy: rawPolicy,
    policySha256,
    sha256: sha256(await fs.readFile(file)),
  };
}

test("sealed live reference binds the policy, capture contract, and screenshot bytes", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
    live_url: "https://scp-wiki.wikidot.com/theme:basalt",
  };
  const artifacts = await screenshotArtifacts(directory);
  const stored = await writeReference(
    directory,
    pair,
    capture(pair, artifacts),
  );
  const candidatePair = {
    ...pair,
    local_url: "https://scp-wiki.wikijump.localhost:24443/theme:basalt",
  };
  const loaded = await loadSealedLiveReference({
    filePath: stored.file,
    expectedSha256: stored.sha256,
    pairs: [candidatePair],
    viewport,
    thresholds,
    policy: stored.policy,
    policySha256: stored.policySha256,
    policyFilePath: stored.policyFile,
  });
  assert.equal(loaded.records.length, 1);
  await fs.writeFile(path.join(directory, "first.png"), "tampered");
  await assert.rejects(
    loadSealedLiveReference({
      filePath: stored.file,
      expectedSha256: stored.sha256,
      pairs: [pair],
      viewport,
      thresholds,
      policy: stored.policy,
      policySha256: stored.policySha256,
      policyFilePath: stored.policyFile,
    }),
    /screenshot SHA-256 mismatch/u,
  );
});

test("live reference refuses an immediate theme mismatch instead of treating a later state as evidence", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
    live_url: "https://scp-wiki.wikidot.com/scp-9506",
  };
  const artifacts = await screenshotArtifacts(directory);
  const live = capture(pair, artifacts, {
    first_paint: {
      document: {
        phase: "domcontentloaded_immediate_observation",
        custom_properties: { "--logo": "" },
        presence_probes: [],
        geometry: {},
      },
      screenshot: artifacts["first.png"],
    },
  });
  const stored = await writeReference(directory, pair, live);
  await assert.rejects(
    loadSealedLiveReference({
      filePath: stored.file,
      expectedSha256: stored.sha256,
      pairs: [pair],
      viewport,
      thresholds,
      policy: stored.policy,
      policySha256: stored.policySha256,
      policyFilePath: stored.policyFile,
    }),
    /DOMContentLoaded theme properties/u,
  );
});

test("live reference refuses a post-settle capture relabeled as immediate evidence", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
    live_url: "https://scp-wiki.wikidot.com/theme:basalt",
  };
  const artifacts = await screenshotArtifacts(directory);
  const live = capture(pair, artifacts);
  live.first_paint.document.phase = "settled";
  const stored = await writeReference(directory, pair, live);
  await assert.rejects(
    loadSealedLiveReference({
      filePath: stored.file,
      expectedSha256: stored.sha256,
      pairs: [pair],
      viewport,
      thresholds,
      policy: stored.policy,
      policySha256: stored.policySha256,
      policyFilePath: stored.policyFile,
    }),
    /required DOMContentLoaded observation/u,
  );
});

test("live reference refuses a capture whose load, font, or image completion was not observed", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
    live_url: "https://scp-wiki.wikidot.com/theme:basalt",
  };
  const artifacts = await screenshotArtifacts(directory);
  const live = capture(pair, artifacts);
  live.document.resource_completion = { status: "timed_out" };
  const stored = await writeReference(directory, pair, live);
  await assert.rejects(
    loadSealedLiveReference({
      filePath: stored.file,
      expectedSha256: stored.sha256,
      pairs: [pair],
      viewport,
      thresholds,
      policy: stored.policy,
      policySha256: stored.policySha256,
      policyFilePath: stored.policyFile,
    }),
    /did not complete load, font, and image observation/u,
  );
});

test("live reference requires a clean 0.25 req/s gate to admit each live navigation", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
    live_url: "https://scp-wiki.wikidot.com/theme:basalt",
  };
  const artifacts = await screenshotArtifacts(directory);
  const rawPolicy = policy();
  const policyFile = path.join(directory, "completion-policy.json");
  await fs.writeFile(policyFile, `${JSON.stringify(rawPolicy)}\n`, {
    mode: 0o600,
  });
  const policySha256 = sha256(await fs.readFile(policyFile));
  assert.throws(
    () =>
      buildLiveReferenceLedger({
        records: [{ input: pair, live: capture(pair, artifacts) }],
        viewport,
        thresholds,
        policy: rawPolicy,
        policySha256,
        browserEnvironment: {
          engine: "chromium",
          version: "fixture",
          executable_sha256: "b".repeat(64),
        },
        requestGate: { ...requestGate(), public_requests: 0 },
      }),
    /did not admit every required live navigation/u,
  );
});

test("live reference refuses a broken external image unless its exact failure is allowlisted", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-live-reference-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const pair = {
    local_url: "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
    live_url: "https://scp-wiki.wikidot.com/theme:basalt",
  };
  const artifacts = await screenshotArtifacts(directory);
  const stored = await writeReference(
    directory,
    pair,
    capture(pair, artifacts, {
      broken_images: [{ src: "https://cdn.example/broken.png" }],
    }),
  );
  await assert.rejects(
    loadSealedLiveReference({
      filePath: stored.file,
      expectedSha256: stored.sha256,
      pairs: [pair],
      viewport,
      thresholds,
      policy: stored.policy,
      policySha256: stored.policySha256,
      policyFilePath: stored.policyFile,
    }),
    /unapproved broken external image/u,
  );
});
