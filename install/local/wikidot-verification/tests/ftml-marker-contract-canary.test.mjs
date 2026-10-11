import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { RETAINED_CANARY } from "../src/ftml-marker-contract-candidate-case-set.mjs";
import {
  composeDocument,
  pageMutationContext,
  parseArgs,
  prepareRunOwnedSeeder,
  readSeedAdministrator,
  replaceFtmlPin,
  selectFtmlPinRewrite,
} from "../scripts/run-ftml-marker-contract-canary.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(here, "../../../..");
const script = path.join(
  here,
  "..",
  "scripts",
  "run-ftml-marker-contract-canary.mjs",
);
const currentFtml = "e82b6909b7c7d0c2d5cef7514dea0ad3c469c2fb";
const ownershipCanaryFtml = "62ebba4efda1f10e82363c23c925061fbe939e49";
const previousCanaryFtml = "3f02c5af6ec7c69599b881a8fc7ece8ea05a0115";
const requiredSurfaces = ["heading", "separator", "div", "span", "alignment"];
const sanitizedEnvironment = Object.fromEntries(
  Object.entries(process.env).filter(
    ([key]) =>
      !key.startsWith("GIT_") &&
      ![
        "DOCKER_CONTEXT",
        "DOCKER_HOST",
        "GIT_CONFIG",
        "GIT_CONFIG_COUNT",
        "GIT_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_WORK_TREE",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "NODE_OPTIONS",
        "PYTHONHOME",
        "PYTHONPATH",
      ].includes(key),
  ),
);

test("the retained marker canary summary identifies its own candidate run", () => {
  const summary = JSON.parse(
    readFileSync(path.join(repositoryRoot, RETAINED_CANARY), "utf8"),
  );

  assert.equal(summary.status, "pass");
  assert.equal(summary.candidate_ftml, currentFtml);
  assert.deepEqual(summary.required_surfaces, requiredSurfaces);
  assert.match(
    summary.run_id ?? "",
    new RegExp(`^ftml-marker-${summary.candidate_ftml.slice(0, 8)}-`, "u"),
    "a hand-edited candidate_ftml must not be accepted without a matching run",
  );
});

test("committed manifest and lock pin the merged FTML revision", () => {
  const manifest = readFileSync(
    path.join(repositoryRoot, "deepwell/Cargo.toml"),
    "utf8",
  );
  const lock = readFileSync(
    path.join(repositoryRoot, "deepwell/Cargo.lock"),
    "utf8",
  );
  assert.equal(
    manifest.match(
      new RegExp(
        `ftml = \\{ git = "https://github\\.com/Rokurolize/ftml", rev = "${currentFtml}" \\}`,
        "gu",
      ),
    )?.length,
    1,
  );
  assert.equal(
    lock.match(
      new RegExp(
        `source = "git\\+https://github\\.com/Rokurolize/ftml\\?rev=${currentFtml}#${currentFtml}"`,
        "gu",
      ),
    )?.length,
    1,
  );
});

test("the 2026-08-04 marker canary receipt remains immutable", () => {
  const receipt = JSON.parse(
    readFileSync(
      path.join(
        repositoryRoot,
        "install/local/wikidot-verification/artifacts",
        "ftml-block-argument-pin-canary-20260804-3f02c5af.json",
      ),
      "utf8",
    ),
  );

  assert.equal(receipt.status, "pass");
  assert.equal(
    receipt.baseline_ftml_sha,
    "6d1550f283f93ec3f4257ffda238a8f9003eed19",
  );
  assert.equal(receipt.candidate_ftml_sha, previousCanaryFtml);
  assert.deepEqual(receipt.required_surfaces, requiredSurfaces);
  assert.deepEqual(receipt.comparison, {
    schema: "wikijump_local_lab.render_compare.v1",
    pairs_total: requiredSurfaces.length,
    matches: requiredSurfaces.length,
    accepted_differences: 0,
    regressions: 0,
    verdict_sha256:
      "abbe667fbbb227ef75b2e428d32202771eaa4f0b9eff2f918e1209c4940a0b0b",
  });
  assert.deepEqual(receipt.resource_disposition, {
    policy: "delete-on-close",
    disposable_containers_remaining: 0,
  });
});

