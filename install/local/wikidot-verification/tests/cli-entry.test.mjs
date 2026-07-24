import assert from "node:assert/strict";
import process from "node:process";
import test from "node:test";

import {isDirectExecution, runCliIfMain} from "../src/cli-entry.mjs";

const moduleUrl = new URL("../scripts/measure.mjs", import.meta.url);

test("CLI entry distinguishes imports from direct execution", () => {
  assert.equal(isDirectExecution(moduleUrl, ["node", new URL(moduleUrl).pathname]), true);
  assert.equal(isDirectExecution(moduleUrl, ["node", "/tmp/other.mjs"]), false);
});

test("CLI entry maps main return values and errors to process exit codes", async (t) => {
  const previous = process.exitCode;
  t.after(() => {
    process.exitCode = previous;
  });
  process.exitCode = undefined;
  const calls = [];
  assert.equal(await runCliIfMain(moduleUrl, async (argv) => {
    calls.push(argv);
    return 4;
  }, {argv: ["--x"], onError: () => 9}), false);
  assert.deepEqual(calls, []);

  const directArgv = ["node", new URL(moduleUrl).pathname];
  assert.equal(await runCliIfMain(moduleUrl, async (argv) => {
    calls.push(argv);
    return 4;
  }, {argv: ["--x"], processArgv: directArgv}), true);
});


test("CLI entry uses the configured error mapper", async (t) => {
  const previous = process.exitCode;
  t.after(() => {
    process.exitCode = previous;
  });
  const directArgv = ["node", new URL(moduleUrl).pathname];
  const errors = [];
  const ran = await runCliIfMain(moduleUrl, async () => {
    throw new Error("boom");
  }, {
    processArgv: directArgv,
    onError: (error) => {
      errors.push(error.message);
      return 7;
    },
  });
  assert.equal(ran, true);
  assert.deepEqual(errors, ["boom"]);
  assert.equal(process.exitCode, 7);
});
