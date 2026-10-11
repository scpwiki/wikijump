import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {readFileSync, writeFileSync, mkdtempSync, rmSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname, join, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import test from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
const script = resolve(root, "scripts/verify-test-quality-evidence-identities.mjs");
const original = JSON.parse(readFileSync(resolve(root, "docs/development/test-quality-audit.json"), "utf8"));

function invoke(t, update = () => {}) {
  const temp = mkdtempSync(join(tmpdir(), "wj-ci-audit-identity-"));
  t.after(() => rmSync(temp, {force: true, recursive: true}));
  const ledger = structuredClone(original);
  update(ledger);
  const file = join(temp, "ledger.json");
  writeFileSync(file, JSON.stringify(ledger));
  // GITHUB_ACTIONS enables a single-commit fetch of the pinned commit; tests must never reach the network.
  return spawnSync(process.execPath, [script], {
    cwd: root,
    env: {...process.env, GITHUB_ACTIONS: "", WIKIJUMP_TEST_QUALITY_LEDGER: file},
    encoding: "utf8", timeout: 30_000,
  });
}

test("local identity gate accepts the pinned audit snapshot and its owners", (t) => {
  const result = invoke(t);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /\d+ source files, 9 inputs, 2 owner\(s\), 1 mutation owner\(s\)/u);
  assert.match(result.stdout, /Pinned audit snapshot [0-9a-f]{12} matches its commit tree/u);
  assert.match(result.stdout, /Mutation results not rechecked/u);
});

test("local identity gate rejects a claimed snapshot whose production hash differs from its pinned commit", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.records[0].sha256 = "0".repeat(64);
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /pinned audit snapshot hash mismatch: .+ at [0-9a-f]{40}/u);
  assert.match(result.stderr, /node scripts\/refresh-test-quality-snapshot\.mjs --commit [0-9a-f]{40}/u);
});

test("local identity gate rejects an altered input hash against the pinned commit", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.inputs[0].sha256 = "1".repeat(64);
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /pinned audit snapshot hash mismatch: deepwell\/Cargo\.toml/u);
});

test("local identity gate rejects a pinned commit tree that does not match", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.snapshot.tree = "0".repeat(40);
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /audit snapshot tree does not match pinned commit/u);
});

test("local identity gate rejects an unavailable pinned commit with refresh guidance", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.snapshot.commit = "f".repeat(40);
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /pinned audit commit is unavailable: f{40}/u);
  assert.match(result.stderr, /refresh-test-quality-snapshot\.mjs --commit/u);
});

test("local identity gate rejects missing or malformed snapshot metadata", (t) => {
  const missing = invoke(t, (ledger) => { delete ledger.snapshot; });
  assert.notEqual(missing.status, 0);
  assert.match(missing.stderr, /audit snapshot metadata is missing/u);

  const malformed = invoke(t, (ledger) => { ledger.snapshot.commit = "HEAD"; });
  assert.notEqual(malformed.status, 0);
  assert.match(malformed.stderr, /malformed pinned commit or tree identity/u);
});

test("local identity gate rejects a snapshot that omits a required in-scope input", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.inputs = ledger.inputs.filter((input) => input.path !== "rust-toolchain.toml");
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /audit snapshot is missing required input: rust-toolchain\.toml/u);
});

test("local identity gate rejects production records outside the declared audit scope", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.records.push({
      path: "docs/development/test-quality-audit.json", sha256: "2".repeat(64),
      status: "pending", anchors: [], findings: [], review: null,
    });
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /production source is outside audit scope: docs\/development\/test-quality-audit\.json/u);
});

test("local identity gate rejects duplicate production records", (t) => {
  const result = invoke(t, (ledger) => { ledger.records.push(structuredClone(ledger.records[0])); });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /duplicate production source identity/u);
});

test("local identity gate rejects a missing behavioral test anchor", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.owners[0].anchors[0] = `${ledger.owners[0].anchors[0].split("#")[0]}#definitely_not_a_function_000000`;
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /invalid or missing anchor/u);
});

test("local identity gate rejects missing and duplicate owners", (t) => {
  const missing = invoke(t, (ledger) => { ledger.owners = []; });
  assert.notEqual(missing.status, 0);
  assert.match(missing.stderr, /missing required collections/u);

  const duplicate = invoke(t, (ledger) => { ledger.owners.push(structuredClone(ledger.owners[0])); });
  assert.notEqual(duplicate.status, 0);
  assert.match(duplicate.stderr, /duplicate owner id/u);
});

test("local identity gate rejects path escapes without reading outside source bytes", (t) => {
  const absolute = invoke(t, (ledger) => { ledger.inputs[0].path = "/etc/hosts"; });
  assert.notEqual(absolute.status, 0);
  assert.match(absolute.stderr, /repository-relative path/u);

  const relativeEscape = invoke(t, (ledger) => {
    ledger.owners[0].anchors[0] = "../../../etc/hosts#outside";
  });
  assert.notEqual(relativeEscape.status, 0);
  assert.match(relativeEscape.stderr, /escapes repository/u);
});

test("local identity gate rejects invalid mutation inventory identities", (t) => {
  const result = invoke(t, (ledger) => {
    ledger.owners.find((owner) => owner.mutation_inventory_sha256).mutation_inventory_sha256 = "bad";
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /invalid mutation inventory identity/u);
});
