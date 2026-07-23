import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import {
  materializeExactCoordinator,
  WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS,
} from "../scripts/run-wikidot-xmlrpc-acquisition.mjs";
import { verifyMaterializedDescriptor } from "../scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs";
import {
  assertDistinctOutputDestinations,
  assertPinnedPilotWorkerIdentity,
  capturePending,
  derivePilotInventory,
  expectedWorkerExitCode,
  normalizeRunnerOptions,
  partitionRunnerOptions,
  scrubWikidotCredentials,
  takeCredentialsAfterSeal,
  WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS,
  runAcquisition,
  usage,
  WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY,
} from "../src/wikidot-xmlrpc-acquisition-runner.mjs";
import {
  createAcquisitionFixture,
  createXmlrpcCampaignFixture,
  responseFor,
} from "./wikidot-xmlrpc-acquisition-fixtures.mjs";

function rawRows(count) {
  return Array.from({ length: count }, (_, index) => ({
    attachments: [],
    file_path: `/private/source/scp-${173 + index}/source.wikidot.txt`,
    fullname: `scp-${173 + index}`,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    required_browser: false,
    revisions: index + 1,
    source_branch: "en",
    source_entity_id: `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
    source_sha256: sha256Hex(`source-${index}\n`),
    source_site: "scp-wiki",
    updated_at: `2026-07-${String(index + 1).padStart(2, "0")}T12:34:56+00:00`,
  }));
}

test("runner options are partitioned into immutable workflow phase inputs", () => {
  const phases = partitionRunnerOptions(normalizeRunnerOptions(runnerOptions()));

  assert.equal(Object.isFrozen(phases), true);
  assert.equal(Object.isFrozen(phases.inventory), true);
  assert.deepEqual(Object.keys(phases).sort(), [
    "campaign",
    "inventory",
    "launch",
    "outputs",
    "runtime",
    "source",
    "storage",
  ]);
  assert.equal(phases.inventory.selectionCount, 128);
  assert.equal(phases.campaign.principalId, 5700026);
});

function summaryFor(rows, manifestBytes) {
  return Buffer.from(
    `${stableStringify({
      attachment_count: 0,
      attachment_page_count: 0,
      first_fullname: rows[0].fullname,
      last_fullname: rows.at(-1).fullname,
      manifest_sha256: sha256Hex(manifestBytes),
      parent_count: 0,
      required_browser_count: 0,
      row_count: rows.length,
      source_branches: ["en"],
      source_browser_visibility_counts: {},
      source_required_actor_count: 0,
      source_sites: ["scp-wiki"],
    })}\n`,
  );
}

function fixture(count = 8) {
  const rows = rawRows(count);
  const manifestBytes = Buffer.from(
    `${rows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const summaryBytes = summaryFor(rows, manifestBytes);
  const fullInventory = buildReferenceAcquisitionInventory({
    expectedCount: rows.length,
    expectedManifestSha256: sha256Hex(manifestBytes),
    expectedSummarySha256: sha256Hex(summaryBytes),
    family: "EN",
    manifestBytes,
    shardCount: 4,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
  return { fullInventory, manifestBytes, rows, summaryBytes };
}

function runnerOptions(overrides = {}) {
  return {
    "campaign-nonce": "00000000-0000-4000-8000-000000000001",
    "capsule-parent": "/tmp/capsules",
    "expected-full-inventory-sha256": "a".repeat(64),
    "expected-manifest-sha256": "b".repeat(64),
    "expected-summary-sha256": "c".repeat(64),
    "full-inventory": "/tmp/full-inventory.json",
    "inventory-output": "/tmp/inventory.json",
    manifest: "/tmp/manifest.jsonl",
    "principal-id": "5700026",
    "result-receipt": "/tmp/result.json",
    "runtime-python": "bin/python",
    "runtime-root": "/tmp/runtime",
    "runtime-venv-config": "pyvenv.cfg",
    "runtime-version": "3.13.13",
    "selection-count": "128",
    shards: "8",
    "source-commit": "1".repeat(40),
    "source-git-dir": "/tmp/source.git",
    "source-tree": "2".repeat(40),
    store: "/tmp/store",
    summary: "/tmp/summary.json",
    "throttle-receipt": "/tmp/throttle.json",
    verdict: "/tmp/verdict.json",
    "wikijump-commit": "3".repeat(40),
    "wikijump-git-dir": "/tmp/wikijump.git",
    "wikijump-tree": "4".repeat(40),
    ...overrides,
  };
}

const REPOSITORY_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../..",
);
const LOCAL_IMPORT = /(?:\bfrom\s+|\bimport\s*\(\s*)["']([^"']+)["']/gu;
const TEST_GIT_ENVIRONMENT = Object.freeze({
  GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
  GIT_AUTHOR_EMAIL: "oracle@example.invalid",
  GIT_AUTHOR_NAME: "Oracle",
  GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
  GIT_COMMITTER_EMAIL: "oracle@example.invalid",
  GIT_COMMITTER_NAME: "Oracle",
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_PAGER: "cat",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin",
});

function git(cwd, args) {
  return execFileSync("/usr/bin/git", args, {
    cwd,
    encoding: "utf8",
    env: TEST_GIT_ENVIRONMENT,
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

function runPrivateCoordinator(entrypoint, input, descriptor) {
  const program = [
    `import { runAcquisition } from ${JSON.stringify(pathToFileURL(entrypoint).href)};`,
    `const input = ${JSON.stringify(input)};`,
    "try {",
    "  await runAcquisition(input);",
    "  process.exitCode = 0;",
    "} catch (error) {",
    "  process.stdout.write(String(error?.message ?? error) + '\\n');",
    "  process.exitCode = 7;",
    "}",
  ].join("\n");
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      ["--input-type=module", "--eval", program],
      { stdio: ["ignore", "pipe", "pipe", "ignore", "pipe"] },
    );
    const stdout = [];
    const stderr = [];
    child.once("error", reject);
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("close", (code, signal) => {
      resolve({
        code,
        signal,
        stderr: Buffer.concat(stderr).toString("utf8"),
        stdout: Buffer.concat(stdout).toString("utf8"),
      });
    });
    child.stdio[4].once("error", () => {});
    child.stdio[4].end(JSON.stringify(descriptor));
  });
}

test("coordinator closure declares every transitive local import", async () => {
  assert.deepEqual(
    WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS,
    WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS,
  );
  const declared = new Set(WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS);
  const pending = [...declared];
  const visited = new Set();
  while (pending.length !== 0) {
    const sourcePath = pending.pop();
    if (visited.has(sourcePath)) continue;
    visited.add(sourcePath);
    const filePath = path.join(REPOSITORY_ROOT, sourcePath);
    const source = await fs.readFile(filePath, "utf8");
    for (const match of source.matchAll(LOCAL_IMPORT)) {
      if (!match[1].startsWith(".")) continue;
      const importedPath = path.relative(
        REPOSITORY_ROOT,
        path.resolve(path.dirname(filePath), match[1]),
      );
      const normalized = importedPath.split(path.sep).join("/");
      assert.equal(
        declared.has(normalized),
        true,
        `${sourcePath} imports undeclared ${normalized}`,
      );
      pending.push(normalized);
    }
  }
  assert.equal(visited.size, declared.size);
});

test("bootstrap and materialized entrypoint have no pre-verification local import", async () => {
  const source = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition.mjs",
    ),
    "utf8",
  );
  const specifiers = [...source.matchAll(LOCAL_IMPORT)].map(
    (match) => match[1],
  );
  assert.equal(specifiers.length > 0, true);
  assert.equal(
    specifiers.every((specifier) => specifier.startsWith("node:")),
    true,
  );
  assert.equal(/\b(?:createRequire|eval)\s*\(/u.test(source), false);
  const entrypoint = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs",
    ),
    "utf8",
  );
  const entrypointSpecifiers = [...entrypoint.matchAll(LOCAL_IMPORT)].map(
    (match) => match[1],
  );
  assert.equal(
    entrypointSpecifiers.every((specifier) => specifier.startsWith("node:")),
    true,
  );
  assert.equal(
    entrypoint.indexOf(
      "await verifyDescriptorAgainstGit(descriptor, identity);",
    ) < entrypoint.indexOf("const bootstrap = await import("),
    true,
  );
  assert.equal(
    entrypoint.indexOf(
      "await verifyDescriptorAgainstGit(descriptor, identity);",
    ) < entrypoint.indexOf("const coordinator = await import("),
    true,
  );
  assert.match(entrypoint, /createReadStream\(null, \{/u);
  assert.doesNotMatch(entrypoint, /readFile\("\/proc\/self\/fd\/3"\)/u);
  const coordinator = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    ),
    "utf8",
  );
  assert.match(coordinator, /MATERIALIZED_DESCRIPTOR_FD = 4/u);
  assert.match(coordinator, /createReadStream\(null, \{/u);
  assert.doesNotMatch(coordinator, /readFile\("\/proc\/self\/fd\/\$\{/u);
});

test("bootstrap materializes the complete exact coordinator closure privately", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-bootstrap-"));
  const checkout = path.join(root, "checkout");
  const capsules = path.join(root, "capsules");
  await fs.mkdir(checkout);
  await fs.mkdir(capsules, { mode: 0o700 });
  await fs.chmod(capsules, 0o700);
  t.after(() => fs.rm(root, { force: true, recursive: true }));
  for (const sourcePath of WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS) {
    const destination = path.join(checkout, sourcePath);
    await fs.mkdir(path.dirname(destination), { recursive: true });
    await fs.copyFile(path.join(REPOSITORY_ROOT, sourcePath), destination);
  }
  git(checkout, ["init", "--initial-branch=main"]);
  git(checkout, ["add", "."]);
  git(checkout, ["commit", "-m", "coordinator"]);
  const commit = git(checkout, ["rev-parse", "HEAD"]);
  const tree = git(checkout, ["rev-parse", "HEAD^{tree}"]);
  await fs.writeFile(
    path.join(
      checkout,
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    ),
    "throw new Error('mutable checkout must not execute');\n",
  );
  const coordinator = await materializeExactCoordinator({
    capsuleParent: capsules,
    wikijumpCommit: commit,
    wikijumpGitDirectory: path.join(checkout, ".git"),
    wikijumpTree: tree,
  });
  t.after(() =>
    fs.rm(coordinator.root, { force: true, maxRetries: 2, recursive: true }),
  );
  assert.equal((await fs.stat(coordinator.root)).mode & 0o777, 0o700);
  for (const sourcePath of WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS) {
    const [expected, actual, stat] = await Promise.all([
      fs.readFile(path.join(REPOSITORY_ROOT, sourcePath)),
      fs.readFile(path.join(coordinator.root, sourcePath)),
      fs.stat(path.join(coordinator.root, sourcePath)),
    ]);
    assert.deepEqual(actual, expected);
    assert.equal(stat.mode & 0o777, 0o400);
  }
  const descriptor = {
    coordinator_path:
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    entrypoint_path:
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs",
    files: coordinator.files,
    materialization_root: coordinator.root,
    schema: "wikijump_full_parity.wikidot_xmlrpc_materialized_launch.v1",
    wikijump_commit: commit,
    wikijump_tree: tree,
  };
  const identityArguments = [
    "--wikijump-commit",
    commit,
    "--wikijump-git-dir",
    path.join(checkout, ".git"),
    "--wikijump-tree",
    tree,
  ];
  await assert.doesNotReject(
    verifyMaterializedDescriptor(
      descriptor,
      identityArguments,
      coordinator.entrypoint,
    ),
  );
  const forged = {
    ...descriptor,
    files: descriptor.files.map((file, index, files) =>
      index === 0 ? { ...file, blob_oid: files[1].blob_oid } : file,
    ),
  };
  await assert.rejects(
    verifyMaterializedDescriptor(
      forged,
      identityArguments,
      coordinator.entrypoint,
    ),
    /descriptor_git_mismatch/u,
  );
  const mutableCoordinator = path.join(
    coordinator.root,
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
  );
  const mutableDependency = path.join(
    coordinator.root,
    "install/local/wikidot-verification/src/atomic-no-replace.mjs",
  );
  await fs.chmod(mutableDependency, 0o600);
  await fs.appendFile(mutableDependency, "// mutable omitted dependency\n");
  await fs.chmod(mutableDependency, 0o400);
  const incompleteDescriptor = {
    ...descriptor,
    files: descriptor.files.filter(
      (file) =>
        file.path !==
        "install/local/wikidot-verification/src/atomic-no-replace.mjs",
    ),
  };
  const omittedDependency = await runPrivateCoordinator(
    mutableCoordinator,
    runnerOptions({
      "wikijump-commit": commit,
      "wikijump-git-dir": path.join(checkout, ".git"),
      "wikijump-tree": tree,
    }),
    incompleteDescriptor,
  );
  assert.equal(omittedDependency.signal, null);
  assert.equal(omittedDependency.code, 7, omittedDependency.stderr);
  assert.match(
    omittedDependency.stdout,
    /materialized_launch_descriptor_invalid/u,
  );
  await fs.chmod(mutableCoordinator, 0o600);
  await fs.appendFile(mutableCoordinator, "// mutable private copy\n");
  await fs.chmod(mutableCoordinator, 0o400);
  const privateCopy = await runPrivateCoordinator(
    mutableCoordinator,
    runnerOptions({
      "wikijump-commit": commit,
      "wikijump-git-dir": path.join(checkout, ".git"),
      "wikijump-tree": tree,
    }),
    descriptor,
  );
  assert.equal(privateCopy.signal, null);
  assert.equal(privateCopy.code, 7, privateCopy.stderr);
  assert.match(privateCopy.stdout, /materialized_launch_private_file_invalid/u);
});

test("pilot selection is deterministic, derives a complete inventory, and omits source host paths", () => {
  const state = fixture();
  const options = {
    expectedFullInventorySha256: state.fullInventory.identity.sha256,
    expectedManifestSha256: sha256Hex(state.manifestBytes),
    expectedSummarySha256: sha256Hex(state.summaryBytes),
    fullInventory: state.fullInventory,
    manifestBytes: state.manifestBytes,
    selectionCount: 3,
    shardCount: 2,
    summaryBytes: state.summaryBytes,
  };
  const first = derivePilotInventory(options);
  const repeated = derivePilotInventory(options);

  assert.equal(first.inventory.rows.length, 3);
  assert.equal(
    first.inventory.identity.sha256,
    repeated.inventory.identity.sha256,
  );
  assert.deepEqual(first.inventoryBytes, repeated.inventoryBytes);
  assert.deepEqual(
    first.inventory.rows.map((row) => row.ordinal),
    [0, 1, 2],
  );
  assert.deepEqual(
    first.inventory.rows.map((row) => row.fullname),
    [...first.inventory.rows.map((row) => row.fullname)].sort(),
  );
  assert.equal(
    first.inventoryBytes.includes(Buffer.from("/private/source", "utf8")),
    false,
  );
  assert.equal(first.selection.selected_count, 3);
  assert.equal(
    first.selection.full_inventory_sha256,
    state.fullInventory.identity.sha256,
  );
});

test("pilot selection rejects a raw manifest that no longer matches the verified source capsule", () => {
  const state = fixture();
  const changedRows = state.rows.map((row) => ({
    ...row,
    source_sha256: "f".repeat(64),
  }));
  const changedManifest = Buffer.from(
    `${changedRows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const changedSummary = summaryFor(changedRows, changedManifest);
  assert.throws(
    () =>
      derivePilotInventory({
        expectedFullInventorySha256: state.fullInventory.identity.sha256,
        expectedManifestSha256: sha256Hex(changedManifest),
        expectedSummarySha256: sha256Hex(changedSummary),
        fullInventory: state.fullInventory,
        manifestBytes: changedManifest,
        selectionCount: 2,
        shardCount: 2,
        summaryBytes: changedSummary,
      }),
    /full_inventory_authority_invalid/u,
  );
});

test("runner usage declares the sealed-receipt and exact-identity inputs", () => {
  const text = usage();
  for (const option of [
    "--expected-full-inventory-sha256",
    "--throttle-receipt",
    "--result-receipt",
    "--wikijump-commit",
    "--source-commit",
  ]) {
    assert.match(text, new RegExp(option, "u"));
  }
});

test("runner option parsing accepts canonical UUID campaign nonces", () => {
  const parsed = normalizeRunnerOptions(runnerOptions());
  assert.equal(parsed.campaignNonce, "00000000-0000-4000-8000-000000000001");
  assert.throws(
    () =>
      normalizeRunnerOptions(
        runnerOptions({ "campaign-nonce": "00000000-0000-4000-000000000001" }),
      ),
    /campaign_nonce_invalid/u,
  );
});

test("pilot worker identity is pinned before the coordinator can launch", () => {
  const pinned = normalizeRunnerOptions(
    runnerOptions({
      "source-commit": WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.commit,
      "source-tree": WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.tree,
    }),
  );
  assert.doesNotThrow(() => assertPinnedPilotWorkerIdentity(partitionRunnerOptions(pinned).source));
  assert.throws(
    () =>
      assertPinnedPilotWorkerIdentity(
        partitionRunnerOptions(normalizeRunnerOptions(runnerOptions())).source,
      ),
    /pilot_worker_identity_invalid/u,
  );
});

test("runner rejects output destination aliases before a throttle can be sealed", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-output-"));
  t.after(() => fs.rm(root, { force: true, recursive: true }));
  const options = partitionRunnerOptions(normalizeRunnerOptions(
    runnerOptions({
      "inventory-output": path.join(root, "inventory.json"),
      "result-receipt": path.join(root, "result.json"),
      "throttle-receipt": path.join(root, "throttle.json"),
      verdict: path.join(root, "verdict.json"),
    }),
  )).outputs;
  await assert.doesNotReject(assertDistinctOutputDestinations(options));
  await assert.rejects(
    assertDistinctOutputDestinations({
      ...options,
      resultReceipt: options.throttleReceipt,
    }),
    /output_destinations_alias/u,
  );
  await fs.writeFile(options.inventoryOutput, "receipt\n");
  await fs.link(options.inventoryOutput, options.resultReceipt);
  await assert.rejects(
    assertDistinctOutputDestinations(options),
    /output_destinations_alias/u,
  );
});

test("credentials are removed at the seal boundary without entering a receipt", () => {
  const environment = Object.assign(Object.create(null), {
    WIKIDOT_API_KEY: "test-api-key",
    WIKIDOT_APP_NAME: "test-application",
  });
  assert.deepEqual(takeCredentialsAfterSeal(environment), {
    apiKey: "test-api-key",
    appName: "test-application",
  });
  assert.equal(Object.hasOwn(environment, "WIKIDOT_API_KEY"), false);
  assert.equal(Object.hasOwn(environment, "WIKIDOT_APP_NAME"), false);
  environment.WIKIDOT_API_KEY = "another-test-api-key";
  environment.WIKIDOT_APP_NAME = "another-test-application";
  scrubWikidotCredentials(environment);
  assert.deepEqual(environment, Object.create(null));
});

test("coordinator rejects a direct mutable-checkout launch before opening inputs", async () => {
  await assert.rejects(
    runAcquisition(runnerOptions()),
    /materialized_launch_descriptor_unavailable/u,
  );
});

test("retryable and internal worker outcomes require their declared worker exits", () => {
  assert.equal(
    expectedWorkerExitCode({ code: "transport_exhausted", retryable: true }),
    75,
  );
  assert.equal(
    expectedWorkerExitCode({ code: "worker_internal_error", retryable: false }),
    70,
  );
  assert.equal(
    expectedWorkerExitCode({ code: "wikidot_forbidden", retryable: false }),
    null,
  );
});

test("offline worker capture publishes semantic completion without exposing a launch capability", async (t) => {
  const state = await createAcquisitionFixture(t, 1);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const calls = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture(ordinal, fullname) {
        calls.push({ fullname, ordinal });
        return { ok: true, response: responseFor(state, ordinal) };
      },
      async expectExit() {
        assert.fail("successful worker must remain available until clean EOF");
      },
    },
  });
  assert.deepEqual(calls, [{ fullname: "scp-173", ordinal: 0 }]);
  assert.deepEqual(outcome, {
    failure: null,
    status: "complete",
    workerExited: false,
  });
  assert.equal((await semantic.planResume()).pending.length, 0);
});

test("exact deleted worker results become tombstones and later captures continue", async (t) => {
  const state = await createAcquisitionFixture(t, 3);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const calls = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture(ordinal, fullname) {
        calls.push({ fullname, ordinal });
        if (ordinal === 1) {
          return {
            code: "wikidot_deleted",
            ok: false,
            ordinal,
            retryable: false,
          };
        }
        return { ok: true, response: responseFor(state, ordinal) };
      },
      async expectExit() {
        assert.fail("deleted page does not terminate a healthy worker");
      },
    },
  });
  assert.deepEqual(calls, [
    { fullname: "scp-173", ordinal: 0 },
    { fullname: "scp-174", ordinal: 1 },
    { fullname: "scp-175", ordinal: 2 },
  ]);
  assert.deepEqual(outcome, {
    failure: null,
    status: "complete",
    workerExited: false,
  });
  const plan = await semantic.planResume();
  assert.equal(plan.pending.length, 0);
  assert.equal(plan.complete.length, 3);
  assert.equal((await semantic.resolve({ ordinal: 0 })).kind, "live");
  const deleted = await semantic.resolve({ ordinal: 1 });
  assert.equal(deleted.kind, "deleted");
  assert.equal(deleted.tombstone.classification, "wikidot_deleted");
  assert.equal("response" in deleted, false);
  assert.equal((await semantic.resolve({ ordinal: 2 })).kind, "live");
});

test("forbidden and unclassified worker results remain terminal and pending", async (t) => {
  for (const code of ["wikidot_forbidden", "wikidot_fault_unclassified"]) {
    await t.test(code, async (t) => {
      const state = await createAcquisitionFixture(t, 2);
      const { semantic } = await createXmlrpcCampaignFixture(state);
      const calls = [];
      const outcome = await capturePending({
        completions: semantic,
        context: state.context,
        store: state.store,
        worker: {
          async capture(ordinal) {
            calls.push(ordinal);
            return { code, ok: false, ordinal, retryable: false };
          },
          async expectExit() {
            assert.fail("terminal fault should not require worker exit");
          },
        },
      });
      assert.deepEqual(calls, [0]);
      assert.equal(outcome.status, "terminal_stop");
      assert.equal(outcome.failure.code, code);
      assert.equal((await semantic.planResume()).pending.length, 2);
      assert.equal(await semantic.resolve({ ordinal: 0 }), null);
    });
  }
});

test("offline retryable capture persists the failed attempt and requires exit 75", async (t) => {
  const state = await createAcquisitionFixture(t, 1);
  const { semantic } = await createXmlrpcCampaignFixture(state);
  const exits = [];
  const outcome = await capturePending({
    completions: semantic,
    context: state.context,
    store: state.store,
    worker: {
      async capture() {
        return {
          code: "transport_exhausted",
          ok: false,
          ordinal: 0,
          retryable: true,
        };
      },
      async expectExit(code) {
        exits.push(code);
      },
    },
  });
  assert.deepEqual(exits, [75]);
  assert.equal(outcome.status, "retryable_stop");
  assert.equal(outcome.failure.code, "transport_exhausted");
  assert.equal(outcome.failure.retryable, true);
  assert.equal(outcome.failure.attempt.algorithm, "sha256");
  await state.store.verifyObject(outcome.failure.attempt);
  assert.equal((await semantic.planResume()).pending.length, 1);
});
