import {execFile, execFileSync} from "node:child_process";
import {randomUUID} from "node:crypto";
import {mkdir, readFile, rename, rm, writeFile} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";
const SCHEMA_VERSION = 1;
const DEFAULT_TTL_MS = 30000;
const DEFAULT_COMPLETED_TTL_MS = 300000;
const GH_TIMEOUT_MS = 15000;
const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const GIT_ROOT = (() => {
  try {
    const root = execFileSync("git", ["-C", PACKAGE_ROOT, "rev-parse", "--show-toplevel"], {encoding: "utf8", stdio: ["ignore", "pipe", "ignore"]}).trim();
    return root.length > 0 ? root : null;
  } catch {
    return null;
  }
})();
class GhCommandError extends Error {
  constructor(message, details = {}) {
    super(message);
    Object.assign(this, {name: "GhCommandError", isToolFailure: true, args: details.args ?? [], exitCode: details.exitCode ?? null, stderr: details.stderr ?? "", stdout: details.stdout ?? ""});
    this.errorExcerpt = redactText(details.errorExcerpt ?? details.stderr ?? details.stdout ?? message);
  }
}
function shellQuote(value) {
  const text = String(value);
  return text.length === 0 ? "''" : `'${text.replaceAll("'", "'\\''")}'`;
}
function createRunId(nowMs) {
  return `${new Date(nowMs).toISOString().replace(/[:.]/g, "-")}-${randomUUID().slice(0, 8)}`;
}
function elapsedSince(startNs) {
  return Number(process.hrtime.bigint() - startNs) / 1_000_000;
}
function deepEqual(left, right) {
  if (left === right) {
    return true;
  }
  if (typeof left !== "object" || left === null || typeof right !== "object" || right === null) {
    return false;
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length && left.every((value, index) => deepEqual(value, right[index]));
  }
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  return leftKeys.length === rightKeys.length && leftKeys.every((key) => Object.hasOwn(right, key) && deepEqual(left[key], right[key]));
}
function subjectId(subject) {
  return subject.kind === "pr" ? {kind: "pr", prNumber: subject.prNumber} : subject.kind === "branch" ? {kind: "branch", branch: subject.branch} : {kind: "sha", sha: subject.sha};
}
function normalizeSubject(subject) {
  if (subject?.kind === "pr" && Number.isInteger(subject.prNumber) && subject.prNumber > 0) return {kind: "pr", prNumber: subject.prNumber};
  if (subject?.kind === "branch" && typeof subject.branch === "string" && subject.branch.length > 0) return {kind: "branch", branch: subject.branch};
  if (subject?.kind === "sha" && typeof subject.sha === "string" && subject.sha.length > 0) return {kind: "sha", sha: subject.sha};
  throw new Error("subject must be pr, branch, or sha");
}
function statusFileName(subject) {
  return subject.kind === "pr" ? `pr-${subject.prNumber}.json` : subject.kind === "branch" ? `branch-${subject.branch.replaceAll("/", "--")}.json` : `sha-${subject.sha.slice(0, 12)}.json`;
}
function parseCachedBranch(raw) {
  if (raw === null) return null;
  try {
    const artifact = JSON.parse(raw);
    if (artifact?.subject?.kind === "pr") return typeof artifact.subject.headRefName === "string" && artifact.subject.headRefName.length > 0 ? artifact.subject.headRefName : null;
    if (artifact?.subject?.kind === "branch") return typeof artifact.subject.branch === "string" && artifact.subject.branch.length > 0 ? artifact.subject.branch : null;
  } catch {
    return null;
  }
  return null;
}
function cacheTtlFor(overall, ttlMs, completedTtlMs) {
  return overall === "passing" || overall === "failing" ? completedTtlMs : ttlMs;
}
function withCacheFields(artifact, {cacheStatus, cacheReason, ageMs, expired, ttlMs, completedTtlMs}) {
  const fetchedMs = Date.parse(artifact.fetchedAt);
  const ttlForArtifact = cacheTtlFor(artifact.overall, ttlMs, completedTtlMs);
  return {...artifact, ttlMs, completedTtlMs, expiresAt: Number.isFinite(fetchedMs) ? new Date(fetchedMs + ttlForArtifact).toISOString() : artifact.expiresAt, ageMs, expired, cacheStatus, cacheReason};
}
async function writeJsonAtomically(statusPath, artifact, runId) {
  const directory = path.dirname(statusPath);
  const temporaryPath = path.join(directory, `${path.basename(statusPath)}.tmp-${process.pid}-${runId}`);
  await mkdir(directory, {recursive: true});
  try {
    await writeFile(temporaryPath, `${JSON.stringify(artifact, null, 2)}\n`, "utf8");
    await rename(temporaryPath, statusPath);
  } finally {
    await rm(temporaryPath, {force: true}).catch(() => {});
  }
}
function execGh(args) {
  return new Promise((resolve, reject) => {
    execFile("gh", args, {encoding: "utf8", timeout: GH_TIMEOUT_MS, maxBuffer: 1024 * 1024}, (error, stdout, stderr) => {
      if (error === null) {
        resolve(stdout);
        return;
      }
      const timedOut = error.killed === true && error.signal === "SIGTERM";
      reject(
        new GhCommandError(timedOut ? `gh timed out after ${GH_TIMEOUT_MS}ms` : "gh invocation failed", {
          args,
          exitCode: typeof error.code === "number" ? error.code : null,
          stderr: stderr ?? "",
          stdout: stdout ?? "",
          errorExcerpt: timedOut ? `timed out after ${GH_TIMEOUT_MS}ms` : stderr || stdout || error.message,
        }),
      );
    });
  });
}
function parseJsonOutput(stdout, args) {
  try {
    return JSON.parse(stdout);
  } catch {
    throw new GhCommandError("unparseable gh output", {args, stdout, errorExcerpt: "unparseable gh output"});
  }
}
function isProtectionUnreadable(error) {
  const text = `${error?.exitCode ?? ""} ${error?.message ?? ""} ${error?.stderr ?? ""} ${error?.stdout ?? ""}`;
  return /\b(?:403|404)\b/.test(text);
}
function normalizeStatus(value, conclusion = null) {
  const upper = typeof value === "string" ? value.toUpperCase() : "";
  if (upper === "COMPLETED" || (upper === "" && conclusion !== null)) return "completed";
  if (upper === "IN_PROGRESS" || upper === "IN PROGRESS") return "in_progress";
  return "queued";
}
function normalizeConclusion(value) {
  if (value === null || value === undefined || value === "") return null;
  return String(value).toUpperCase();
}
function statusContextConclusion(state) {
  const upper = normalizeConclusion(state);
  if (upper === "SUCCESS") return "SUCCESS";
  if (upper === "FAILURE" || upper === "ERROR") return "FAILURE";
  return null;
}
function isFailingCheck(check) {
  if (check.status !== "completed" || check.conclusion === null) {
    return false;
  }
  return check.conclusion !== "SUCCESS" && check.conclusion !== "NEUTRAL" && check.conclusion !== "SKIPPED";
}
function normalizeRollupEntry(entry, requiredContexts) {
  const name = entry?.name ?? entry?.context ?? "";
  const isStatusContext = entry?.__typename === "StatusContext" || entry?.context !== undefined;
  const conclusion = isStatusContext ? statusContextConclusion(entry.state) : normalizeConclusion(entry?.conclusion);
  const status = isStatusContext ? (conclusion === null ? "queued" : "completed") : normalizeStatus(entry?.status, conclusion);
  return {
    name: String(name),
    workflowName: entry?.workflowName ?? entry?.workflow?.name ?? null,
    status,
    conclusion,
    startedAt: entry?.startedAt ?? entry?.started_at ?? null,
    completedAt: entry?.completedAt ?? entry?.completed_at ?? null,
    detailsUrl: entry?.detailsUrl ?? entry?.targetUrl ?? entry?.html_url ?? null,
    required: requiredContexts?.has(String(name)) ?? false,
  };
}
function normalizeCheckRun(entry, requiredContexts) {
  const conclusion = normalizeConclusion(entry?.conclusion);
  const name = String(entry?.name ?? "");
  return {
    name,
    workflowName: null,
    status: normalizeStatus(entry?.status, conclusion),
    conclusion,
    startedAt: entry?.started_at ?? entry?.startedAt ?? null,
    completedAt: entry?.completed_at ?? entry?.completedAt ?? null,
    detailsUrl: entry?.html_url ?? entry?.detailsUrl ?? null,
    required: requiredContexts?.has(name) ?? false,
  };
}
function buildRequiredChecks(protection, checks) {
  const contexts = Array.isArray(protection?.contexts) ? protection.contexts.map(String) : null;
  const strict = typeof protection?.strict === "boolean" ? protection.strict : null;
  if (contexts === null) {
    return {
      branch: "develop",
      strict,
      contexts: null,
      passing: checks.filter((check) => check.conclusion === "SUCCESS").map((check) => check.name),
      pending: checks.filter((check) => check.status !== "completed").map((check) => check.name),
      failing: checks.filter(isFailingCheck).map((check) => check.name),
      missing: [],
      satisfied: null,
    };
  }
  const passing = [];
  const pending = [];
  const failing = [];
  const missing = [];
  for (const context of contexts) {
    const matches = checks.filter((check) => check.name === context);
    if (matches.length === 0) {
      missing.push(context);
    } else if (matches.some(isFailingCheck)) {
      failing.push(context);
    } else if (matches.some((check) => check.status === "completed" && check.conclusion === "SUCCESS")) {
      passing.push(context);
    } else {
      pending.push(context);
    }
  }
  return {branch: "develop", strict, contexts, passing, pending, failing, missing, satisfied: missing.length === 0 && pending.length === 0 && failing.length === 0};
}
function overallStatus(requiredChecks, checks) {
  if (requiredChecks.contexts === null) {
    // Unknown required-check set (protection unreadable): a required context
    // with no posted check would be invisible, so "passing" can never be
    // claimed here — report failing on visible failures, otherwise pending.
    return checks.some(isFailingCheck) ? "failing" : "pending";
  }
  if (requiredChecks.failing.length > 0) {
    return "failing";
  }
  if (requiredChecks.missing.length > 0 || requiredChecks.pending.length > 0) {
    return "pending";
  }
  return "passing";
}
function firstFailure(requiredChecks, checks) {
  const failing = checks.find((check) => (requiredChecks.contexts === null || check.required) && isFailingCheck(check));
  return failing === undefined ? null : {name: failing.name, conclusion: failing.conclusion};
}
function normalizeProtection(raw) {
  if (raw === null) {
    return {strict: null, contexts: null};
  }
  return {
    strict: typeof raw.strict === "boolean" ? raw.strict : null,
    contexts: Array.isArray(raw.contexts) ? raw.contexts.map(String) : null,
  };
}
async function fetchProtection(fetchers, repo) {
  try {
    return normalizeProtection(await fetchers.fetchProtection({repo}));
  } catch (error) {
    if (isProtectionUnreadable(error)) {
      return {strict: null, contexts: null};
    }
    throw error;
  }
}
function normalizeCheckRunsResponse(response) {
  return Array.isArray(response?.check_runs) ? response.check_runs : Array.isArray(response) ? response : [];
}
function createDefaultLocalGit() {
  return {
    resolveHeadSha(branchName) {
      if (GIT_ROOT === null || typeof branchName !== "string" || branchName.length === 0) {
        return null;
      }
      try {
        const head = execFileSync("git", ["rev-parse", `refs/heads/${branchName}`], {
          cwd: GIT_ROOT,
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
        }).trim();
        return /^[0-9a-f]{40}$/i.test(head) ? head : null;
      } catch {
        return null;
      }
    },
  };
}
export function redactText(text) {
  if (text === null || text === undefined) {
    return null;
  }
  return String(text)
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/)([^/\s@]+)@([^/\s?#]+)/gi, "$1[redacted]@[redacted]")
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/[^\s?#]+)(?:[?#][^\s]*)?/gi, "$1")
    .replace(/(^|[\s,;])([A-Za-z_][A-Za-z0-9_]*(?:PASSWORD|SECRET|TOKEN|KEY)[A-Za-z0-9_]*|PASSWORD|SECRET|TOKEN|KEY|DATABASE_URL|DB_URL|PG[A-Z]+)=(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi, "$1$2=[redacted]")
    .slice(0, 500);
}
export function defaultStatusPath(subject) {
  return path.join(PACKAGE_ROOT, "artifacts", "ci-status", statusFileName(normalizeSubject(subject)));
}
export function buildFingerprint(config) {
  return {
    schemaVersion: SCHEMA_VERSION,
    repo: config.repo,
    subject: subjectId(normalizeSubject(config.subject)),
    probeSetVersion: 1,
  };
}
export function evaluateCiCache({raw, nowMs, fingerprint, ttlMs, completedTtlMs, localHeadSha}) {
  if (raw === null) {
    return {cacheStatus: "miss", cacheReason: "no-file", artifact: null};
  }
  let artifact;
  try {
    artifact = JSON.parse(raw);
  } catch {
    return {cacheStatus: "miss", cacheReason: "invalid-json", artifact: null};
  }
  if (artifact.schemaVersion !== SCHEMA_VERSION) {
    return {cacheStatus: "miss", cacheReason: "schema-mismatch", artifact: null};
  }
  if (!deepEqual(artifact.fingerprint, fingerprint)) {
    return {cacheStatus: "miss", cacheReason: "fingerprint-mismatch", artifact: null};
  }
  const ageMs = nowMs - Date.parse(artifact.fetchedAt);
  if ((artifact.subject?.kind === "pr" || artifact.subject?.kind === "branch") && typeof localHeadSha === "string" && localHeadSha.length > 0 && artifact.subject?.headSha !== localHeadSha) {
    return {cacheStatus: "stale", cacheReason: "head-sha-changed", artifact: withCacheFields(artifact, {cacheStatus: "stale", cacheReason: "head-sha-changed", ageMs, expired: false, ttlMs, completedTtlMs})};
  }
  const ttlForArtifact = cacheTtlFor(artifact.overall, ttlMs, completedTtlMs);
  if (!Number.isFinite(ageMs) || ageMs > ttlForArtifact) {
    return {cacheStatus: "stale", cacheReason: "ttl-expired", artifact: withCacheFields(artifact, {cacheStatus: "stale", cacheReason: "ttl-expired", ageMs, expired: true, ttlMs, completedTtlMs})};
  }
  return {cacheStatus: "hit", cacheReason: "fresh", artifact: withCacheFields(artifact, {cacheStatus: "hit", cacheReason: "fresh", ageMs, expired: false, ttlMs, completedTtlMs})};
}
export function createGhFetchers() {
  return {
    async fetchPrView({repo, prNumber}) {
      const args = ["pr", "view", String(prNumber), "--repo", repo, "--json", "number,state,headRefName,headRefOid,mergeable,mergeStateStatus,statusCheckRollup"];
      return parseJsonOutput(await execGh(args), args);
    },
    async resolveBranchSha({repo, branch}) {
      const args = ["api", `repos/${repo}/commits/${encodeURIComponent(branch)}`, "--jq", ".sha"];
      const sha = (await execGh(args)).trim();
      if (!/^[0-9a-f]{40}$/i.test(sha)) {
        throw new GhCommandError("unparseable gh output", {args, stdout: sha, errorExcerpt: "unparseable gh output"});
      }
      return sha;
    },
    async fetchCheckRuns({repo, sha}) {
      const args = ["api", `repos/${repo}/commits/${sha}/check-runs?per_page=100`, "--jq", "{check_runs: [.check_runs[] | {name,status,conclusion,started_at,completed_at,html_url}]}"];
      return parseJsonOutput(await execGh(args), args);
    },
    async fetchProtection({repo}) {
      const args = ["api", `repos/${repo}/branches/develop/protection/required_status_checks`, "--jq", "{strict, contexts}"];
      try {
        return parseJsonOutput(await execGh(args), args);
      } catch (error) {
        if (isProtectionUnreadable(error)) {
          return null;
        }
        throw error;
      }
    },
  };
}
export async function collectCiStatus(options) {
  const nowMs = options.nowMs ?? Date.now();
  const repo = options.repo ?? "Rokurolize/wikijump";
  const subject = normalizeSubject(options.subject);
  const ttlMs = options.ttlMs ?? DEFAULT_TTL_MS;
  const completedTtlMs = options.completedTtlMs ?? DEFAULT_COMPLETED_TTL_MS;
  const statusPath = path.resolve(options.statusPath ?? defaultStatusPath(subject));
  const fingerprint = buildFingerprint({repo, subject});
  const localGit = options.localGit ?? createDefaultLocalGit();
  if (options.refresh !== true) {
    let raw = null;
    try {
      raw = await readFile(statusPath, "utf8");
    } catch (error) {
      if (error.code !== "ENOENT") {
        raw = null;
      }
    }
    const branchName = parseCachedBranch(raw);
    const localHeadSha = branchName === null ? null : localGit.resolveHeadSha(branchName);
    const cache = evaluateCiCache({raw, nowMs, fingerprint, ttlMs, completedTtlMs, localHeadSha});
    if (cache.cacheStatus === "hit") {
      return {...cache.artifact, artifactPaths: {...cache.artifact.artifactPaths, status: statusPath}};
    }
    options = {...options, cacheStatus: cache.cacheStatus, cacheReason: cache.cacheReason};
  } else {
    options = {...options, cacheStatus: "refresh", cacheReason: "refresh"};
  }
  const fetchers = options.fetchers ?? createGhFetchers();
  const byFetch = {view: 0, checkRuns: 0, protection: 0};
  const totalStartNs = process.hrtime.bigint();
  async function timedFetch(label, fn) {
    const startNs = process.hrtime.bigint();
    try {
      return await fn();
    } finally {
      byFetch[label] += elapsedSince(startNs);
    }
  }
  let checks = [];
  let prView = null;
  let headSha = null;
  let headRefName = null;
  if (subject.kind === "pr") {
    prView = await timedFetch("view", () => fetchers.fetchPrView({repo, prNumber: subject.prNumber}));
    headSha = prView.headRefOid ?? null;
    headRefName = prView.headRefName ?? null;
  } else if (subject.kind === "branch") {
    headSha = await timedFetch("view", () => fetchers.resolveBranchSha({repo, branch: subject.branch}));
  } else {
    headSha = subject.sha;
  }
  if (subject.kind !== "pr") {
    const response = await timedFetch("checkRuns", () => fetchers.fetchCheckRuns({repo, sha: headSha}));
    checks = normalizeCheckRunsResponse(response);
  }
  const protection = await timedFetch("protection", () => fetchProtection(fetchers, repo));
  const requiredContexts = Array.isArray(protection.contexts) ? new Set(protection.contexts) : null;
  checks = subject.kind === "pr" ? (Array.isArray(prView.statusCheckRollup) ? prView.statusCheckRollup : []).map((entry) => normalizeRollupEntry(entry, requiredContexts)) : checks.map((entry) => normalizeCheckRun(entry, requiredContexts));
  const requiredChecks = buildRequiredChecks(protection, checks);
  const overall = overallStatus(requiredChecks, checks);
  const runId = createRunId(nowMs);
  const fetchedAt = new Date(nowMs).toISOString();
  const ttlForArtifact = cacheTtlFor(overall, ttlMs, completedTtlMs);
  const artifact = {
    schemaVersion: SCHEMA_VERSION,
    runId,
    fingerprint,
    fetchedAt,
    ttlMs,
    completedTtlMs,
    expiresAt: new Date(nowMs + ttlForArtifact).toISOString(),
    ageMs: 0,
    expired: false,
    cacheStatus: options.cacheStatus,
    cacheReason: options.cacheReason,
    repo,
    subject: {kind: subject.kind, prNumber: subject.prNumber ?? null, branch: subject.branch ?? null, sha: subject.sha ?? null, headSha, headRefName},
    state: prView?.state ?? null,
    mergeable: prView?.mergeable ?? null,
    mergeStateStatus: prView?.mergeStateStatus ?? null,
    requiredChecks,
    checks,
    overall,
    firstFailure: firstFailure(requiredChecks, checks),
    elapsedMs: {total: elapsedSince(totalStartNs), byFetch},
    artifactPaths: {status: statusPath},
    rerunCommand: `cd ${shellQuote(PACKAGE_ROOT)} && node scripts/ci-status.mjs --repo ${shellQuote(repo)} --${subject.kind} ${shellQuote(subject.kind === "pr" ? subject.prNumber : subject.kind === "branch" ? subject.branch : subject.sha)} --refresh --json${options.statusPath === undefined ? "" : ` --status ${shellQuote(statusPath)}`}`,
  };
  await writeJsonAtomically(statusPath, artifact, runId);
  return artifact;
}
