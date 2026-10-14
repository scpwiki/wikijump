import assert from "node:assert/strict";
import {mkdtempSync, readFileSync, rmSync} from "node:fs";
import {tmpdir} from "node:os";
import {join, resolve} from "node:path";
import {spawnSync} from "node:child_process";
import test from "node:test";
import {fileURLToPath} from "node:url";

const root = resolve(fileURLToPath(new URL("../../../../", import.meta.url)));
const command = resolve(root, "scripts/run-test-quality-audit.mjs");

function run(args) {
  return spawnSync(process.execPath, [command, ...args], {
    cwd: root, encoding: "utf8", timeout: 120_000,
  });
}

function output(t) {
  const directory = mkdtempSync(join(tmpdir(), "wikijump-source-audit-cli-"));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  return join(directory, "report");
}

test("source audit CLI rejects unsupported commands and checkout-local outputs", () => {
  const unknown = run(["unknown", "--output-dir", "/tmp/wj-unknown-command"]);
  assert.notEqual(unknown.status, 0);
  assert.match(unknown.stderr, /unknown audit mode/u);

  const insideCheckout = run(["inventory", "--output-dir", join(root, `.tmp-audit-cli-${process.pid}-${Date.now()}`)]);
  assert.notEqual(insideCheckout.status, 0);
  assert.match(insideCheckout.stderr, /outside the source checkout/u);
});

test("source audit inventory CLI writes a fresh report and prints its identity", (t) => {
  const directory = output(t);
  const result = run(["inventory", "--output-dir", directory]);
  assert.equal(result.status, 0, result.stderr);
  const inventory = JSON.parse(readFileSync(join(directory, "inventory.json"), "utf8"));
  assert.match(inventory.digest, /^[0-9a-f]{64}$/u);
  assert.match(result.stdout, new RegExp(inventory.digest));
});

test("incomplete verification reports pending records after mutation owners are proven", (t) => {
  const directory = output(t);
  const result = run(["verify", "--allow-incomplete", "--output-dir", directory]);
  assert.equal(result.status, 0, result.stderr);
  const verification = JSON.parse(readFileSync(join(directory, "verification.json"), "utf8"));
  assert.ok(verification.production_files > 0);
  assert.ok(verification.unresolved_records > 0);
  assert.equal(verification.unresolved_mutations, 0);
  assert.equal(verification.closure_ready, false);
});

test("normal verification refuses the incomplete issue ledger", (t) => {
  const result = run(["verify", "--output-dir", output(t)]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /production records remain unresolved/u);
});
