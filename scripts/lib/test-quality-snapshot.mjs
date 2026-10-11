// Dependency-free source identity snapshot for docs/development/test-quality-audit.json.
// The snapshot pins every production source and frozen input hash to one exact Git commit
// and tree. Ordinary working-tree drift is reported as staleness; it is not a verification failure.
import {spawnSync} from "node:child_process";
import {createHash} from "node:crypto";
import {existsSync, readFileSync, realpathSync} from "node:fs";
import {isAbsolute, relative, resolve} from "node:path";

export const SNAPSHOT_SCHEMA = "wikijump.test_quality_snapshot.v1";
export const SNAPSHOT_SCOPE = [
  "deepwell/", "framerail/", "install/local/wikidot-verification/", "install/standing/", "scripts/",
];
export const REQUIRED_INPUTS = [
  "deepwell/Cargo.toml", "deepwell/Cargo.lock", "deepwell/relation-impl-derive/Cargo.toml",
  "framerail/package.json", "framerail/pnpm-lock.yaml", "install/local/wikidot-verification/package.json",
  "install/local/wikidot-verification/pnpm-lock.yaml", ".cargo/config.toml", "rust-toolchain.toml",
];
export const REFRESH_COMMAND = "node scripts/refresh-test-quality-snapshot.mjs --commit <audited-commit>";

const HEX64 = /^[0-9a-f]{64}$/u;
const HEX40 = /^[0-9a-f]{40}$/u;
const fail = (message) => {
  throw new Error(message);
};

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function repositoryRelative(file, label) {
  if (typeof file !== "string" || !file || isAbsolute(file) || file.includes("\\")) {
    fail(`${label} must be a repository-relative path: ${file}`);
  }
  if (file.split("/").some((part) => part === "..")) fail(`${label} escapes repository: ${file}`);
  if (file.split("/").some((part) => part === "" || part === ".")) {
    fail(`${label} must be a normalized repository-relative path: ${file}`);
  }
  return file;
}

function inScope(file) {
  return SNAPSHOT_SCOPE.some((prefix) => file.startsWith(prefix));
}

function refreshHint(commit) {
  return `regenerate with: node scripts/refresh-test-quality-snapshot.mjs --commit ${commit ?? "<audited-commit>"}`;
}

function git(root, args, input) {
  return spawnSync("git", args, {cwd: root, input, maxBuffer: 1 << 30});
}

export function resolveCommit(root, revision) {
  if (typeof revision !== "string" || revision.startsWith("-")) return null;
  const result = git(root, ["rev-parse", "--verify", "--quiet", `${revision}^{commit}`]);
  return result.status === 0 ? result.stdout.toString("utf8").trim() : null;
}

function hasCommit(root, commit) {
  return git(root, ["cat-file", "-e", `${commit}^{commit}`]).status === 0;
}

// GitHub checkouts are shallow, so the pinned commit may need an explicit single-commit fetch.
// Local runs never fetch implicitly; they fail with an actionable message instead.
export function ensurePinnedCommit(root, commit) {
  if (hasCommit(root, commit)) return;
  if (process.env.GITHUB_ACTIONS === "true") {
    const fetched = git(root, ["fetch", "--no-tags", "--depth=1", "origin", commit]);
    if (fetched.status === 0 && hasCommit(root, commit)) return;
  }
  fail(`pinned audit commit is unavailable: ${commit}; fetch its history or ${refreshHint(commit)}`);
}

// Returns Map<path, sha256> for blobs in the commit. Paths absent from the commit are omitted.
export function committedFileHashes(root, commit, paths) {
  const listing = git(root, ["ls-tree", "-r", "-z", commit]);
  if (listing.status !== 0) fail(`cannot read pinned audit commit tree: ${commit}`);
  const objects = new Map();
  for (const entry of listing.stdout.toString("utf8").split("\0")) {
    const tab = entry.indexOf("\t");
    if (tab < 0) continue;
    const [, type, object] = entry.slice(0, tab).split(" ");
    if (type === "blob") objects.set(entry.slice(tab + 1), object);
  }
  const wanted = paths.filter((file) => objects.has(file));
  const hashes = new Map();
  if (wanted.length === 0) return hashes;
  const batch = git(root, ["cat-file", "--batch"], wanted.map((file) => `${objects.get(file)}\n`).join(""));
  if (batch.status !== 0) fail(`cannot read pinned audit blobs at ${commit}`);
  const output = batch.stdout;
  let offset = 0;
  for (const file of wanted) {
    const eol = output.indexOf(0x0a, offset);
    const header = output.toString("utf8", offset, eol).split(" ");
    if (header.length !== 3 || header[1] !== "blob") fail(`unexpected git object response for ${file}`);
    const start = eol + 1;
    const size = Number(header[2]);
    hashes.set(file, sha256(output.subarray(start, start + size)));
    offset = start + size + 1;
  }
  return hashes;
}

export function commitTree(root, commit) {
  const result = git(root, ["rev-parse", "--verify", "--quiet", `${commit}^{tree}`]);
  return result.status === 0 ? result.stdout.toString("utf8").trim() : null;
}

