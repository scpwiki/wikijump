#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import fsp from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import {spawnSync} from "node:child_process";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {sha256} from "../src/syntax-differential.mjs";

const BUILD_CANDIDATE = "/home/roku/wjlab/scripts/build-deepwell-candidate.sh";
const OWNER = "generic-runtime-differential";

function valueAfter(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    repository: null,
    cases: null,
    captures: [],
    externalReferences: [],
    output: null,
    site: "sandbox-for-codex",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--repository") args.repository = path.resolve(valueAfter(argv, index++, option));
    else if (option === "--cases") args.cases = path.resolve(valueAfter(argv, index++, option));
    else if (option === "--captures") args.captures.push(path.resolve(valueAfter(argv, index++, option)));
    else if (option === "--external-reference") {
      args.externalReferences.push(path.resolve(valueAfter(argv, index++, option)));
    } else if (option === "--output") args.output = path.resolve(valueAfter(argv, index++, option));
    else if (option === "--site") args.site = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  for (const field of ["repository", "cases", "output"]) {
    if (!args[field]) throw new Error(`--${field} is required`);
  }
  if (args.captures.length === 0) throw new Error("--captures is required");
  if (args.site !== "sandbox-for-codex") throw new Error("--site must be sandbox-for-codex");
  return args;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: options.stdio ?? "pipe",
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr || result.stdout}`);
  }
  return result.stdout.trim();
}

async function freePort() {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close((error) => error ? reject(error) : resolve(address.port));
    });
  });
}

function standingImageId(service) {
  return run("docker", [
    "inspect",
    `wikijump-standing-${service}-1`,
    "--format",
    "{{.Image}}",
  ]);
}

function readAdministrator(repository) {
  const users = JSON.parse(
    fs.readFileSync(path.join(repository, "deepwell/seeder/users.json"), "utf8"),
  );
  const administrator = users.find((user) => user?.slug === "administrator");
  if (!administrator?.email || !administrator?.password) {
    throw new Error("seeded administrator credentials are unavailable");
  }
  return {email: administrator.email, password: administrator.password};
}

export function composeDocument({
  project,
  labels,
  images,
  binary,
  config,
  migrations,
  locales,
  seeder,
  port,
  credentials,
}) {
  const labelLines = Object.entries(labels)
    .map(([key, value]) => `      ${key}: ${JSON.stringify(value)}`)
    .join("\n");
  const volumeLabels = Object.entries(labels)
    .map(([key, value]) => `      ${key}: ${JSON.stringify(value)}`)
    .join("\n");
  const databaseUrl = new URL("postgres://database/wikijump");
  databaseUrl.username = "wikijump";
  databaseUrl.password = credentials.databasePassword;
  return `name: ${project}
services:
  database:
    image: ${images.database}
    pull_policy: never
    environment:
      POSTGRES_DB: wikijump
      POSTGRES_USER: wikijump
      POSTGRES_PASSWORD: ${JSON.stringify(credentials.databasePassword)}
      POSTGRES_HOST_AUTH_METHOD: md5
    volumes:
      - database:/var/lib/postgresql/data
    labels:
${labelLines}
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
    healthcheck:
      test: ["CMD", "valkey-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 24
  files:
    image: ${images.files}
    pull_policy: never
    environment:
      MINIO_ROOT_USER: ${JSON.stringify(credentials.filesAccessKey)}
      MINIO_ROOT_PASSWORD: ${JSON.stringify(credentials.filesSecretKey)}
      MINIO_REGION_NAME: local
      INITIAL_BUCKETS: deepwell-files deepwell-text-blocks
      DATA_DIR: /data
    volumes:
      - files:/data
    labels:
${labelLines}
    healthcheck:
      test: ["CMD", "/healthcheck.sh"]
      interval: 5s
      timeout: 3s
      retries: 24
  deepwell:
    image: ${images.deepwell}
    pull_policy: never
    command: ["/bin/sh", "-ec", "until /usr/local/cargo/bin/sqlx migrate run --source /opt/runtime/migrations; do sleep 1; done; exec /opt/runtime/deepwell /opt/runtime/config.toml"]
    environment:
      DATABASE_URL: ${JSON.stringify(databaseUrl.href)}
      REDIS_URL: redis://cache
      S3_FILES_BUCKET: deepwell-files
      S3_TEXT_BLOCKS_BUCKET: deepwell-text-blocks
      S3_REGION_NAME: local
      S3_PATH_STYLE: "true"
      S3_CUSTOM_ENDPOINT: http://files:9000
      S3_ACCESS_KEY_ID: ${JSON.stringify(credentials.filesAccessKey)}
      S3_SECRET_ACCESS_KEY: ${JSON.stringify(credentials.filesSecretKey)}
    ports:
      - "127.0.0.1:${port}:2747"
    volumes:
      - type: bind
        source: ${JSON.stringify(binary)}
        target: /opt/runtime/deepwell
        read_only: true
      - type: bind
        source: ${JSON.stringify(config)}
        target: /opt/runtime/config.toml
        read_only: true
      - type: bind
        source: ${JSON.stringify(migrations)}
        target: /opt/runtime/migrations
        read_only: true
      - type: bind
        source: ${JSON.stringify(locales)}
        target: /opt/locales
        read_only: true
      - type: bind
        source: ${JSON.stringify(seeder)}
        target: /src/deepwell/seeder
        read_only: true
    labels:
${labelLines}
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
volumes:
  database:
    name: ${project}-database
    labels:
${volumeLabels}
  cache:
    name: ${project}-cache
    labels:
${volumeLabels}
  files:
    name: ${project}-files
    labels:
${volumeLabels}
networks:
  default:
    name: ${project}-network
    labels:
${volumeLabels}
`;
}

export function runtimeIdentity(manifest, compose, config) {
  const source = manifest.source ?? manifest.before?.inputs;
  const build = manifest.build ?? manifest;
  const wikijumpSha = source?.wikijump_sha ?? source?.repository_sha ?? source?.repo_sha;
  const ftmlSha = source?.ftml_sha ?? manifest.ftml_sha;
  const lockHash = build?.cargo_lock_sha256?.after ?? build?.cargo_lock_sha256;
  const executableHash = build?.binary_sha256 ?? manifest.binary_sha256;
  for (const [name, value, length] of [
    ["Wikijump SHA", wikijumpSha, 40],
    ["FTML SHA", ftmlSha, 40],
    ["Cargo.lock hash", lockHash, 64],
    ["executable hash", executableHash, 64],
  ]) {
    if (typeof value !== "string" || !new RegExp(`^[0-9a-f]{${length}}$`, "u").test(value)) {
      throw new Error(`candidate manifest has no valid ${name}`);
    }
  }
  return {
    schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1",
    wikijump_sha: wikijumpSha,
    ftml_sha: ftmlSha,
    dependency_lock_sha256: lockHash,
    executable_sha256: executableHash,
    runtime_config_sha256: sha256(`${compose}\0${config}`),
  };
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (run("git", ["status", "--porcelain"], {cwd: args.repository}) !== "") {
    throw new Error("candidate repository must be clean");
  }
  if (fs.existsSync(args.output)) throw new Error(`output already exists: ${args.output}`);
  const runId = `runtime-diff-${crypto.randomUUID().slice(0, 12)}`;
  const expiresAt = new Date(Date.now() + 8 * 60 * 60 * 1000).toISOString();
  const runRoot = await fsp.mkdtemp(path.join(os.tmpdir(), `${runId}-`));
  const project = runId;
  const composePath = path.join(runRoot, "compose.yaml");
  const configPath = path.join(runRoot, "config.toml");
  const manifestPath = path.join(runRoot, "candidate-manifest.json");
  const identityPath = path.join(runRoot, "runtime-identity.json");
  const targetPath = path.join(runRoot, "target");
  const port = await freePort();
  const labels = {
    "com.rokurolize.wikijump.owner": OWNER,
    "com.rokurolize.wikijump.expiry": expiresAt,
    "com.rokurolize.wikijump.run_id": runId,
    "com.rokurolize.wikijump.lifecycle": "delete-on-close",
  };
  const credentials = {
    databasePassword: crypto.randomBytes(32).toString("hex"),
    filesAccessKey: `runtime${crypto.randomBytes(12).toString("hex")}`,
    filesSecretKey: crypto.randomBytes(32).toString("hex"),
  };
  let composeStarted = false;
  try {
    run(BUILD_CANDIDATE, [
      "--repo", args.repository,
      "--target-dir", targetPath,
      "--profile", "dev",
      "--manifest", manifestPath,
    ]);
    const manifest = JSON.parse(await fsp.readFile(manifestPath, "utf8"));
    const binary = manifest.build?.binary_path_at_build;
    if (!binary || sha256(await fsp.readFile(binary)) !== manifest.build?.binary_sha256) {
      throw new Error("candidate binary does not match its manifest");
    }
    const localConfig = await fsp.readFile(
      path.join(args.repository, "install/local/deepwell/config.toml"),
      "utf8",
    );
    const config = localConfig.replace('pid-file = "/run/deepwell.pid"', 'pid-file = ""');
    await fsp.writeFile(configPath, config, {mode: 0o600});
    const compose = composeDocument({
      project,
      labels,
      images: {
        database: standingImageId("database"),
        cache: standingImageId("cache"),
        files: standingImageId("files"),
        deepwell: standingImageId("deepwell"),
      },
      binary,
      config: configPath,
      migrations: path.join(args.repository, "deepwell/migrations"),
      locales: path.join(args.repository, "locales"),
      seeder: path.join(args.repository, "deepwell/seeder"),
      port,
      credentials,
    });
    await fsp.writeFile(composePath, compose, {mode: 0o600});
    const identity = runtimeIdentity(manifest, compose, config);
    await fsp.writeFile(identityPath, `${JSON.stringify(identity, null, 2)}\n`, {mode: 0o600});
    run("docker", [
      "compose", "-p", project, "-f", composePath,
      "up", "--detach", "--wait", "--wait-timeout", "600", "deepwell",
    ]);
    composeStarted = true;
    const ratingUpdate = run("docker", [
      "compose", "-p", project, "-f", composePath,
      "exec", "--no-TTY", "--user", "wikijump",
      "database", "psql",
      "--dbname", "wikijump",
      "--set", "ON_ERROR_STOP=1",
      "--command",
      "UPDATE page_category SET rating_type = 'plus' WHERE site_id = (SELECT site_id FROM site WHERE slug = 'sandbox-for-codex') AND slug = '_default';",
    ]);
    if (!ratingUpdate.endsWith("UPDATE 1")) {
      throw new Error("sandbox oracle rating state did not update exactly one category");
    }
    const administrator = readAdministrator(args.repository);
    const runnerArgs = [
      path.join(path.dirname(new URL(import.meta.url).pathname), "run-generic-runtime-differential.mjs"),
      "--cases", args.cases,
      ...args.captures.flatMap((file) => ["--captures", file]),
      ...args.externalReferences.flatMap((file) => ["--external-reference", file]),
      "--runtime-identity", identityPath,
      "--rpc-url", `http://127.0.0.1:${port}/jsonrpc`,
      "--site", args.site,
      "--output", args.output,
    ];
    const result = spawnSync(process.execPath, runnerArgs, {
      cwd: args.repository,
      encoding: "utf8",
      env: {
        ...process.env,
        WIKIDOT_VERIFY_ADMIN_EMAIL: administrator.email,
        WIKIDOT_VERIFY_ADMIN_PASS: administrator.password,
      },
    });
    if (!fs.existsSync(args.output)) {
      throw new Error(`runtime differential produced no report: ${result.stderr || result.stdout}`);
    }
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    return result.status ?? 2;
  } finally {
    if (composeStarted) {
      const logs = spawnSync(
        "docker",
        ["compose", "-p", project, "-f", composePath, "logs", "--no-color"],
        {encoding: "utf8"},
      );
      await fsp.writeFile(`${args.output}.stack.log`, `${logs.stdout}\n${logs.stderr}`, {mode: 0o600})
        .catch(() => {});
    }
    if (fs.existsSync(composePath)) {
      spawnSync("docker", [
        "compose", "-p", project, "-f", composePath,
        "down", "--volumes", "--remove-orphans",
      ], {encoding: "utf8"});
    }
    await fsp.rm(runRoot, {recursive: true, force: true});
  }
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error.stack ?? error);
    return 2;
  },
});