test("the 2026-08-10 ownership pin marker canary receipt remains immutable", () => {
  const receipt = JSON.parse(
    readFileSync(
      path.join(
        repositoryRoot,
        "install/local/wikidot-verification/artifacts",
        "ftml-ownership-pin-canary-20260810-62ebba4e.json",
      ),
      "utf8",
    ),
  );

  assert.equal(receipt.status, "pass");
  assert.equal(
    receipt.wikijump_base_sha,
    "78d83cb920a7ea72626f2bcfc747f1493686900b",
  );
  assert.equal(
    receipt.baseline_ftml_sha,
    "902e72a2ff261b7af42402734b2f8b659e6a294a",
  );
  assert.equal(receipt.candidate_ftml_sha, ownershipCanaryFtml);
  assert.deepEqual(receipt.required_surfaces, requiredSurfaces);
  assert.deepEqual(
    {
      schema: receipt.comparison.schema,
      pairs_total: receipt.comparison.pairs_total,
      matches: receipt.comparison.matches,
      accepted_differences: receipt.comparison.accepted_differences,
      regressions: receipt.comparison.regressions,
    },
    {
      schema: "wikijump_local_lab.render_compare.v1",
      pairs_total: requiredSurfaces.length,
      matches: requiredSurfaces.length,
      accepted_differences: 0,
      regressions: 0,
    },
  );
  assert.equal(
    receipt.evidence.root_path,
    "/home/roku/wjlab/evidence/20260810-open43-ftml-marker-62ebba4e-pin78d",
  );
  assert.deepEqual(receipt.resource_disposition, {
    policy: "delete-on-close",
    disposable_worktrees_remaining: 0,
    disposable_containers_remaining: 0,
    disposable_volumes_remaining: 0,
    disposable_networks_remaining: 0,
    outer_worktree_removed: true,
  });
});

test("the 2026-08-01 marker canary receipt remains immutable", () => {
  const receipt = JSON.parse(
    readFileSync(
      path.join(
        repositoryRoot,
        "install/local/wikidot-verification/artifacts",
        "ftml-block-argument-pin-canary-20260801.json",
      ),
      "utf8",
    ),
  );
  assert.equal(receipt.run_id, "ftml-marker-a8fcd3dc-2a7ca167");
  assert.equal(
    receipt.wikijump_base_sha,
    "4ec938ea409e68de0eea5d459d5fd03c4b01046c",
  );
  assert.equal(
    receipt.baseline_ftml_sha,
    "3a5d874f485eb6ac64638bb5aa189beffbcff410",
  );
  assert.equal(
    receipt.candidate_ftml_sha,
    "a8fcd3dce089aefd6a9a3619116d4777c9ebd7cc",
  );
  assert.deepEqual(receipt.comparison, {
    schema: "wikijump_local_lab.render_compare.v1",
    pairs_total: 5,
    matches: 5,
    accepted_differences: 0,
    regressions: 0,
    verdict_sha256:
      "6463d478a6a1b087240deb1b1ac0d54f6f87608372c3f4f0c5e9510400217f83",
  });
});

test("marker canary authenticates page mutations in the exact page context", () => {
  const context = pageMutationContext(
    {
      "X-Deepwell-Session-Token": "session-token",
      "X-Deepwell-Site-Id": "42",
    },
    "marker-heading",
  );
  assert.deepEqual(context, {
    "X-Deepwell-Session-Token": "session-token",
    "X-Deepwell-Site-Id": "42",
    "X-Deepwell-Page": "marker-heading",
  });
});

test("marker canary changes the manifest and lock to the same FTML revision", () => {
  const baselineFtml = "1".repeat(40);
  const candidateFtml = "2".repeat(40);
  const manifest = `ftml = { git = "https://github.com/Rokurolize/ftml", rev = "${baselineFtml}" }\n`;

  assert.equal(
    replaceFtmlPin(manifest, baselineFtml, candidateFtml),
    `ftml = { git = "https://github.com/Rokurolize/ftml", rev = "${candidateFtml}" }\n`,
  );
  assert.throws(
    () => replaceFtmlPin(manifest, "3".repeat(40), candidateFtml),
    /source FTML pin exactly once/u,
  );
});

