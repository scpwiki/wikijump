import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {mkdirSync, mkdtempSync, rmSync, writeFileSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname, join} from "node:path";
import test from "node:test";
import {
  REQUIRED_INPUTS, buildSnapshotLedger, sha256, snapshotStaleness, verifyPinnedSnapshot,
} from "../../../../scripts/lib/test-quality-snapshot.mjs";

function git(cwd, args) {
  const result = spawnSync("git", args, {cwd, encoding: "utf8"});
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

// Builds a throwaway repository with every required input and one production source, then a
// ledger pinned to its committed bytes. Staleness is exercised by editing the working tree only.
function fixture(t) {
  const dir = mkdtempSync(join(tmpdir(), "wj-test-quality-snapshot-"));
  t.after(() => rmSync(dir, {force: true, recursive: true}));
  git(dir, ["init", "-q"]);
  git(dir, ["config", "user.email", "test@example.invalid"]);
  git(dir, ["config", "user.name", "Test"]);
  const files = new Map([
    ...REQUIRED_INPUTS.map((file) => [file, `input ${file}\n`]),
    ["deepwell/src/lib.rs", "pub fn stable() {}\n"],
  ]);
  for (const [file, content] of files) {
    mkdirSync(dirname(join(dir, file)), {recursive: true});
    writeFileSync(join(dir, file), content);
  }
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-q", "-m", "fixture"]);
  const commit = git(dir, ["rev-parse", "HEAD"]);
  const tree = git(dir, ["rev-parse", "HEAD^{tree}"]);
  const ledger = {
    schema: "wikijump.test_quality_audit.v1",
    snapshot: {schema: "wikijump.test_quality_snapshot.v1", commit, tree},
    inputs: REQUIRED_INPUTS.map((path) => ({path, sha256: sha256(Buffer.from(`input ${path}\n`))})),
    records: [{
      path: "deepwell/src/lib.rs", sha256: sha256(Buffer.from("pub fn stable() {}\n")),
      status: "pending", anchors: [], findings: [], review: null,
    }],
    owners: [],
  };
  return {dir, ledger};
}

test("pinned snapshot verifies against commit objects and ignores working-tree edits", (t) => {
  const {dir, ledger} = fixture(t);
  const summary = verifyPinnedSnapshot(dir, ledger);
  assert.equal(summary.records, 1);
  assert.equal(summary.inputs, REQUIRED_INPUTS.length);
  assert.deepEqual(snapshotStaleness(dir, ledger), []);

  writeFileSync(join(dir, "deepwell/src/lib.rs"), "pub fn changed() {}\n");
  assert.doesNotThrow(() => verifyPinnedSnapshot(dir, ledger));
  assert.deepEqual(snapshotStaleness(dir, ledger), [{path: "deepwell/src/lib.rs", reason: "changed"}]);
});

test("removed working-tree sources are reported as stale rather than failing", (t) => {
  const {dir, ledger} = fixture(t);
  rmSync(join(dir, "deepwell/src/lib.rs"));
  assert.deepEqual(snapshotStaleness(dir, ledger), [{path: "deepwell/src/lib.rs", reason: "removed"}]);
  assert.doesNotThrow(() => verifyPinnedSnapshot(dir, ledger));
});

test("a pinned hash that disagrees with its commit fails with refresh guidance", (t) => {
  const {dir, ledger} = fixture(t);
  ledger.records[0].sha256 = "0".repeat(64);
  assert.throws(() => verifyPinnedSnapshot(dir, ledger), /pinned audit snapshot hash mismatch: deepwell\/src\/lib\.rs.*--commit/u);
});

test("an unavailable pinned commit fails locally without fetching", (t) => {
  const {dir, ledger} = fixture(t);
  ledger.snapshot.commit = "a".repeat(40);
  assert.throws(() => verifyPinnedSnapshot(dir, ledger), /pinned audit commit is unavailable/u);
});

test("buildSnapshotLedger preserves review state and rejects dropping reviewed records", () => {
  const ledger = {
    schema: "wikijump.test_quality_audit.v1", scope: [], inputs: [], owners: [],
    records: [
      {path: "deepwell/src/a.rs", sha256: "1".repeat(64), status: "covered", anchors: ["x"], findings: [], review: {}},
      {path: "deepwell/src/old.rs", sha256: "2".repeat(64), status: "pending", anchors: [], findings: [], review: null},
    ],
  };
  const inventory = {
    digest: "3".repeat(64),
    inputs: [],
    entries: [
      {path: "deepwell/src/a.rs", role: "production", sha256: "4".repeat(64)},
      {path: "deepwell/src/new.rs", role: "production", sha256: "5".repeat(64)},
    ],
  };
  const next = buildSnapshotLedger(ledger, inventory, {commit: "c".repeat(40), tree: "d".repeat(40)});
  assert.deepEqual(next.records.map((record) => record.path), ["deepwell/src/a.rs", "deepwell/src/new.rs"]);
  assert.equal(next.records[0].status, "covered");
  assert.deepEqual(next.records[0].anchors, ["x"]);
  assert.equal(next.records[0].sha256, "4".repeat(64));
  assert.equal(next.records[1].status, "pending");
  assert.deepEqual(next.snapshot, {schema: "wikijump.test_quality_snapshot.v1", commit: "c".repeat(40), tree: "d".repeat(40)});

  const reviewed = structuredClone(ledger);
  reviewed.records.push({path: "deepwell/src/gone.rs", sha256: "6".repeat(64), status: "covered", anchors: [], findings: [], review: {}});
  assert.throws(() => buildSnapshotLedger(reviewed, inventory, {commit: "c".repeat(40), tree: "d".repeat(40)}),
    /refresh would drop reviewed production records: deepwell\/src\/gone\.rs/u);
});
