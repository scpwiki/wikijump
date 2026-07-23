#!/usr/bin/env node

// Runs the FTML pin marker-contract gate in disposable local stacks. The only
// comparison implementation is the existing V3 Local Lab comparator.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPT_DIR = path.dirname(SCRIPT_PATH);
const REPOSITORY_ROOT = path.resolve(SCRIPT_DIR, "../../../..");
const FIXTURES_PATH = path.join(
  REPOSITORY_ROOT,
  "install/local/wikidot-verification/fixtures/ftml-marker-contract/fixtures.json",
);
const BUILD_CANDIDATE = "/home/roku/wjlab/scripts/build-deepwell-candidate.sh";
const LEASE = "/home/roku/.local/bin/roku-resource-lease";
const REQUIRED_SURFACES = ["heading", "separator", "div", "span", "alignment"];
const OWNER = "ftml-marker-contract-canary";
const EXPIRY_HOURS = 8;

function usage() {
  console.log(`Usage: run-ftml-marker-contract-canary.mjs --candidate-ftml SHA --output-dir DIR [--baseline-ftml SHA] [--work-root DIR] [--dry-run]

Creates baseline and candidate throwaway worktrees, builds Deepwell under registered leases, starts only disposable non-443 database/cache/files/Deepwell/Framerail services, and compares fixture visible text with the existing V3 Local Lab comparator. It never reads or writes a standing runtime or corpus volume.`);
}

function sha(value) {
  if (!/^[0-9a-f]{40}$/u.test(value ?? ""))
    throw new Error(`expected full lowercase SHA: ${value}`);
  return value;
}

function parseArgs(argv) {
  const args = {
    workRoot: path.join(os.tmpdir(), "wikijump-ftml-marker-contract"),
    dryRun: false,
  };
  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      const next = argv[++index];
      if (!next || next.startsWith("--"))
        throw new Error(`${arg} requires a value`);
      return next;
    };
    if (arg === "--candidate-ftml") args.candidateFtml = sha(value());
    else if (arg === "--baseline-ftml") args.baselineFtml = sha(value());
    else if (arg === "--output-dir") args.outputDir = path.resolve(value());
    else if (arg === "--work-root") args.workRoot = path.resolve(value());
    else if (arg === "--dry-run") args.dryRun = true;
    else if (arg === "--help" || arg === "-h") {
      usage();
      process.exit(0);
    } else throw new Error(`unknown argument: ${arg}`);
  }
  if (!args.candidateFtml) throw new Error("--candidate-ftml is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  return args;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? REPOSITORY_ROOT,
    encoding: "utf8",
    stdio: "pipe",
    env: options.env ?? process.env,
  });
  if (result.status !== 0)
    throw new Error(
      `${command} ${args.join(" ")} failed: ${result.stderr || result.stdout}`,
    );
  return result.stdout.trim();
}

function output(command, args, options = {}) {
  return run(command, args, options);
}

async function runCandidateBuild(args) {
  const deadline = Date.now() + 15 * 60 * 1000;
  for (;;) {
    try {
      run(BUILD_CANDIDATE, args);
      return;
    } catch (error) {
      if (
        !error.message.includes("unregistered legacy Rust build detected") ||
        Date.now() >= deadline
      )
        throw error;
      await new Promise((resolve) => setTimeout(resolve, 5_000));
    }
  }
}

async function writeJson(filePath, value) {
  await fs.mkdir(path.dirname(filePath), { recursive: true, mode: 0o700 });
  await fs.writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, {
    mode: 0o600,
  });
}

async function freePort() {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close((error) => (error ? reject(error) : resolve(address.port)));
    });
  });
}

function currentFtmlSha(repository) {
  const source = output("python3", [
    "/home/roku/wjlab/scripts/candidate-artifact-manifest.py",
    "ftml-sha",
    "--cargo-lock",
    path.join(repository, "deepwell/Cargo.lock"),
  ]);
  return sha(source);
}

