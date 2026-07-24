import assert from "node:assert/strict";
import fs from "node:fs/promises";
import process from "node:process";
import test from "node:test";

import {
  OperatorSignalError,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "../src/wikidot-xmlrpc-worker-protocol.mjs";
import {
  client,
  start,
} from "./support/wikidot-xmlrpc-worker-client-fixture.mjs";

test("an over-limit fragment and a response timeout terminate the worker", async (t) => {
  const oversized = client(
    String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else process.stdout.write(Buffer.alloc(32 * 1024 * 1024 + 4097, 120));
};`,
    { captureTimeoutMs: 10_000 },
  );
  t.after(() => oversized.terminate().catch(() => {}));
  await start(oversized);
  await assert.rejects(oversized.capture(0, "scp-173"), WorkerProtocolError);

  const stalled = client(
    String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`,
    { captureTimeoutMs: 25 },
  );
  t.after(() => stalled.terminate().catch(() => {}));
  await start(stalled);
  await assert.rejects(stalled.capture(0, "scp-173"), WorkerTerminatedError);
});

test("requests are bounded and operator signals preserve conventional exit status", async (t) => {
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await start(worker);
  await assert.rejects(
    worker.capture(0, `scp-${"x".repeat(4096)}`),
    WorkerProtocolError,
  );

  const interrupted = client(String.raw`
globalThis.onEnd = () => {};
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`);
  t.after(() => interrupted.terminate().catch(() => {}));
  await start(interrupted);
  const closing = interrupted.closeClean();
  interrupted.handleSignal("SIGINT");
  await assert.rejects(closing, OperatorSignalError);
  assert.throws(
    () => interrupted.assertNotSignaled(),
    (error) => error instanceof OperatorSignalError && error.exitCode === 130,
  );
});

test("protocol failure stops the worker's entire process group", async (t) => {
  const worker = client(String.raw`
const {spawn} = require("node:child_process");
const grandchild = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {stdio:"ignore"});
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else if (record.ordinal === 0) process.stdout.write(JSON.stringify({ok:true,op:"capture",ordinal:0,response:{pid:grandchild.pid}}) + "\n");
  else process.stdout.write('{"ok":true,"ok":false,"op":"capture","ordinal":1,"response":{}}\n');
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await start(worker);
  const pid = (await worker.capture(0, "scp-173")).response.pid;
  await assert.rejects(worker.capture(1, "scp-174"), WorkerProtocolError);
  let gone = false;
  for (let attempt = 0; attempt < 50; attempt += 1) {
    try {
      const stat = await fs.readFile(`/proc/${pid}/stat`, "utf8");
      if (stat.slice(stat.lastIndexOf(") ") + 2).startsWith("Z")) {
        gone = true;
        break;
      }
    } catch (error) {
      if (error.code === "ENOENT" || error.code === "ESRCH") {
        gone = true;
        break;
      }
      throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.equal(gone, true, "grandchild remained alive");
});

test("termination still signals descendants after the worker leader exits", async (t) => {
  const worker = client(String.raw`
const {spawn} = require("node:child_process");
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else {
    const descendant = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {stdio:["ignore",process.stdout,"ignore"]});
    process.stdout.write(JSON.stringify({ok:true,op:"capture",ordinal:record.ordinal,response:{pid:descendant.pid}}) + "\n");
    setTimeout(() => process.exit(0), 20);
  }
};`);
  let pid;
  t.after(async () => {
    await worker.terminate().catch(() => {});
    if (pid) {
      try {
        process.kill(pid, "SIGKILL");
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    }
  });
  await start(worker);
  pid = (await worker.capture(0, "scp-173")).response.pid;
  await new Promise((resolve) => worker.child.once("exit", resolve));

  const closed = await worker.terminate();
  assert.equal(closed.code, 0);
  let gone = false;
  for (let attempt = 0; attempt < 50; attempt += 1) {
    try {
      const stat = await fs.readFile(`/proc/${pid}/stat`, "utf8");
      if (stat.slice(stat.lastIndexOf(") ") + 2).startsWith("Z")) {
        gone = true;
        break;
      }
    } catch (error) {
      if (error.code === "ENOENT" || error.code === "ESRCH") {
        gone = true;
        break;
      }
      throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.equal(gone, true, "descendant remained alive after leader exit");
});

test("termination cleans a process group after the worker close event", async (t) => {
  const worker = client(String.raw`
const {spawn} = require("node:child_process");
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else {
    const descendant = spawn(
      process.execPath,
      [
        "-e",
        "const fs = require('node:fs'); process.on('SIGTERM', () => {}); fs.writeSync(3, 'ready'); setInterval(() => {}, 1000)",
      ],
      {stdio:["ignore", "ignore", "ignore", "pipe"]},
    );
    descendant.stdio[3].once("data", () => {
      process.stdout.write(JSON.stringify({ok:true,op:"capture",ordinal:record.ordinal,response:{pid:descendant.pid}}) + "\n");
      setTimeout(() => process.exit(0), 20);
    });
  }
};`);
  let pid;
  t.after(async () => {
    await worker.terminate().catch(() => {});
    if (pid) {
      try {
        process.kill(pid, "SIGKILL");
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    }
  });
  await start(worker);
  const closed = new Promise((resolve) => worker.child.once("close", resolve));
  pid = (await worker.capture(0, "scp-173")).response.pid;
  await closed;

  await worker.terminate();
  const beforeEscalation = await fs.readFile(`/proc/${pid}/stat`, "utf8");
  assert.equal(
    beforeEscalation
      .slice(beforeEscalation.lastIndexOf(") ") + 2)
      .startsWith("Z"),
    false,
    "SIGTERM unexpectedly removed the signal-ignoring descendant",
  );
  await worker.terminate("SIGKILL");
  let gone = false;
  for (let attempt = 0; attempt < 50; attempt += 1) {
    try {
      const stat = await fs.readFile(`/proc/${pid}/stat`, "utf8");
      if (stat.slice(stat.lastIndexOf(") ") + 2).startsWith("Z")) {
        gone = true;
        break;
      }
    } catch (error) {
      if (error.code === "ENOENT" || error.code === "ESRCH") {
        gone = true;
        break;
      }
      throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.equal(gone, true, "descendant remained alive after worker close");
});

test("an escaped descendant cannot hold the coordinator stdout pipe open", async (t) => {
  const worker = client(
    String.raw`
const {spawn} = require("node:child_process");
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else if (record.ordinal === 0) {
    const escaped = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {detached:true,stdio:["ignore",process.stdout,"ignore"]});
    escaped.unref();
    process.stdout.write(JSON.stringify({ok:true,op:"capture",ordinal:0,response:{pid:escaped.pid}}) + "\n");
  }
  else process.stdout.write('{"ok":true,"ok":false}\n');
};`,
    { exitGraceMs: 25 },
  );
  let pid;
  t.after(async () => {
    await worker.terminate().catch(() => {});
    if (pid) {
      try {
        process.kill(pid, "SIGKILL");
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    }
  });
  await start(worker);
  pid = (await worker.capture(0, "scp-173")).response.pid;
  await assert.rejects(worker.capture(1, "scp-174"), WorkerTerminatedError);
});
