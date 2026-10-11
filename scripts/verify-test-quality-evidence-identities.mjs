#!/usr/bin/env node
// Repository-local source identity gate. No cargo, fixtures, network, or mutable output.
// This is NOT the full test-quality or mutation acceptance verifier.
// It verifies the pinned audit snapshot against Git objects and reports working-tree staleness
// without failing ordinary implementation changes. See docs/development/test-quality-audit.md.
import {existsSync, readFileSync, realpathSync} from "node:fs";
import {dirname, isAbsolute, relative, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {
  REFRESH_COMMAND, repositoryRelative, snapshotStaleness, verifyPinnedSnapshot,
} from "./lib/test-quality-snapshot.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ledgerPath = process.env.WIKIJUMP_TEST_QUALITY_LEDGER ||
  resolve(root, "docs/development/test-quality-audit.json");
const fail = (message) => {throw new Error(message);};
const STALE_LIMIT = 10;

function repositoryFile(file, label) {
  repositoryRelative(file, label);
  const target = resolve(root, file);
  const local = relative(root, target);
  if (!local || local === ".." || local.startsWith("../") || isAbsolute(local)) {
    fail(`${label} escapes repository: ${file}`);
  }
  if (!existsSync(target)) fail(`${label} does not exist: ${file}`);
  const actual = relative(root, realpathSync(target));
  if (!actual || actual === ".." || actual.startsWith("../") || isAbsolute(actual)) {
    fail(`${label} symlink points outside repository: ${file}`);
  }
  return target;
}

export function verifyAuditIdentityLedger(ledger) {
  if (ledger?.schema !== "wikijump.test_quality_audit.v1" ||
      !Array.isArray(ledger.inputs) || !Array.isArray(ledger.owners) ||
      ledger.owners.length === 0 || !Array.isArray(ledger.records)) {
    fail("audit ledger is missing required collections");
  }
  const pinned = verifyPinnedSnapshot(root, ledger);
  const records = new Map(ledger.records.map((record) => [record.path, record]));

  const ids = new Set();
  let mutationOwners = 0;
  for (const owner of ledger.owners) {
    if (typeof owner.id !== "string" || !owner.id || ids.has(owner.id)) {
      fail(`missing or duplicate owner id: ${owner.id}`);
    }
    ids.add(owner.id);
    if (!Array.isArray(owner.files) || owner.files.length === 0 ||
        !Array.isArray(owner.anchors) || owner.anchors.length === 0) {
      fail(`owner ${owner.id} requires source files and behavioral anchors`);
    }
    for (const file of owner.files) {
      if (!records.has(file)) fail(`owner ${owner.id} references an unknown source: ${file}`);
    }
    for (const anchor of owner.anchors) {
      const separator = typeof anchor === "string" ? anchor.indexOf("#") : -1;
      if (separator <= 0 || separator === anchor.length - 1) {
        fail(`owner ${owner.id} has an invalid anchor: ${anchor}`);
      }
      const file = anchor.slice(0, separator);
      const name = anchor.slice(separator + 1);
      const source = readFileSync(repositoryFile(file, `test anchor for ${owner.id}`), "utf8");
      if (!source.includes(name)) fail(`owner ${owner.id} has an invalid or missing anchor: ${anchor}`);
    }
    if (owner.mutation_inventory_sha256) {
      if (!/^[0-9a-f]{64}$/u.test(owner.mutation_inventory_sha256)) {
        fail(`owner ${owner.id} has an invalid mutation inventory identity`);
      }
      mutationOwners++;
    }
  }
  const stale = snapshotStaleness(root, ledger);
  return {inputs: ledger.inputs.length, records: records.size, owners: ids.size, mutationOwners, pinned, stale};
}

try {
  const ledger = JSON.parse(readFileSync(ledgerPath, "utf8"));
  const result = verifyAuditIdentityLedger(ledger);
  const commit = result.pinned.commit.slice(0, 12);
  process.stdout.write(
    `Verified test quality identities: ${result.records} source files, ${result.inputs} inputs, ` +
    `${result.owners} owner(s), ${result.mutationOwners} mutation owner(s). Mutation results not rechecked.\n` +
    `Pinned audit snapshot ${commit} matches its commit tree.\n`,
  );
  if (result.stale.length > 0) {
    const listed = result.stale.slice(0, STALE_LIMIT).map((entry) => `  ${entry.reason}: ${entry.path}`);
    const more = result.stale.length > STALE_LIMIT ? [`  ... and ${result.stale.length - STALE_LIMIT} more`] : [];
    process.stdout.write(
      `Audit snapshot ${commit} is stale for ${result.stale.length} identity(ies) in this checkout. ` +
      "Ordinary implementation changes do not fail this gate.\n" +
      `${[...listed, ...more].join("\n")}\n` +
      `Refresh only when recording new audit evidence: ${REFRESH_COMMAND}\n`,
    );
  }
} catch (error) {
  console.error(`Test quality identity check failed: ${error.message}`);
  process.exitCode = 1;
}
