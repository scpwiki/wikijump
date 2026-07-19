import assert from "node:assert/strict";
import fs from "node:fs/promises";
import process from "node:process";
import test from "node:test";

import {
  OperatorSignalError,
  parseWikidotXmlrpcWorkerRecord,
  WikidotXmlrpcWorkerClient,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "../src/wikidot-xmlrpc-worker-client.mjs";

const PRINCIPAL_ID = 5700026;
const BASE_CHILD = String.raw`
let buffer = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;
  for (;;) {
    const newline = buffer.indexOf("\n");
    if (newline < 0) break;
    const record = JSON.parse(buffer.slice(0, newline));
    buffer = buffer.slice(newline + 1);
    globalThis.handle(record);
  }
});
process.stdin.on("end", () => globalThis.onEnd ? globalThis.onEnd() : process.exit(0));
`;

function client(source, overrides = {}) {
  return new WikidotXmlrpcWorkerClient({
    command: process.execPath,
    args: ["-e", `${source}\n${BASE_CHILD}`],
    env: { PATH: process.env.PATH },
    startupTimeoutMs: 2_000,
    captureTimeoutMs: 2_000,
    exitGraceMs: 1_000,
    ...overrides,
  });
}

test("record parser preserves Python numeric spelling while rejecting framing and duplicate keys", () => {
  for (const literal of ["1.0", "1e20", "9007199254740992.0"])
    assert.equal(
      parseWikidotXmlrpcWorkerRecord(
        Buffer.from(`{"response":{"value":${literal}}}\n`),
      ).response.value,
      Number(literal),
    );
  for (const invalid of [
    Buffer.from("{}"),
    Buffer.from("{}\r\n"),
    Buffer.from([0xff, 0x0a]),
    Buffer.from('{"ok":true,"ok":false}\n'),
    Buffer.from('{"response":{"title":"first","title":"second"}}\n'),
    Buffer.from('{"response":{"value":1e400}}\n'),
    Buffer.from('{"response":{"value":9007199254740992}}\n'),
    Buffer.from('{"response":{"value":-9007199254740992}}\n'),
    Buffer.from(`${"[".repeat(65)}0${"]".repeat(65)}\n`),
    Buffer.concat([Buffer.alloc(32 * 1024 * 1024 + 4096), Buffer.from("\n")]),
  ]) {
    assert.throws(
      () => parseWikidotXmlrpcWorkerRecord(invalid),
      WorkerProtocolError,
    );
  }
});
test("startup admission rejects malformed readiness and a missing deadline", async (t) => {
  for (const ready of [
    { ok: true, op: "ready", principal_id: PRINCIPAL_ID + 1 },
    { extra: true, ok: true, op: "ready", principal_id: PRINCIPAL_ID },
  ]) {
    const worker = client(String.raw`
globalThis.handle = () => process.stdout.write(${JSON.stringify(`${JSON.stringify(ready)}\n`)});`);
    t.after(() => worker.terminate().catch(() => {}));
    await assert.rejects(worker.start(PRINCIPAL_ID), WorkerProtocolError);
  }
  const stalled = client("globalThis.handle = () => {};", {
    startupTimeoutMs: 25,
  });
  t.after(() => stalled.terminate().catch(() => {}));
  await assert.rejects(stalled.start(PRINCIPAL_ID), WorkerTerminatedError);
});

test("one persistent worker handles chunked success, terminal failure, and clean EOF", async (t) => {
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") {
    const ready = Buffer.from(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
    process.stdout.write(ready.subarray(0, 5));
    setImmediate(() => process.stdout.write(ready.subarray(5)));
  } else if (record.ordinal === 0) {
    const result = Buffer.from(JSON.stringify({ok:true,op:"capture",ordinal:0,response:{blob:"x".repeat(262144)}}) + "\n");
    for (let offset = 0; offset < result.length; offset += 4093) process.stdout.write(result.subarray(offset, offset + 4093));
  } else process.stdout.write(JSON.stringify({code:"response_rejected",ok:false,op:"capture",ordinal:1,retryable:false}) + "\n");
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await worker.start(PRINCIPAL_ID);
  await worker.capture(0, "scp-173");
  assert.deepEqual(await worker.capture(1, "scp-174"), {
    code: "response_rejected",
    ok: false,
    op: "capture",
    ordinal: 1,
    retryable: false,
  });
  await worker.closeClean();
});

for (const [code, retryable, exitCode] of [
  ["transport_exhausted", true, 75],
  ["worker_internal_error", false, 70],
]) {
  test(`declared ${code} result is bound to worker exit ${exitCode}`, async (t) => {
    const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else {
    process.stdout.write(JSON.stringify({code:${JSON.stringify(code)},ok:false,op:"capture",ordinal:record.ordinal,retryable:${retryable}}) + "\n", () => process.exit(${exitCode}));
  }
};`);
    t.after(() => worker.terminate().catch(() => {}));
    await worker.start(PRINCIPAL_ID);
    assert.equal((await worker.capture(0, "scp-173")).code, code);
    await worker.expectExit(exitCode);
  });
}

test("declared terminating failures reject a mismatched exit or extra fragment", async (t) => {
  for (const action of [
    "process.exit(70)",
    "process.stdout.write('partial', () => process.exit(75))",
  ]) {
    const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else process.stdout.write(JSON.stringify({code:"transport_exhausted",ok:false,op:"capture",ordinal:record.ordinal,retryable:true}) + "\n", () => { ${action}; });
};`);
    t.after(() => worker.terminate().catch(() => {}));
    await worker.start(PRINCIPAL_ID);
    assert.equal((await worker.capture(0, "scp-173")).retryable, true);
    await assert.rejects(worker.expectExit(75));
  }
});

