import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const script = path.join(
  here,
  "..",
  "scripts",
  "run-ftml-marker-contract-canary.mjs",
);

test("marker canary dry run requires the exact five marker surfaces", () => {
  const result = spawnSync(
    process.execPath,
    [
      script,
      "--candidate-ftml",
      "b3e2cca4bbc80693eb4e1085a3acb8619b3b524b",
      "--output-dir",
      "/tmp/ftml-marker-contract-test",
      "--dry-run",
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  const plan = JSON.parse(result.stdout);
  assert.deepEqual(plan.required_surfaces, [
    "heading",
    "separator",
    "div",
    "span",
    "alignment",
  ]);
  assert.deepEqual(
    plan.fixtures.map((fixture) => fixture.surface).sort(),
    [...plan.required_surfaces].sort(),
  );
  assert.equal(plan.resource_disposition, "delete-on-close");
  assert.equal(plan.baseline_ftml, null);
});

test("marker canary rejects abbreviated FTML revisions", () => {
  const result = spawnSync(
    process.execPath,
    [
      script,
      "--candidate-ftml",
      "b3e2cca4",
      "--output-dir",
      "/tmp/ftml-marker-contract-test",
      "--dry-run",
    ],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /full lowercase SHA/u);
});
