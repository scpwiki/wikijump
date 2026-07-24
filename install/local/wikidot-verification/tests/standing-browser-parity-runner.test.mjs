import assert from "node:assert/strict";
import test from "node:test";

import { main as runParityCli, usage as parityCliUsage } from "../scripts/run-standing-browser-parity.mjs";
import { closeParityBrowserResources } from "../src/standing-browser-parity-browser-session.mjs";
import { parseStandingBrowserParityArgs } from "../src/standing-browser-parity-runner.mjs";

const policy = "/tmp/standing-policy.json";

test("standing parity CLI exposes result and help without opening a browser", async () => {
  const output = [];
  const code = await runParityCli(["node", "script", "--mode", "candidate"], {
    parseArgs: () => ({mode: "candidate"}),
    runParity: async () => ({mode: "candidate", status: "pass", output_dir: "/tmp/parity"}),
    stdout: (line) => output.push(JSON.parse(line)),
  });
  assert.equal(code, 0);
  assert.deepEqual(output, [{
    schema: "wikijump.standing_browser_parity_cli_result.v1",
    mode: "candidate",
    status: "pass",
    output_dir: "/tmp/parity",
  }]);
  assert.match(parityCliUsage(), /live-reference/u);
});

test("parity browser cleanup attempts every resource and reports every failure", async () => {
  const attempts = [];
  const context = {
    async close() {
      attempts.push("context");
      throw new Error("context close failed");
    },
  };
  const browser = {
    async close() {
      attempts.push("browser");
      throw new Error("browser close failed");
    },
  };

  await assert.rejects(
    () => closeParityBrowserResources(context, browser),
    (error) => {
      assert(error instanceof AggregateError);
      assert.equal(error.errors.length, 2);
      return true;
    },
  );
  assert.deepEqual(attempts, ["context", "browser"]);
});

test("live reference capture requires a policy before a browser can be opened", () => {
  const args = parseStandingBrowserParityArgs([
    "node",
    "runner",
    "--mode",
    "live-reference",
    "--output-dir",
    "/tmp/standing-reference",
    "--live-completion-policy",
    policy,
    "--viewport",
    "1440x960",
  ]);
  assert.equal(args.mode, "live-reference");
  assert.deepEqual(args.viewport, { width: 1440, height: 960 });
  assert.equal(args.timeoutMs, 900_000);
  assert.throws(
    () =>
      parseStandingBrowserParityArgs([
        "node",
        "runner",
        "--mode",
        "live-reference",
        "--output-dir",
        "/tmp/standing-reference",
      ]),
    /live-completion-policy/u,
  );
});

test("candidate capture requires its sealed identity and exact live reference digest", () => {
  assert.throws(
    () =>
      parseStandingBrowserParityArgs([
        "node",
        "runner",
        "--mode",
        "candidate",
        "--output-dir",
        "/tmp/standing-candidate",
        "--live-completion-policy",
        policy,
      ]),
    /candidate-identity/u,
  );
  const args = parseStandingBrowserParityArgs([
    "node",
    "runner",
    "--mode",
    "candidate",
    "--output-dir",
    "/tmp/standing-candidate",
    "--live-completion-policy",
    policy,
    "--candidate-identity",
    "/tmp/candidate.json",
    "--live-reference-ledger",
    "/tmp/reference.json",
    "--live-reference-sha256",
    "a".repeat(64),
  ]);
  assert.equal(args.mode, "candidate");
  assert.equal(args.liveReferenceSha256, "a".repeat(64));
});
