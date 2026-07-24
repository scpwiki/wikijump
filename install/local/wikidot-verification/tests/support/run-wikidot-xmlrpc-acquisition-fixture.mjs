import {execFileSync, spawn} from "node:child_process";
import path from "node:path";
import {fileURLToPath, pathToFileURL} from "node:url";

import {sha256Hex, stableStringify} from "../../src/corpus-import-manifest.mjs";
import {buildReferenceAcquisitionInventory} from "../../src/reference-acquisition-inventory.mjs";

export function rawRows(count) {
  return Array.from({ length: count }, (_, index) => ({
    attachments: [],
    file_path: `/private/source/scp-${173 + index}/source.wikidot.txt`,
    fullname: `scp-${173 + index}`,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    required_browser: false,
    revisions: index + 1,
    source_branch: "en",
    source_entity_id: `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
    source_sha256: sha256Hex(`source-${index}\n`),
    source_site: "scp-wiki",
    updated_at: `2026-07-${String(index + 1).padStart(2, "0")}T12:34:56+00:00`,
  }));
}

export function summaryFor(rows, manifestBytes) {
  return Buffer.from(
    `${stableStringify({
      attachment_count: 0,
      attachment_page_count: 0,
      first_fullname: rows[0].fullname,
      last_fullname: rows.at(-1).fullname,
      manifest_sha256: sha256Hex(manifestBytes),
      parent_count: 0,
      required_browser_count: 0,
      row_count: rows.length,
      source_branches: ["en"],
      source_browser_visibility_counts: {},
      source_required_actor_count: 0,
      source_sites: ["scp-wiki"],
    })}\n`,
  );
}

export function fixture(count = 8) {
  const rows = rawRows(count);
  const manifestBytes = Buffer.from(
    `${rows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const summaryBytes = summaryFor(rows, manifestBytes);
  const fullInventory = buildReferenceAcquisitionInventory({
    expectedCount: rows.length,
    expectedManifestSha256: sha256Hex(manifestBytes),
    expectedSummarySha256: sha256Hex(summaryBytes),
    family: "EN",
    manifestBytes,
    shardCount: 4,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
  return { fullInventory, manifestBytes, rows, summaryBytes };
}

export function runnerOptions(overrides = {}) {
  return {
    "campaign-nonce": "00000000-0000-4000-8000-000000000001",
    "capsule-parent": "/tmp/capsules",
    "expected-full-inventory-sha256": "a".repeat(64),
    "expected-manifest-sha256": "b".repeat(64),
    "expected-summary-sha256": "c".repeat(64),
    "full-inventory": "/tmp/full-inventory.json",
    "inventory-output": "/tmp/inventory.json",
    manifest: "/tmp/manifest.jsonl",
    "principal-id": "5700026",
    "result-receipt": "/tmp/result.json",
    "runtime-python": "bin/python",
    "runtime-root": "/tmp/runtime",
    "runtime-venv-config": "pyvenv.cfg",
    "runtime-version": "3.13.13",
    "selection-count": "128",
    shards: "8",
    "source-commit": "1".repeat(40),
    "source-git-dir": "/tmp/source.git",
    "source-tree": "2".repeat(40),
    store: "/tmp/store",
    summary: "/tmp/summary.json",
    "throttle-receipt": "/tmp/throttle.json",
    verdict: "/tmp/verdict.json",
    "wikijump-commit": "3".repeat(40),
    "wikijump-git-dir": "/tmp/wikijump.git",
    "wikijump-tree": "4".repeat(40),
    ...overrides,
  };
}

export const REPOSITORY_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../../..",
);
export const LOCAL_IMPORT = /(?:\bfrom\s+|\bimport\s*\(\s*)["']([^"']+)["']/gu;
export const TEST_GIT_ENVIRONMENT = Object.freeze({
  GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
  GIT_AUTHOR_EMAIL: "oracle@example.invalid",
  GIT_AUTHOR_NAME: "Oracle",
  GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
  GIT_COMMITTER_EMAIL: "oracle@example.invalid",
  GIT_COMMITTER_NAME: "Oracle",
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_PAGER: "cat",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin",
});

export function git(cwd, args) {
  return execFileSync("/usr/bin/git", args, {
    cwd,
    encoding: "utf8",
    env: TEST_GIT_ENVIRONMENT,
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

export function runPrivateCoordinator(entrypoint, input, descriptor) {
  const program = [
    `import { runAcquisition } from ${JSON.stringify(pathToFileURL(entrypoint).href)};`,
    `const input = ${JSON.stringify(input)};`,
    "try {",
    "  await runAcquisition(input);",
    "  process.exitCode = 0;",
    "} catch (error) {",
    "  process.stdout.write(String(error?.message ?? error) + '\\n');",
    "  process.exitCode = 7;",
    "}",
  ].join("\n");
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      ["--input-type=module", "--eval", program],
      { stdio: ["ignore", "pipe", "pipe", "ignore", "pipe"] },
    );
    const stdout = [];
    const stderr = [];
    child.once("error", reject);
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("close", (code, signal) => {
      resolve({
        code,
        signal,
        stderr: Buffer.concat(stderr).toString("utf8"),
        stdout: Buffer.concat(stdout).toString("utf8"),
      });
    });
    child.stdio[4].once("error", () => {});
    child.stdio[4].end(JSON.stringify(descriptor));
  });
}