function currentImages() {
  const serviceImage = (service) =>
    output("docker", [
      "inspect",
      `wikijump-standing-${service}-1`,
      "--format",
      "{{.Config.Image}}",
    ]);
  return {
    database: serviceImage("database"),
    cache: serviceImage("cache"),
    files: serviceImage("files"),
    deepwell: serviceImage("deepwell"),
    framerail: serviceImage("framerail"),
  };
}

function composeDocument({
  project,
  images,
  labels,
  binary,
  config,
  migrations,
  locales,
  deepwellPort,
  framerailPort,
}) {
  const labelLines = Object.entries(labels)
    .map(([key, value]) => `      ${key}: ${JSON.stringify(value)}`)
    .join("\n");
  return `name: ${project}
services:
  database:
    image: ${images.database}
    pull_policy: never
    environment:
      POSTGRES_DB: wikijump
      POSTGRES_USER: wikijump
      POSTGRES_PASSWORD: wikijump
      POSTGRES_HOST_AUTH_METHOD: md5
    volumes:
      - database:/var/lib/postgresql/data
    labels:
${labelLines}
      com.rokurolize.wikijump.role: database
    healthcheck:
      test: ["CMD", "wikijump-health-check"]
      interval: 5s
      timeout: 3s
      retries: 24
  cache:
    image: ${images.cache}
    pull_policy: never
    volumes:
      - cache:/data
    labels:
${labelLines}
      com.rokurolize.wikijump.role: cache
    healthcheck:
      test: ["CMD", "valkey-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 24
  files:
    image: ${images.files}
    pull_policy: never
    environment:
      MINIO_ROOT_USER: minio
      MINIO_ROOT_PASSWORD: defaultpassword
      MINIO_REGION_NAME: local
      INITIAL_BUCKETS: deepwell-files deepwell-text-blocks
      DATA_DIR: /data
    volumes:
      - files:/data
    labels:
${labelLines}
      com.rokurolize.wikijump.role: files
    healthcheck:
      test: ["CMD", "/healthcheck.sh"]
      interval: 5s
      timeout: 3s
      retries: 24
  deepwell:
    image: ${images.deepwell}
    pull_policy: never
    # The production image's normal entrypoint runs these migrations before
    # launching Deepwell. This controller replaces that entrypoint with the
    # candidate binary, so preserve that ordering explicitly. PostgreSQL can
    # become network-healthy while its first-run init scripts are still active.
    command: ["/bin/sh", "-ec", "until /usr/local/cargo/bin/sqlx migrate run --source /opt/marker/migrations; do sleep 1; done; exec /opt/marker/deepwell /opt/marker/config.toml"]
    environment:
      DATABASE_URL: postgres://wikijump:wikijump@database/wikijump
      REDIS_URL: redis://cache
      S3_FILES_BUCKET: deepwell-files
      S3_TEXT_BLOCKS_BUCKET: deepwell-text-blocks
      S3_REGION_NAME: local
      S3_PATH_STYLE: "true"
      S3_CUSTOM_ENDPOINT: http://files:9000
      S3_ACCESS_KEY_ID: minio
      S3_SECRET_ACCESS_KEY: defaultpassword
    ports:
      - "127.0.0.1:${deepwellPort}:2747"
    volumes:
      - type: bind
        source: ${JSON.stringify(binary)}
        target: /opt/marker/deepwell
        read_only: true
      - type: bind
        source: ${JSON.stringify(config)}
        target: /opt/marker/config.toml
        read_only: true
      - type: bind
        source: ${JSON.stringify(migrations)}
        target: /opt/marker/migrations
        read_only: true
      - type: bind
        source: ${JSON.stringify(locales)}
        target: /opt/locales
        read_only: true
    labels:
${labelLines}
      com.rokurolize.wikijump.role: deepwell
    healthcheck:
      test: ["CMD", "wikijump-health-check"]
      interval: 5s
      timeout: 3s
      retries: 60
    depends_on:
      database:
        condition: service_healthy
      cache:
        condition: service_healthy
      files:
        condition: service_healthy
  framerail:
    image: ${images.framerail}
    pull_policy: never
    environment:
      DEEPWELL_HOST: deepwell
      FRAMERAIL_MODE: built
      FRAMERAIL_ENV: local
      REDIS_URL: redis://cache
    ports:
      - "127.0.0.1:${framerailPort}:3393"
    labels:
${labelLines}
      com.rokurolize.wikijump.role: framerail
    healthcheck:
      test: ["CMD", "node", "-e", "const net=require('node:net');const socket=net.connect(3393,'127.0.0.1',()=>{socket.destroy();process.exit(0)});socket.on('error',()=>process.exit(1));setTimeout(()=>process.exit(1),1500)"]
      interval: 5s
      timeout: 3s
      retries: 60
    depends_on:
      deepwell:
        condition: service_healthy
volumes:
  database:
    name: ${project}-database
  cache:
    name: ${project}-cache
  files:
    name: ${project}-files
`;
}