test("marker canary preserves whichever side already equals exact HEAD", () => {
  const baselineFtml = "1".repeat(40);
  const candidateFtml = "2".repeat(40);

  assert.deepEqual(
    selectFtmlPinRewrite(baselineFtml, baselineFtml, candidateFtml),
    {
      stage: "candidate",
      sourceFtml: baselineFtml,
      targetFtml: candidateFtml,
    },
  );
  assert.deepEqual(
    selectFtmlPinRewrite(candidateFtml, baselineFtml, candidateFtml),
    {
      stage: "baseline",
      sourceFtml: candidateFtml,
      targetFtml: baselineFtml,
    },
  );
  assert.throws(
    () => selectFtmlPinRewrite("3".repeat(40), baselineFtml, candidateFtml),
    /matches neither baseline/u,
  );
  assert.throws(
    () => selectFtmlPinRewrite(baselineFtml, baselineFtml, baselineFtml),
    /must be distinct/u,
  );
});

test("marker canary module parses sliced argv and injects run-owned credentials", async () => {
  const parsed = parseArgs([
    "--candidate-ftml",
    currentFtml,
    "--output-dir",
    "/tmp/ftml-marker-contract-test",
    "--dry-run",
  ]);
  assert.equal(parsed.candidateFtml, currentFtml);
  assert.equal(parsed.dryRun, true);

  const administrator = await readSeedAdministrator(repositoryRoot);
  assert.equal(administrator.email, "admin@wikijump");
  assert.equal(administrator.password, null);

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
    seeder: "/private/seeder",
    deepwellPort: 42747,
    framerailPort: 43393,
    credentials: {
      databasePassword: "database-secret",
      filesAccessKey: "marker-access-key",
      filesSecretKey: "files-secret",
      rpcToken: "rpc-secret",
    },
  });
  assert.match(compose, /POSTGRES_PASSWORD: "database-secret"/u);
  assert.match(
    compose,
    /DATABASE_URL: "postgres:\/\/wikijump:database-secret@database\/wikijump"/u,
  );
  assert.match(compose, /MINIO_ROOT_PASSWORD: "files-secret"/u);
  assert.doesNotMatch(compose, /defaultpassword/u);
  assert.match(compose, /\/usr\/local\/bin\/sqlx migrate run/u);
  assert.equal(
    compose.match(/DEEPWELL_RPC_TOKEN: "rpc-secret"/gu)?.length,
    2,
  );
  assert.match(compose, /target: \/seeder/u);
});

test("marker canary gives each disposable stack a run-owned administrator password", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-marker-seeder-"));
  try {
    const seeder = await prepareRunOwnedSeeder(
      repositoryRoot,
      path.join(root, "seeder"),
      "run-owned-admin-secret",
    );
    assert.deepEqual(seeder, {
      email: "admin@wikijump",
      password: "run-owned-admin-secret",
    });
    const users = JSON.parse(await fs.readFile(path.join(root, "seeder", "users.json"), "utf8"));
    assert.equal(users.find((user) => user.slug === "administrator").password, "run-owned-admin-secret");
    assert.equal((await fs.stat(path.join(root, "seeder", "users.json"))).mode & 0o777, 0o644);
    assert.equal((await readSeedAdministrator(repositoryRoot)).password, null);
  } finally {
    await fs.rm(root, {recursive: true, force: true});
  }
});

test("marker canary dry run requires the exact five marker surfaces", () => {
  const result = spawnSync(
    process.execPath,
    [
      script,
      "--candidate-ftml",
      currentFtml,
      "--output-dir",
      "/tmp/ftml-marker-contract-test",
      "--dry-run",
    ],
    { encoding: "utf8", env: sanitizedEnvironment },
  );
  assert.equal(result.status, 0, result.stderr);
  const plan = JSON.parse(result.stdout);
  assert.deepEqual(plan.required_surfaces, requiredSurfaces);
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
    { encoding: "utf8", env: sanitizedEnvironment },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /full lowercase SHA/u);
});