test("wrong ordinal, retryability, fields, and unsolicited records fail closed", async (t) => {
  const cases = [
    '{"ok":true,"op":"capture","ordinal":9,"response":{}}\n',
    '{"code":"transport_exhausted","ok":false,"op":"capture","ordinal":0,"retryable":false}\n',
    '{"extra":1,"ok":true,"op":"capture","ordinal":0,"response":{}}\n',
    '{"ok":true,"op":"capture","ordinal":0,"response":[]}\n',
    '{"ok":true,"ok":false,"op":"capture","ordinal":0,"response":{}}\n',
  ];
  for (const result of cases) {
    const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else process.stdout.write(${JSON.stringify(result)});
};`);
    t.after(() => worker.terminate().catch(() => {}));
    await worker.start(PRINCIPAL_ID);
    await assert.rejects(worker.capture(0, "scp-173"), WorkerProtocolError);
  }

  const unsolicited = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") {
    const ready = JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n";
    process.stdout.write(ready + '{"ok":');
  }
};`);
  t.after(() => unsolicited.terminate().catch(() => {}));
  await assert.rejects(unsolicited.start(PRINCIPAL_ID), WorkerProtocolError);
});

test("a bounded trailing fragment is discarded only after process close", async (t) => {
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else process.stdout.write('{"ok":', () => process.exit(70));
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await worker.start(PRINCIPAL_ID);
  await assert.rejects(worker.capture(0, "scp-173"), WorkerTerminatedError);
});

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
  await oversized.start(PRINCIPAL_ID);
  await assert.rejects(oversized.capture(0, "scp-173"), WorkerProtocolError);

  const stalled = client(
    String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`,
    { captureTimeoutMs: 25 },
  );
  t.after(() => stalled.terminate().catch(() => {}));
  await stalled.start(PRINCIPAL_ID);
  await assert.rejects(stalled.capture(0, "scp-173"), WorkerTerminatedError);
});

test("requests are bounded and operator signals preserve conventional exit status", async (t) => {
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await worker.start(PRINCIPAL_ID);
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
  await interrupted.start(PRINCIPAL_ID);
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
  await worker.start(PRINCIPAL_ID);
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
      if (error.code === "ENOENT") {
        gone = true;
        break;
      }
      throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.equal(gone, true, "grandchild remained alive");
});

test("an escaped descendant cannot hold the coordinator stdout pipe open", async (t) => {
  const worker = client(
    String.raw`
const {spawn} = require("node:child_process");
const escaped = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {detached:true,stdio:["ignore",process.stdout,"ignore"]});
escaped.unref();
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else if (record.ordinal === 0) process.stdout.write(JSON.stringify({ok:true,op:"capture",ordinal:0,response:{pid:escaped.pid}}) + "\n");
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
  await worker.start(PRINCIPAL_ID);
  pid = (await worker.capture(0, "scp-173")).response.pid;
  await assert.rejects(worker.capture(1, "scp-174"), WorkerTerminatedError);
});
