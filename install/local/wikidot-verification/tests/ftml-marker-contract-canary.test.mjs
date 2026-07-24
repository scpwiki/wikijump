import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  composeDocument,
  parseArgs,
  readSeedAdministrator,
} from "../scripts/run-ftml-marker-contract-canary.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(here, "../../../..");
const script = path.join(
  here,
  "..",
  "scripts",
  "run-ftml-marker-contract-canary.mjs",
);
const candidateFtml = "b3e2cca4bbc80693eb4e1085a3acb8619b3b524b";

test("marker canary module parses sliced argv and injects run-owned credentials", async () => {
  const parsed = parseArgs([
    "--candidate-ftml",
    candidateFtml,
    "--output-dir",
    "/tmp/ftml-marker-contract-test",
    "--dry-run",
  ]);
  assert.equal(parsed.candidateFtml, candidateFtml);
  assert.equal(parsed.dryRun, true);

  const administrator = await readSeedAdministrator(repositoryRoot);
  assert.equal(administrator.email, "admin@wikijump");
  assert.equal(administrator.password.length > 0, true);

  const compose = composeDocument({
    project: "marker-test",
    images: {
      database: "database-image",
      cache: "cache-image",
      files: "files-image",
      deepwell: "deepwell-image",
      framerail: "framerail-image",
    },
    labels: { "example.label": "value" },
    binary: "/private/deepwell",
    config: "/private/config.toml",
    migrations: "/private/migrations",
    locales: "/private/locales",
    deepwellPort: 42747,
    framerailPort: 43393,
    credentials: {
      databasePassword: "database-secret",
      filesAccessKey: "marker-access-key",
      filesSecretKey: "files-secret",
    },
  });
  assert.match(compose, /POSTGRES_PASSWORD: "database-secret"/u);
  assert.match(
    compose,
    /DATABASE_URL: "postgres:\/\/wikijump:database-secret@database\/wikijump"/u,
  );
  assert.match(compose, /MINIO_ROOT_PASSWORD: "files-secret"/u);
  assert.doesNotMatch(compose, /defaultpassword/u);
});

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
