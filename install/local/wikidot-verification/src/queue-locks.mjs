import {mkdir, readFile, writeFile} from "node:fs/promises";
import path from "node:path";

const LOCK_RE = /^(repo|path|behavior|database|runtime|github|artifact):[^\0]+$/;
const SHA_RE = /^[0-9a-f]{40}$/;

function nowIso() {
  return new Date().toISOString();
}

function assertLockName(lockName) {
  if (typeof lockName !== "string" || !LOCK_RE.test(lockName)) {
    throw new Error(`invalid lock name: ${lockName}`);
  }
}

function lockKind(lockName) {
  return lockName.split(":", 1)[0];
}

function lockTarget(lockName) {
  return lockName.slice(lockName.indexOf(":") + 1);
}

function isWriteMode(mode) {
  return mode === "write" || mode === "exclusive";
}

function normalizeLease(lease) {
  if (lease === null || typeof lease !== "object" || Array.isArray(lease)) {
    throw new TypeError("lease must be an object");
  }
  assertLockName(lease.lock);
  if (!["read", "write", "exclusive"].includes(lease.mode)) {
    throw new Error("lease.mode must be read, write, or exclusive");
  }
  for (const field of ["task_id", "assignment_id"]) {
    if (typeof lease[field] !== "string" || lease[field].length === 0) {
      throw new Error(`${field} must be a non-empty string`);
    }
  }
  return {
    acquired_at: nowIso(),
    heartbeat_at: nowIso(),
    ...lease,
  };
}

function targetIsBelow(target, prefix) {
  return target === prefix || target.startsWith(`${prefix}/`);
}

function wildcardCovers(wildcardTarget, candidateTarget) {
  if (!wildcardTarget.endsWith("/**")) {
    return false;
  }
  return targetIsBelow(candidateTarget, wildcardTarget.slice(0, -3));
}

function pathPrefixesOverlap(left, right) {
  const leftTarget = lockTarget(left);
  const rightTarget = lockTarget(right);
  return (
    targetIsBelow(leftTarget, rightTarget) ||
    targetIsBelow(rightTarget, leftTarget) ||
    wildcardCovers(leftTarget, rightTarget) ||
    wildcardCovers(rightTarget, leftTarget)
  );
}

export function locksConflict(left, right) {
  const a = normalizeLease(left);
  const b = normalizeLease(right);
  if (a.assignment_id === b.assignment_id) {
    return false;
  }
  if (!isWriteMode(a.mode) && !isWriteMode(b.mode)) {
    return false;
  }
  const aKind = lockKind(a.lock);
  const bKind = lockKind(b.lock);
  if (aKind !== bKind) {
    return false;
  }
  if (aKind === "path") {
    return pathPrefixesOverlap(a.lock, b.lock);
  }
  return a.lock === b.lock;
}

export function findLockConflicts(existingLeases, requestedLeases) {
  const conflicts = [];
  const normalizedExisting = existingLeases.map(normalizeLease);
  const normalizedRequested = requestedLeases.map(normalizeLease);
  for (const requested of normalizedRequested) {
    for (const existing of normalizedExisting) {
      if (locksConflict(existing, requested)) {
        conflicts.push({existing, requested});
      }
    }
  }
  return conflicts;
}

function findRequestedBatchConflicts(requestedLeases) {
  const conflicts = [];
  const normalizedRequested = requestedLeases.map(normalizeLease);
  for (let left = 0; left < normalizedRequested.length; left += 1) {
    for (let right = left + 1; right < normalizedRequested.length; right += 1) {
      if (locksConflict(normalizedRequested[left], normalizedRequested[right])) {
        conflicts.push({
          existing: normalizedRequested[left],
          requested: normalizedRequested[right],
        });
      }
    }
  }
  return conflicts;
}

export function acquireLocks({existingLeases = [], requestedLeases = []} = {}) {
  const normalizedExisting = existingLeases.map(normalizeLease);
  const normalizedRequested = requestedLeases.map(normalizeLease);
  const conflicts = [
    ...findLockConflicts(normalizedExisting, normalizedRequested),
    ...findRequestedBatchConflicts(normalizedRequested),
  ];
  if (conflicts.length > 0) {
    return {
      acquired: false,
      conflicts,
      leases: normalizedExisting,
    };
  }
  return {
    acquired: true,
    conflicts: [],
    leases: [...normalizedExisting, ...normalizedRequested],
  };
}

export function classifyBaseFreshness({
  taskBaseSha,
  observedDevelopSha,
  mutable,
  allowHistoricalBase = false,
} = {}) {
  if (typeof taskBaseSha !== "string" || !SHA_RE.test(taskBaseSha)) {
    throw new Error("taskBaseSha must be a 40-character SHA1");
  }
  if (typeof observedDevelopSha !== "string" || !SHA_RE.test(observedDevelopSha)) {
    throw new Error("observedDevelopSha must be a 40-character SHA1");
  }
  if (typeof mutable !== "boolean") {
    throw new Error("mutable must be a boolean");
  }
  if (typeof allowHistoricalBase !== "boolean") {
    throw new Error("allowHistoricalBase must be a boolean");
  }
  if (taskBaseSha === observedDevelopSha) {
    return {state: "CURRENT", stale: false};
  }
  if (!mutable || allowHistoricalBase) {
    return {state: "HISTORICAL_ALLOWED", stale: false};
  }
  return {state: "STALE_BASE", stale: true};
}

export async function appendLockEvent({eventLogPath, event}) {
  await mkdir(path.dirname(eventLogPath), {recursive: true});
  const record = {
    ...event,
    time: nowIso(),
  };
  await writeFile(eventLogPath, `${JSON.stringify(record)}\n`, {flag: "a"});
  return record;
}

export async function readLockEvents(eventLogPath) {
  const text = await readFile(eventLogPath, "utf8").catch((error) => {
    if (error.code === "ENOENT") {
      return "";
    }
    throw error;
  });
  return text
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line));
}

export async function acquireLocksWithEvents({eventLogPath, existingLeases = [], requestedLeases = []}) {
  const result = acquireLocks({existingLeases, requestedLeases});
  await appendLockEvent({
    eventLogPath,
    event: {
      event: result.acquired ? "LOCKS_ACQUIRED" : "LOCK_CONFLICT",
      requested: requestedLeases,
      conflicts: result.conflicts,
    },
  });
  return result;
}
