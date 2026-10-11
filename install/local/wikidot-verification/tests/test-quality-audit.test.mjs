import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname, join, resolve} from "node:path";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
import test from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
const driver = join(root, "scripts/run-test-quality-frontier-audit.mjs");
const sha = (path) => createHash("sha256").update(readFileSync(join(root, path))).digest("hex");

const sourcePath = "docs/development/test-quality-audit-frontier-20261008.md";
const testPath = "scripts/run-test-quality-frontier-audit.mjs";

function identity(path) {
  return {path, sha256: sha(path)};
}

function baseLedger() {
  return {
    schema: 1,
    lockfiles: [identity("deepwell/Cargo.lock")],
    owners: [{
      id: "fixture:owner",
      status: "in_progress",
      source: identity(sourcePath),
      tests: [{...identity(testPath), anchors: ["verifyIdentity"]}],
    }],
  };
}

function runVerify(ledger, args = []) {
  const directory = mkdtempSync(join(tmpdir(), "wj-audit-test-"));
  const ledgerFile = join(directory, "ledger.json");
  writeFileSync(ledgerFile, JSON.stringify(ledger));
  return spawnSync("node", [driver, "verify", "--output-dir", join(directory, "out"), ...args], {
    cwd: root,
    encoding: "utf8",
    env: {...process.env, WIKIJUMP_TEST_QUALITY_LEDGER: ledgerFile},
  });
}

function rejected(ledger, pattern, args) {
  const result = runVerify(ledger, args);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, pattern);
}

test("verify accepts a ledger whose identities, anchors and owners are current", () => {
  const result = runVerify(baseLedger());
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Verified 1 audit owner/u);
});

test("verify rejects a requested owner that is not in the ledger", () => {
  rejected(baseLedger(), /unknown audit owner: fixture:missing/u, ["--owner", "fixture:missing"]);
});

test("verify rejects an empty ledger", () => {
  const ledger = baseLedger();
  ledger.owners = [];
  rejected(ledger, /no owners/u);
});

test("verify rejects stale source, test and lockfile hashes", () => {
  const source = baseLedger();
  source.owners[0].source.sha256 = "0".repeat(64);
  rejected(source, /source for fixture:owner hash is stale/u);
  const suite = baseLedger();
  suite.owners[0].tests[0].sha256 = "0".repeat(64);
  rejected(suite, /test for fixture:owner hash is stale/u);
  const lock = baseLedger();
  lock.lockfiles[0].sha256 = "0".repeat(64);
  rejected(lock, /lockfile hash is stale/u);
});

test("verify rejects owners without tests and invalid test anchors", () => {
  const none = baseLedger();
  none.owners[0].tests = [];
  rejected(none, /has no tests/u);
  const anchor = baseLedger();
  anchor.owners[0].tests[0].anchors = ["no_such_anchor_in_that_file"];
  rejected(anchor, /invalid test anchor/u);
});

test("verify rejects accepted owners with unresolved gaps", () => {
  const ledger = baseLedger();
  ledger.owners[0].status = "accepted";
  ledger.owners[0].acceptance_gaps = ["open"];
  rejected(ledger, /unresolved acceptance gaps/u);
});

test("verify rejects contradictory mutation evidence and pending survivors on accepted owners", () => {
  const mutation = (evidence) => ({file: "x", function: "x", inventory_count: 1, runs: [], completed_evidence: [evidence]});
  const sums = baseLedger();
  sums.owners[0].mutation = mutation({run: "r", shard: "0/1", mutants: 3, caught: 1, missed: 0, unviable: 0, timeout: 0});
  rejected(sums, /outcomes sum to 1, expected 3/u);

  const undisposed = baseLedger();
  undisposed.owners[0].status = "accepted";
  undisposed.owners[0].mutation = mutation({run: "r", shard: "0/1", mutants: 1, caught: 0, missed: 1, unviable: 0, timeout: 0});
  rejected(undisposed, /1 survivor\(s\) but 0 disposition/u);

  const pending = baseLedger();
  pending.owners[0].status = "accepted";
  pending.owners[0].mutation = mutation({
    run: "r", shard: "0/1", mutants: 1, caught: 0, missed: 1, unviable: 0, timeout: 0,
    survivor_dispositions: [{classification: "provisionally equivalent pending independent review"}],
  });
  rejected(pending, /pending review/u);
});

test("driver rejects unknown commands, missing output directory and malformed shards", () => {
  const run = (...args) => spawnSync("node", [driver, ...args], {cwd: root, encoding: "utf8"});
  assert.equal(run("bogus").status, 2);
  assert.match(run("verify").stderr, /--output-dir/u);
  assert.match(run("mutate", "--output-dir", "x", "--shard", "8/8").stderr, /zero-based/u);
});


test("frontier inventory tracks all checked-in frontend visual style and shell sources", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "wj-frontier-visual-inventory-"));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const inventory = spawnSync(process.execPath, [driver, "inventory", "--output-dir", directory], {
    cwd: root,
    encoding: "utf8",
    timeout: 120_000,
  });
  assert.equal(inventory.status, 0, inventory.stderr);
  const report = JSON.parse(readFileSync(join(directory, "inventory.json"), "utf8"));
  const actual = new Set(report.production.map(({path}) => path));
  const tracked = spawnSync("git", ["ls-files", "-z", "--", "framerail/src"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(tracked.status, 0, tracked.stderr);
  const visualSources = tracked.stdout.split("\0").filter((name) =>
    /\.(?:css|scss|sass|html)$/u.test(name));
  assert.ok(visualSources.length > 0, "the visual production inventory must not be vacuously empty");
  for (const visualSource of visualSources) {
    assert.ok(actual.has(visualSource), `omitted visual source: ${visualSource}`);
  }
  // This static reachability analysis is only a candidate import graph; it
  // must never be advertised as proof that visual output matches Wikidot.
  assert.match(report.static_import_ownership.semantics, /not behavioral coverage/u);
  assert.equal(report.static_import_ownership.production_file_count, actual.size);
  assert.ok(report.executable_owners.every(({path}) => !/\.(?:css|scss|sass|html)$/u.test(path)),
    "stylesheet and HTML files are not executable test owners");
});
