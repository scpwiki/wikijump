import assert from "node:assert/strict";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  acquireLocks,
  acquireLocksWithEvents,
  appendLockEvent,
  classifyBaseFreshness,
  locksConflict,
  readLockEvents,
} from "../src/queue-locks.mjs";

async function tempDir(t) {
  const dir = await mkdtemp(path.join(os.tmpdir(), "wikijump-queue-locks-"));
  t.after(() => rm(dir, {recursive: true, force: true}));
  return dir;
}

function lease(name, mode, id = "a1") {
  return {lock: name, mode, task_id: `task-${id}`, assignment_id: id};
}

test("allows shared reads but blocks read/write overlap", () => {
  assert.equal(
    locksConflict(lease("behavior:workflow-grid", "read", "a1"), lease("behavior:workflow-grid", "read", "a2")),
    false,
  );
  assert.equal(
    locksConflict(lease("behavior:workflow-grid", "read", "a1"), lease("behavior:workflow-grid", "write", "a2")),
    true,
  );
});

test("detects overlapping path ownership without wildcard prefix false positives", () => {
  assert.equal(
    locksConflict(
      lease("path:repo:install/local/wikidot-verification/**", "write", "a1"),
      lease("path:repo:install/local/wikidot-verification/src/grid-worker.mjs", "write", "a2"),
    ),
    true,
  );
  assert.equal(
    locksConflict(
      lease("path:repo:foo/**", "write", "a1"),
      lease("path:repo:foobar/file.mjs", "write", "a2"),
    ),
    false,
  );
  assert.equal(
    locksConflict(
      lease("path:repo:deepwell/src/services/render/**", "write", "a1"),
      lease("path:repo:install/local/wikidot-verification/src/grid-worker.mjs", "write", "a2"),
    ),
    false,
  );
});

test("acquires all requested ownership only when the set is conflict-free", () => {
  const success = acquireLocks({
    existingLeases: [lease("repo:Rokurolize/wikijump", "read", "a1")],
    requestedLeases: [lease("behavior:workflow-grid", "write", "a2")],
  });
  assert.equal(success.acquired, true);
  assert.equal(success.leases.length, 2);

  const blocked = acquireLocks({
    existingLeases: [lease("behavior:workflow-grid", "write", "a1")],
    requestedLeases: [
      lease("behavior:workflow-grid", "write", "a2"),
      lease("artifact:artifact-validator", "write", "a2"),
    ],
  });
  assert.equal(blocked.acquired, false);
  assert.equal(blocked.leases.length, 1);
  assert.equal(blocked.conflicts.length, 1);
});

test("refuses conflicting ownership inside the requested batch", () => {
  const result = acquireLocks({
    existingLeases: [],
    requestedLeases: [
      lease("behavior:workflow-grid", "write", "a1"),
      lease("behavior:workflow-grid", "write", "a2"),
    ],
  });

  assert.equal(result.acquired, false);
  assert.equal(result.conflicts.length, 1);
});

test("classifies mutable base drift as stale", () => {
  const oldSha = "1111111111111111111111111111111111111111";
  const newSha = "2222222222222222222222222222222222222222";

  assert.deepEqual(
    classifyBaseFreshness({taskBaseSha: newSha, observedDevelopSha: newSha, mutable: true}),
    {state: "CURRENT", stale: false},
  );
  assert.deepEqual(
    classifyBaseFreshness({taskBaseSha: oldSha, observedDevelopSha: newSha, mutable: true}),
    {state: "STALE_BASE", stale: true},
  );
  assert.deepEqual(
    classifyBaseFreshness({taskBaseSha: oldSha, observedDevelopSha: newSha, mutable: false}),
    {state: "HISTORICAL_ALLOWED", stale: false},
  );
  assert.throws(
    () => classifyBaseFreshness({taskBaseSha: oldSha, observedDevelopSha: newSha}),
    /mutable must be a boolean/,
  );
});

test("records ownership events as append-only JSON lines", async (t) => {
  const root = await tempDir(t);
  const eventLogPath = path.join(root, "events.jsonl");

  await acquireLocksWithEvents({
    eventLogPath,
    existingLeases: [],
    requestedLeases: [lease("behavior:workflow-grid", "write", "a1")],
  });
  await acquireLocksWithEvents({
    eventLogPath,
    existingLeases: [lease("behavior:workflow-grid", "write", "a1")],
    requestedLeases: [lease("behavior:workflow-grid", "write", "a2")],
  });

  const events = await readLockEvents(eventLogPath);
  assert.equal(events.length, 2);
  assert.equal(events[0].event, "LOCKS_ACQUIRED");
  assert.equal(events[1].event, "LOCK_CONFLICT");
});

test("generated event time cannot be overridden by payload data", async (t) => {
  const root = await tempDir(t);
  const eventLogPath = path.join(root, "events.jsonl");

  await appendLockEvent({
    eventLogPath,
    event: {event: "TEST", time: "1999-01-01T00:00:00.000Z"},
  });

  const events = await readLockEvents(eventLogPath);
  assert.match(events[0].time, /^\d{4}-\d{2}-\d{2}T/);
  assert.notEqual(events[0].time, "1999-01-01T00:00:00.000Z");
});