async function rpc(url, method, params = {}, headers = {}) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json", ...headers },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: crypto.randomUUID(),
      method,
      params,
    }),
    signal: AbortSignal.timeout(60_000),
  });
  const body = await response.json();
  if (!response.ok || body.error)
    throw new Error(`${method}: ${JSON.stringify(body.error ?? body)}`);
  return body.result;
}

async function seedFixtures({ rpcUrl, fixtures, expectedFtml }) {
  const site = await rpc(rpcUrl, "site_get", { site: fixtures.site_slug });
  assert.ok(site?.site_id, `seeded ${fixtures.site_slug} site is missing`);
  const login = await rpc(rpcUrl, "login", {
    name_or_email: "admin@wikijump",
    password: "wikijumpadmin1",
    ip_address: "127.0.0.1",
    user_agent: OWNER,
  });
  const admin = await rpc(rpcUrl, "user_get", { user: "administrator" });
  assert.ok(admin?.user_id, "seeded administrator user is missing");
  const context = {
    "X-Deepwell-Session-Token": login.session_token,
    "X-Deepwell-Site-Id": String(site.site_id),
  };
  const results = [];
  for (const fixture of fixtures.fixtures) {
    let page = await rpc(rpcUrl, "page_get", {
      site_id: site.site_id,
      page: fixture.slug,
      details: { wikitext: true, compiled: true },
    });
    if (!page) {
      await rpc(
        rpcUrl,
        "page_create",
        {
          site_id: site.site_id,
          slug: fixture.slug,
          title: fixture.title,
          wikitext: fixture.wikitext,
          layout: fixtures.layout,
          user_id: admin.user_id,
          ip_address: "127.0.0.1",
          tags: [],
          revision_comments: "FTML marker contract fixture",
        },
        { ...context, "X-Deepwell-Page": fixture.slug },
      );
      page = await rpc(rpcUrl, "page_get", {
        site_id: site.site_id,
        page: fixture.slug,
        details: { wikitext: true, compiled: true },
      });
    }
    assert.ok(page, `fixture ${fixture.fixture_id} was not created`);
    await rpc(rpcUrl, "page_rerender", {
      site_id: site.site_id,
      category_id: page.page_category_id,
      page_id: page.page_id,
    });
    page = await rpc(rpcUrl, "page_get", {
      site_id: site.site_id,
      page: fixture.slug,
      details: { wikitext: true, compiled: true },
    });
    assert.ok(
      page.compiled_generator?.includes(expectedFtml.slice(0, 8)),
      `${fixture.fixture_id} generator did not identify ${expectedFtml}`,
    );
    results.push({
      fixture_id: fixture.fixture_id,
      slug: fixture.slug,
      compiled_generator: page.compiled_generator,
    });
  }
  return { site_id: site.site_id, results };
}

