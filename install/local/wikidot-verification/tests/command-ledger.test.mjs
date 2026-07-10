import assert from "node:assert/strict";
import {spawn, spawnSync} from "node:child_process";
import {existsSync, readFileSync} from "node:fs";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";

const PACKAGE_ROOT = fileURLToPath(new URL("..", import.meta.url));
const MEASURE_SCRIPT = fileURLToPath(new URL("../scripts/measure.mjs", import.meta.url));

async function tempLedger(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-command-ledger-"));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return path.join(directory, "ledger.jsonl");
}

function runMeasure(args) {
  return spawnSync(process.execPath, [MEASURE_SCRIPT, ...args], {
    cwd: PACKAGE_ROOT,
    encoding: "utf8",
  });
}

function nodeCommand(source) {
  return [process.execPath, "-e", source];
}

function ledgerRecords(ledgerPath) {
  return readFileSync(ledgerPath, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}

function waitForClose(child, timeoutMs = 3000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      reject(new Error("child did not close before timeout"));
    }, timeoutMs);
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once("close", (code, signal) => {
      clearTimeout(timer);
      resolve({code, signal});
    });
  });
}

test("success tees stdout and records one ledger line", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure(["--family", "node-mjs", "--ledger", ledgerPath, "--", ...nodeCommand("console.log('ok')")]);

  assert.equal(result.status, 0);
  assert.match(result.stdout, /ok/);

  const records = ledgerRecords(ledgerPath);
  assert.equal(records.length, 1);
  const record = records[0];
  assert.equal(record.exitCode, 0);
  assert.equal(record.timedOut, false);
  assert.equal(record.cacheStatus, "uncached");
  assert.equal(record.family, "node-mjs");
  assert.equal(record.schemaVersion, 1);
  assert.deepEqual(Object.keys(record), [
    "schemaVersion",
    "runId",
    "family",
    "label",
    "command",
    "args",
    "cwd",
    "gitHead",
    "startTime",
    "endTime",
    "elapsedMs",
    "exitCode",
    "signal",
    "timedOut",
    "timeoutMs",
    "cacheStatus",
    "artifactPaths",
    "stdoutBytes",
    "stderrBytes",
    "firstErrorExcerpt",
    "rerunCommand",
    "envFingerprint",
  ]);
  assert.match(readFileSync(record.artifactPaths.stdout, "utf8"), /ok/);
});

test("failure preserves child exit, stderr, and first error excerpt", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure([
    "--family",
    "node-mjs",
    "--ledger",
    ledgerPath,
    "--",
    ...nodeCommand("console.error('boom'); process.exit(7)"),
  ]);

  assert.equal(result.status, 7);
  assert.match(result.stderr, /boom/);

  const [record] = ledgerRecords(ledgerPath);
  assert.equal(record.exitCode, 7);
  assert.ok(record.stderrBytes > 0);
  assert.match(record.firstErrorExcerpt, /boom/);
  assert.match(readFileSync(record.artifactPaths.stderr, "utf8"), /boom/);
});

test("quiet suppresses live child output but still writes logs and byte counts", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure([
    "--family",
    "node-mjs",
    "--ledger",
    ledgerPath,
    "--quiet",
    "--",
    ...nodeCommand("console.log('quiet-out'); console.error('quiet-err')"),
  ]);

  assert.equal(result.status, 0);
  assert.doesNotMatch(result.stdout, /quiet-out/);
  assert.doesNotMatch(result.stderr, /quiet-err/);

  const [record] = ledgerRecords(ledgerPath);
  assert.ok(record.stdoutBytes > 0);
  assert.ok(record.stderrBytes > 0);
  assert.match(readFileSync(record.artifactPaths.stdout, "utf8"), /quiet-out/);
  assert.match(readFileSync(record.artifactPaths.stderr, "utf8"), /quiet-err/);
});

test("append writes two independently parseable JSONL records", async (t) => {
  const ledgerPath = await tempLedger(t);
  const first = runMeasure(["--family", "node-mjs", "--ledger", ledgerPath, "--", ...nodeCommand("console.log('one')")]);
  const second = runMeasure(["--family", "node-mjs", "--ledger", ledgerPath, "--", ...nodeCommand("console.log('two')")]);

  assert.equal(first.status, 0);
  assert.equal(second.status, 0);
  const records = ledgerRecords(ledgerPath);
  assert.equal(records.length, 2);
  assert.notEqual(records[0].runId, records[1].runId);
});

test("timeout exits 124 and records a timed out run", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure([
    "--family",
    "node-mjs",
    "--ledger",
    ledgerPath,
    "--timeout",
    "200",
    "--",
    ...nodeCommand("setTimeout(() => {}, 10000)"),
  ]);

  assert.equal(result.status, 124);
  const [record] = ledgerRecords(ledgerPath);
  assert.equal(record.timedOut, true);
  assert.equal(record.exitCode, null);
  assert.equal(record.timeoutMs, 200);
  assert.ok(record.elapsedMs < 10000);
  assert.ok(existsSync(record.artifactPaths.stdout));
  assert.ok(existsSync(record.artifactPaths.stderr));
});