// Verifies the structural claim and the pinned identity against Git objects.
// Returns the snapshot summary used by the gate and by the refresh command.
export function verifyPinnedSnapshot(root, ledger) {
  const snapshot = ledger.snapshot;
  if (!snapshot || typeof snapshot !== "object") {
    fail(`audit snapshot metadata is missing; ${refreshHint(null)}`);
  }
  if (snapshot.schema !== SNAPSHOT_SCHEMA) {
    fail(`audit snapshot schema is missing or unsupported; ${refreshHint(snapshot.commit)}`);
  }
  if (!HEX40.test(snapshot.commit ?? "") || !HEX40.test(snapshot.tree ?? "")) {
    fail(`audit snapshot has malformed pinned commit or tree identity; ${refreshHint(snapshot.commit)}`);
  }
  if (!Array.isArray(ledger.inputs) || !Array.isArray(ledger.records)) {
    fail("audit ledger is missing required collections");
  }
  const inputPaths = ledger.inputs.map((input) => repositoryRelative(input?.path, "audit input"));
  for (const required of REQUIRED_INPUTS) {
    if (!inputPaths.includes(required)) {
      fail(`audit snapshot is missing required input: ${required}; ${refreshHint(snapshot.commit)}`);
    }
  }
  for (const input of inputPaths) {
    if (!REQUIRED_INPUTS.includes(input)) fail(`audit snapshot has unexpected input: ${input}`);
  }
  const seen = new Set();
  for (const record of ledger.records) {
    const path = repositoryRelative(record?.path, "production source");
    if (seen.has(path)) fail(`duplicate production source identity: ${path}`);
    seen.add(path);
    if (!inScope(path)) fail(`production source is outside audit scope: ${path}`);
    if (!HEX64.test(record.sha256 ?? "")) fail(`production source requires a SHA-256 identity: ${path}`);
  }
  for (const input of ledger.inputs) {
    if (!HEX64.test(input.sha256 ?? "")) fail(`audit input requires a SHA-256 identity: ${input.path}`);
  }

  ensurePinnedCommit(root, snapshot.commit);
  if (commitTree(root, snapshot.commit) !== snapshot.tree) {
    fail(`audit snapshot tree does not match pinned commit ${snapshot.commit}; ${refreshHint(snapshot.commit)}`);
  }
  const paths = [...inputPaths, ...ledger.records.map((record) => record.path)];
  const committed = committedFileHashes(root, snapshot.commit, paths);
  for (const entry of [...ledger.inputs, ...ledger.records]) {
    if (!committed.has(entry.path)) {
      fail(`pinned audit commit has no in-scope file ${entry.path}; ${refreshHint(snapshot.commit)}`);
    }
    if (committed.get(entry.path) !== entry.sha256) {
      fail(`pinned audit snapshot hash mismatch: ${entry.path} at ${snapshot.commit}; ${refreshHint(snapshot.commit)}`);
    }
  }
  return {commit: snapshot.commit, tree: snapshot.tree, inputs: ledger.inputs.length, records: ledger.records.length};
}

// Working-tree comparison. Missing or changed identities are reported, never treated as failures.
export function snapshotStaleness(root, ledger) {
  const stale = [];
  for (const entry of [...ledger.inputs, ...ledger.records]) {
    const target = resolve(root, entry.path);
    if (!existsSync(target)) {
      stale.push({path: entry.path, reason: "removed"});
      continue;
    }
    const actual = relative(root, realpathSync(target));
    if (!actual || actual === ".." || actual.startsWith("../") || isAbsolute(actual)) {
      fail(`working-tree source symlink points outside repository: ${entry.path}`);
    }
    if (sha256(readFileSync(target)) !== entry.sha256) stale.push({path: entry.path, reason: "changed"});
  }
  return stale;
}

// Pure projection of a generated inventory into the ledger. Review state for existing records is
// preserved; records are dropped only when they carry no review or finding state.
export function buildSnapshotLedger(ledger, inventory, {commit, tree}) {
  const current = new Map(ledger.records.map((record) => [record.path, record]));
  const production = inventory.entries.filter((entry) => entry.role === "production");
  const produced = new Set(production.map((entry) => entry.path));
  const reviewed = ledger.records.filter((record) => !produced.has(record.path) &&
    (record.status !== "pending" || record.findings?.length > 0 || record.review));
  if (reviewed.length > 0) {
    fail(`refresh would drop reviewed production records: ${reviewed.map((record) => record.path).join(", ")}`);
  }
  for (const owner of ledger.owners) {
    for (const file of owner.files) {
      if (!produced.has(file)) fail(`refresh would orphan executable owner ${owner.id}: ${file}`);
    }
  }
  const records = production.map((entry) => {
    const existing = current.get(entry.path);
    if (existing) return {...existing, sha256: entry.sha256};
    return {path: entry.path, sha256: entry.sha256, status: "pending", anchors: [], findings: [], review: null};
  });
  return {
    ...ledger,
    snapshot: {schema: SNAPSHOT_SCHEMA, commit, tree},
    inventory_digest: inventory.digest,
    inputs: inventory.inputs,
    records,
  };
}