async function captureStage({ baseUrl, fixtures, siteId, outputDir, stage }) {
  const require = createRequire(
    path.join(REPOSITORY_ROOT, "framerail/package.json"),
  );
  const { chromium } = require("playwright");
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    extraHTTPHeaders: {
      "X-Wikijump-Site-Id": String(siteId),
      "X-Wikijump-Site-Slug": fixtures.site_slug,
    },
  });
  const evidence = [];
  try {
    for (const fixture of fixtures.fixtures) {
      const page = await context.newPage();
      try {
        const response = await page.goto(`${baseUrl}/${fixture.slug}`, {
          waitUntil: "domcontentloaded",
          timeout: 60_000,
        });
        assert.equal(
          response?.status(),
          200,
          `${stage}:${fixture.fixture_id} status`,
        );
        await page
          .locator("#page-content")
          .waitFor({ state: "attached", timeout: 60_000 });
        const pageContent = await page
          .locator("#page-content")
          .evaluate((root) => ({
            visibleText: (root.innerText ?? "").replace(/\s+/gu, " ").trim(),
            markerCount: root.querySelectorAll(
              "h1,h2,h3,h4,h5,h6,hr,div,span,[style*='text-align']",
            ).length,
          }));
        const directory = path.join(outputDir, fixture.fixture_id);
        await fs.mkdir(directory, { recursive: true, mode: 0o700 });
        const artifact = path.join(directory, "local.dom.html");
        await fs.writeFile(artifact, await page.content(), { mode: 0o600 });
        evidence.push({
          schema: "wikijump_full_parity.browser_rendering_record.v1",
          evidence_type: "browser_rendering",
          fixture_id: fixture.fixture_id,
          family: "ftml-marker-contract",
          slug: fixture.slug,
          source_url: `${baseUrl}/${fixture.slug}`,
          local_url: `${baseUrl}/${fixture.slug}`,
          source_browser_artifact: artifact,
          local_browser_artifact: artifact,
          source_visible_text: pageContent.visibleText,
          local_visible_text: pageContent.visibleText,
          source_status: 200,
          local_status: 200,
          capture_errors: [],
          marker_count: pageContent.markerCount,
        });
      } finally {
        await page.close();
      }
    }
  } finally {
    await context.close();
    await browser.close();
  }
  const records = {
    schema: "wikijump_full_parity.browser_rendering_evidence.v1",
    stage,
    selected_count: evidence.length,
    evidence,
  };
  await writeJson(path.join(outputDir, "records.json"), records);
  return records;
}

function makeCatalog(baselineDir, fixtures) {
  return {
    schema: "wikijump.ftml_marker_contract_catalog.v1",
    pairs: fixtures.fixtures.map((fixture) => ({
      fixture_id: fixture.fixture_id,
      family: "ftml-marker-contract",
      slug: fixture.slug,
      expected_verdict: "match",
      evidence_directory: baselineDir,
    })),
  };
}

async function main() {
  const args = parseArgs(process.argv);
  try {
    await runCanary(args);
  } catch (error) {
    await writeJson(path.join(args.outputDir, "failure.json"), {
      schema: "wikijump.ftml_marker_contract_canary_failure.v1",
      status: "fail",
      candidate_ftml: args.candidateFtml,
      error: error?.message ?? String(error),
      resource_disposition:
        "temporary worktrees, targets, containers, named volumes, and images are deleted by the controller finally block; no standing or corpus resource is a controller input",
    }).catch(() => {});
    throw error;
  }
}