test("usage errors exit 2 and print usage to stderr", async (t) => {
  const ledgerPath = await tempLedger(t);
  const missingFamily = runMeasure(["--ledger", ledgerPath, "--", ...nodeCommand("console.log('unused')")]);
  assert.equal(missingFamily.status, 2);
  assert.match(missingFamily.stderr, /Usage/);

  const missingSeparator = runMeasure(["--family", "node-mjs", "--ledger", ledgerPath, ...nodeCommand("console.log('unused')")]);
  assert.equal(missingSeparator.status, 2);
  assert.match(missingSeparator.stderr, /Usage/);
});

test("option values that look like help flags do not trigger help", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure([
    "--family",
    "node-mjs",
    "--label",
    "--help",
    "--ledger",
    ledgerPath,
    "--",
    ...nodeCommand("console.log('ran')"),
  ]);

  assert.equal(result.status, 0);
  assert.match(result.stdout, /ran/);
  const [record] = ledgerRecords(ledgerPath);
  assert.equal(record.label, "--help");
  assert.equal(record.exitCode, 0);

  const helpResult = runMeasure(["--help"]);
  assert.equal(helpResult.status, 0);
  assert.match(helpResult.stdout, /Usage/);
});

test("ENOENT exits 127 and records the spawn error excerpt", async (t) => {
  const ledgerPath = await tempLedger(t);
  const result = runMeasure(["--family", "node-mjs", "--ledger", ledgerPath, "--", "wikijump-no-such-binary-for-ledger"]);

  assert.equal(result.status, 127);
  const [record] = ledgerRecords(ledgerPath);
  assert.match(record.firstErrorExcerpt, /ENOENT|no such/i);
});

test("closed downstream stdout does not change wrapped command outcome or lose the ledger record", async (t) => {
  const ledgerPath = await tempLedger(t);
  const childSource = [
    "const chunk = 'x'.repeat(1024);",
    "let count = 0;",
    "const interval = setInterval(() => {",
    "  process.stdout.write(chunk);",
    "  count += 1;",
    "  if (count >= 4) {",
    "    clearInterval(interval);",
    "    setTimeout(() => process.exit(0), 20);",
    "  }",
    "}, 10);",
  ].join("\n");
  const child = spawn(
    process.execPath,
    [MEASURE_SCRIPT, "--family", "node-mjs", "--ledger", ledgerPath, "--quiet", "--", ...nodeCommand(childSource)],
    {
      cwd: PACKAGE_ROOT,
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  child.stdout.on("error", () => {});
  child.stdout.once("data", () => {
    child.stdout.destroy();
  });

  const close = await waitForClose(child);
  assert.equal(close.signal, null);
  assert.equal(close.code, 0);
  const records = ledgerRecords(ledgerPath);
  assert.equal(records.length, 1);
  assert.equal(records[0].exitCode, 0);
  assert.equal(records[0].timedOut, false);
  assert.ok(records[0].stdoutBytes > 0);
});

test("redacts sensitive arguments, rerun command, and output artifacts", async (t) => {
  const ledgerPath = await tempLedger(t);
  const dbUrl = "postgres://wikijump:DB_PASSWORD_AUTOTEST@127.0.0.1:5432/wikijump";
  const sessionToken = "SESSION_TOKEN_AUTOTEST";
  const s3Secret = "S3_SECRET_AUTOTEST";
  const outputSecret = "OUTPUT_SECRET_AUTOTEST";
  const result = runMeasure([
    "--family",
    "import",
    "--ledger",
    ledgerPath,
    "--",
    ...nodeCommand([
      "const args = process.argv.slice(1);",

      "console.log(`args=${args.join(' ')}`);",
      `console.log('token=${outputSecret}');`,
      `console.error('secret=${outputSecret}');`,
    ].join("\n")),
    "--",
    "--db-url",
    dbUrl,
    "--session-token",
    sessionToken,
    `--attachment-s3-secret-access-key=${s3Secret}`,
  ]);

  assert.equal(result.status, 0);
  const ledgerText = readFileSync(ledgerPath, "utf8");
  assert.doesNotMatch(ledgerText, /DB_PASSWORD_AUTOTEST|SESSION_TOKEN_AUTOTEST|S3_SECRET_AUTOTEST/);
  assert.match(ledgerText, /\[REDACTED\]/);

  const [record] = ledgerRecords(ledgerPath);
  assert.deepEqual(record.args.slice(-5), [
    "--db-url",
    "[REDACTED]",
    "--session-token",
    "[REDACTED]",
    "--attachment-s3-secret-access-key=[REDACTED]",
  ]);
  assert.doesNotMatch(record.rerunCommand, /DB_PASSWORD_AUTOTEST|SESSION_TOKEN_AUTOTEST|S3_SECRET_AUTOTEST/);
  assert.doesNotMatch(readFileSync(record.artifactPaths.stdout, "utf8"), /DB_PASSWORD_AUTOTEST|SESSION_TOKEN_AUTOTEST|S3_SECRET_AUTOTEST|OUTPUT_SECRET_AUTOTEST/);
  assert.doesNotMatch(readFileSync(record.artifactPaths.stderr, "utf8"), /OUTPUT_SECRET_AUTOTEST/);
});
