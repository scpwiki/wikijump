import assert from "node:assert/strict";
import test from "node:test";

import { parseStandingBrowserParityArgs } from "../src/standing-browser-parity-runner.mjs";

const policy = "/tmp/standing-policy.json";

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