async function runCanary(args) {
  const fixtures = JSON.parse(await fs.readFile(FIXTURES_PATH, "utf8"));
  assert.deepEqual(
    [...new Set(fixtures.fixtures.map((fixture) => fixture.surface))].sort(),
    [...REQUIRED_SURFACES].sort(),
    "fixture surfaces must be exactly the marker contract",
  );
  const baselineFtml =
    args.baselineFtml ?? (args.dryRun ? null : currentFtmlSha(REPOSITORY_ROOT));
  const runId = `ftml-marker-${args.candidateFtml.slice(0, 8)}-${crypto.randomUUID().slice(0, 8)}`;
  const expiresAt = new Date(
    Date.now() + EXPIRY_HOURS * 60 * 60 * 1000,
  ).toISOString();
  const layout = {
    run_id: runId,
    owner: OWNER,
    expires_at: expiresAt,
    baseline_ftml: baselineFtml,
    candidate_ftml: args.candidateFtml,
    required_surfaces: REQUIRED_SURFACES,
    resource_disposition: "delete-on-close",
  };
  if (args.dryRun) {
    process.stdout.write(
      `${JSON.stringify({ ...layout, fixtures: fixtures.fixtures.map(({ fixture_id, slug, surface }) => ({ fixture_id, slug, surface })) }, null, 2)}\n`,
    );
    return;
  }
  await fs.mkdir(args.outputDir, { recursive: true, mode: 0o700 });
  await fs.mkdir(args.workRoot, { recursive: true, mode: 0o700 });
  const runRoot = await fs.mkdtemp(path.join(args.workRoot, `${runId}-`));
  const baselineWorktree = path.join(runRoot, "baseline");
  const candidateWorktree = path.join(runRoot, "candidate");
  const targetRoot = path.join(runRoot, "targets");
  const stackRoot = path.join(runRoot, "stack");
  let project = null;
  let composePath = null;
  try {
    run("git", ["worktree", "add", "--detach", baselineWorktree, "HEAD"]);
    run("git", ["worktree", "add", "--detach", candidateWorktree, "HEAD"]);
    run("git", [
      "worktree",
      "lock",
      "--reason",
      `owner=${OWNER}; expiry=${expiresAt}`,
      baselineWorktree,
    ]);
    run("git", [
      "worktree",
      "lock",
      "--reason",
      `owner=${OWNER}; expiry=${expiresAt}`,
      candidateWorktree,
    ]);
    run("cargo", ["update", "-p", "ftml", "--precise", args.candidateFtml], {
      cwd: path.join(candidateWorktree, "deepwell"),
    });
    assert.equal(
      currentFtmlSha(baselineWorktree),
      baselineFtml,
      "baseline FTML changed while preparing canary",
    );
    assert.equal(
      currentFtmlSha(candidateWorktree),
      args.candidateFtml,
      "candidate FTML did not resolve exactly",
    );
    run(LEASE, ["status"]);
    const baselineManifest = path.join(runRoot, "baseline.manifest.json");
    const candidateManifest = path.join(runRoot, "candidate.manifest.json");
    await runCandidateBuild([
      "--repo",
      baselineWorktree,
      "--target-dir",
      path.join(targetRoot, "baseline"),
      "--profile",
      "dev",
      "--manifest",
      baselineManifest,
    ]);
    await runCandidateBuild([
      "--repo",
      candidateWorktree,
      "--target-dir",
      path.join(targetRoot, "candidate"),
      "--profile",
      "dev",
      "--manifest",
      candidateManifest,
    ]);
    const images = currentImages();
    const labels = {
      "com.rokurolize.wikijump.owner": OWNER,
      "com.rokurolize.wikijump.expiry": expiresAt,
      "com.rokurolize.wikijump.run_id": runId,
      "com.rokurolize.wikijump.lifecycle": "candidate-unpromoted",
    };
    const baselinePorts = {
      deepwell: await freePort(),
      framerail: await freePort(),
    };
    const candidatePorts = {
      deepwell: await freePort(),
      framerail: await freePort(),
    };
    const config = path.join(stackRoot, "config.toml");
    await fs.mkdir(stackRoot, { recursive: true, mode: 0o700 });
    const localConfig = await fs.readFile(
      path.join(REPOSITORY_ROOT, "install/local/deepwell/config.toml"),
      "utf8",
    );
    await fs.writeFile(
      config,
      localConfig.replace('pid-file = "/run/deepwell.pid"', 'pid-file = ""'),
      { mode: 0o644 },
    );
    const runStage = async (stage, worktree, manifest, ports) => {
      project = `${runId}-${stage}`;
      composePath = path.join(stackRoot, `${stage}.compose.yaml`);
      const binary = JSON.parse(await fs.readFile(manifest, "utf8")).build
        .binary_path_at_build;
      await fs.writeFile(
        composePath,
        composeDocument({
          project,
          images,
          labels,
          binary,
          config,
          migrations: path.join(worktree, "deepwell", "migrations"),
          locales: path.join(worktree, "locales"),
          deepwellPort: ports.deepwell,
          framerailPort: ports.framerail,
        }),
        { mode: 0o600 },
      );
      try {
        run("docker", [
          "compose",
          "-p",
          project,
          "-f",
          composePath,
          "up",
          "--detach",
          "--wait",
          "--wait-timeout",
          "600",
        ]);
      } catch (error) {
        const logs = spawnSync(
          "docker",
          ["compose", "-p", project, "-f", composePath, "logs", "--no-color"],
          { encoding: "utf8" },
        );
        await fs.writeFile(
          path.join(args.outputDir, `${stage}-stack-failure.log`),
          `${logs.stdout}\n${logs.stderr}`,
          { mode: 0o600 },
        );
        throw error;
      }
      const stageFtml =
        stage === "baseline" ? baselineFtml : args.candidateFtml;
      let seeded;
      let records;
      try {
        seeded = await seedFixtures({
          rpcUrl: `http://127.0.0.1:${ports.deepwell}/jsonrpc`,
          fixtures,
          expectedFtml: stageFtml,
        });
        records = await captureStage({
          baseUrl: `http://127.0.0.1:${ports.framerail}`,
          fixtures,
          siteId: seeded.site_id,
          outputDir: path.join(args.outputDir, stage),
          stage,
        });
      } catch (error) {
        const logs = spawnSync(
          "docker",
          ["compose", "-p", project, "-f", composePath, "logs", "--no-color"],
          { encoding: "utf8" },
        );
        await fs.writeFile(
          path.join(args.outputDir, `${stage}-stack-failure.log`),
          `${logs.stdout}\n${logs.stderr}`,
          { mode: 0o600 },
        );
        throw error;
      }
      run("docker", [
        "compose",
        "-p",
        project,
        "-f",
        composePath,
        "down",
        "--volumes",
        "--remove-orphans",
      ]);
      project = null;
      return { seeded, records, ports };
    };
    const baseline = await runStage(
      "baseline",
      baselineWorktree,
      baselineManifest,
      baselinePorts,
    );
    const candidate = await runStage(
      "candidate",
      candidateWorktree,
      candidateManifest,
      candidatePorts,
    );
    const catalogPath = path.join(args.outputDir, "catalog.json");
    await writeJson(
      catalogPath,
      makeCatalog(path.join(args.outputDir, "baseline"), fixtures),
    );
    const comparisonDir = path.join(args.outputDir, "comparison");
    run(process.execPath, [
      path.join(
        REPOSITORY_ROOT,
        "install/local/wikidot-verification/scripts/compare-render-evidence.mjs",
      ),
      "--pairs",
      catalogPath,
      "--mode",
      "records",
      "--records",
      path.join(args.outputDir, "candidate/records.json"),
      "--output-dir",
      comparisonDir,
      "--run-id",
      runId,
    ]);
    const comparison = JSON.parse(
      await fs.readFile(path.join(comparisonDir, "verdict.json"), "utf8"),
    );
    assert.equal(
      comparison.aggregate.regressions.length,
      0,
      "V3 comparison found marker-contract regressions",
    );
    await writeJson(path.join(args.outputDir, "canary-summary.json"), {
      ...layout,
      status: "pass",
      baseline: { manifest: baselineManifest, ...baseline },
      candidate: { manifest: candidateManifest, ...candidate },
      comparator: {
        script:
          "install/local/wikidot-verification/scripts/compare-render-evidence.mjs",
        verdict: path.join(comparisonDir, "verdict.json"),
      },
    });
    process.stdout.write(
      `${JSON.stringify({ status: "pass", run_id: runId, baseline_ftml: baselineFtml, candidate_ftml: args.candidateFtml, comparator: path.join(comparisonDir, "verdict.json") })}\n`,
    );
  } finally {
    if (project && composePath)
      spawnSync(
        "docker",
        [
          "compose",
          "-p",
          project,
          "-f",
          composePath,
          "down",
          "--volumes",
          "--remove-orphans",
        ],
        { stdio: "inherit" },
      );
    for (const worktree of [baselineWorktree, candidateWorktree]) {
      spawnSync("git", ["worktree", "unlock", worktree], {
        cwd: REPOSITORY_ROOT,
      });
      spawnSync("git", ["worktree", "remove", "--force", worktree], {
        cwd: REPOSITORY_ROOT,
      });
    }
    await fs.rm(runRoot, { recursive: true, force: true });
  }
}

main().catch((error) => {
  console.error(error.stack ?? error);
  process.exitCode = 1;
});
