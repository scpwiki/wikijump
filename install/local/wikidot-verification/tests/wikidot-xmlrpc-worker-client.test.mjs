import assert from "node:assert/strict";
import {spawn} from "node:child_process";
import process from "node:process";
import test from "node:test";

import {WikidotXmlrpcWorkerClient} from "../src/wikidot-xmlrpc-worker-client.mjs";
import {
  parseWikidotXmlrpcWorkerRecord,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "../src/wikidot-xmlrpc-worker-protocol.mjs";
import {
  client,
  PRINCIPAL_ID,
  PYTHON_ENVIRONMENT,
  start,
} from "./support/wikidot-xmlrpc-worker-client-fixture.mjs";

test("worker session accepts only a caller-owned child-process capability", () => {
  assert.throws(
    () =>
      new WikidotXmlrpcWorkerClient({
        args: [],
        command: process.execPath,
        env: { PATH: process.env.PATH },
      }),
    /execution capability/u,
  );
});

test("worker session rejects raw launch options even with a child capability", () => {
  assert.throws(
    () => new WikidotXmlrpcWorkerClient({}, { command: process.execPath }),
    /unexpected fields/u,
  );
});

test("invalid options do not claim the caller's execution capability", () => {
  let accessed = false;
  const capability = new Proxy(
    {},
    {
      get() {
        accessed = true;
        throw new Error("capability was accessed");
      },
      ownKeys() {
        accessed = true;
        throw new Error("capability was accessed");
      },
    },
  );

  assert.throws(
    () =>
      new WikidotXmlrpcWorkerClient(capability, { command: process.execPath }),
    /unexpected fields/u,
  );
  assert.equal(accessed, false);
});

test("worker session delegates process-group termination to its capability", async (t) => {
  const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
    detached: false,
    stdio: ["pipe", "pipe", "ignore"],
  });
  const signals = [];
  const worker = new WikidotXmlrpcWorkerClient(
    {
      child,
      signalProcessGroup(signal) {
        signals.push(signal);
        child.kill(signal);
      },
    },
    { exitGraceMs: 1_000 },
  );
  t.after(() => worker.terminate("SIGKILL").catch(() => {}));

  const closed = await worker.terminate("SIGTERM");
  assert.deepEqual(signals, ["SIGTERM"]);
  assert.equal(closed.signal, "SIGTERM");
});

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

test("v2 startup sends exact attestation and credential-bearing initialize records", async (t) => {
  const worker = client(String.raw`
let attested = false;
globalThis.attest = (record) => {
  if (JSON.stringify(record) !== '{"op":"attest","protocol_version":2}') process.exit(91);
  attested = true;
  return {ok:true,op:"attestation",protocol_version:2,runtime:{implementation:"cpython",version:[3,13,13]},worker:"wikidot_xmlrpc_capture_worker"};
};
globalThis.handle = (record) => {
  if (!attested || JSON.stringify(record) !== '{"api_key":"test-api-key","app_name":"wikijump-reference-capture","op":"initialize","principal_id":${PRINCIPAL_ID},"protocol_version":2}') process.exit(92);
  process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await start(worker);
  await worker.closeClean();
});

test("v2 startup rejects malformed credentials without initialize or secret disclosure", async (t) => {
  const secret = "must-not-reach-worker";
  const accessorCredentials = {
    apiKey: "test-api-key",
    get appName() {
      throw new Error(secret);
    },
  };
  const symbolCredentials = {
    apiKey: "test-api-key",
    appName: "wikijump-reference-capture",
    [Symbol("extra")]: secret,
  };
  for (const credentials of [
    null,
    [],
    Object.create(null),
    new Proxy({}, {}),
    accessorCredentials,
    symbolCredentials,
    { apiKey: "", appName: "wikijump-reference-capture" },
    { apiKey: "test-api-key", appName: "a".repeat(4097) },
    { apiKey: `${secret}\n`, appName: "wikijump-reference-capture" },
    {
      apiKey: String.fromCharCode(0xd800),
      appName: "wikijump-reference-capture",
    },
  ]) {
    const worker = client(String.raw`
globalThis.handle = () => process.exit(97);`);
    t.after(() => worker.terminate().catch(() => {}));
    await assert.rejects(
      start(worker, PRINCIPAL_ID, PYTHON_ENVIRONMENT, credentials),
      (error) => {
        assert(error instanceof WorkerProtocolError);
        assert.doesNotMatch(String(error), new RegExp(secret, "u"));
        return true;
      },
    );
    const closed = await worker.closePromise;
    assert.notEqual(closed.code, 97);
  }
});

test("v2 startup accepts credential fields at their byte boundary", async (t) => {
  const credentials = {
    apiKey: "a".repeat(4096),
    appName: "b".repeat(4096),
  };
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op !== "initialize" || Buffer.byteLength(record.api_key) !== 4096 || Buffer.byteLength(record.app_name) !== 4096) process.exit(98);
  process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await start(worker, PRINCIPAL_ID, PYTHON_ENVIRONMENT, credentials);
  await worker.closeClean();
});

test("v2 startup rejects an invalid principal before attestation", async (t) => {
  for (const principalId of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1, "1"]) {
    const worker = client(String.raw`
globalThis.attest = () => process.exit(99);`);
    t.after(() => worker.terminate().catch(() => {}));
    await assert.rejects(start(worker, principalId), WorkerProtocolError);
    const closed = await worker.closePromise;
    assert.notEqual(closed.code, 99);
  }
});

test("v2 startup rejects bad or absent attestation without initialize", async (t) => {
  for (const { error, source, startupTimeoutMs } of [
    {
      error: WorkerProtocolError,
      source: String.raw`
globalThis.attest = () => ({ok:true,op:"attestation",protocol_version:1,runtime:{implementation:"cpython",version:[3,13,13]},worker:"wikidot_xmlrpc_capture_worker"});
globalThis.handle = () => process.exit(97);`,
      startupTimeoutMs: 1_000,
    },
    {
      error: WorkerTerminatedError,
      source: String.raw`
globalThis.attest = () => undefined;
globalThis.handle = () => process.exit(98);`,
      startupTimeoutMs: 200,
    },
  ]) {
    const worker = client(source, { startupTimeoutMs });
    t.after(() => worker.terminate().catch(() => {}));
    await assert.rejects(start(worker), error);
    const closed = await worker.closePromise;
    assert.notEqual(closed.code, 97);
    assert.notEqual(closed.code, 98);
  }
});

test("v2 startup rejects an invalid supplied environment without initialize", async (t) => {
  const worker = client(String.raw`
globalThis.handle = () => process.exit(99);`);
  t.after(() => worker.terminate().catch(() => {}));
  await assert.rejects(start(worker, PRINCIPAL_ID, {}), WorkerProtocolError);
  const closed = await worker.closePromise;
  assert.notEqual(closed.code, 99);
});

test("v2 startup spends one monotonic deadline across attestation and initialize", async (t) => {
  const worker = client(
    String.raw`
globalThis.attest = () => {
  setTimeout(
    () => process.stdout.write(JSON.stringify({ok:true,op:"attestation",protocol_version:2,runtime:{implementation:"cpython",version:[3,13,13]},worker:"wikidot_xmlrpc_capture_worker"}) + "\n"),
    250,
  );
  return undefined;
};
globalThis.handle = (record) => {
  setTimeout(
    () => process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n"),
    250,
  );
};`,
    { startupTimeoutMs: 400 },
  );
  t.after(() => worker.terminate().catch(() => {}));
  await assert.rejects(start(worker), WorkerTerminatedError);
});

test("v2 startup rejects malformed readiness after valid attestation", async (t) => {
  for (const ready of [
    { ok: true, op: "ready", principal_id: PRINCIPAL_ID + 1 },
    { extra: true, ok: true, op: "ready", principal_id: PRINCIPAL_ID },
  ]) {
    const worker = client(String.raw`
globalThis.handle = () => process.stdout.write(${JSON.stringify(`${JSON.stringify(ready)}\n`)});`);
    t.after(() => worker.terminate().catch(() => {}));
    await assert.rejects(start(worker), WorkerProtocolError);
  }
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
  await start(worker);
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
    await start(worker);
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
    await start(worker);
    await assert.rejects(
      worker.capture(0, "scp-173").then((result) => {
        assert.equal(result.retryable, true);
        return worker.expectExit(75);
      }),
    );
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
    await start(worker);
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
  await assert.rejects(start(unsolicited), WorkerProtocolError);
});

test("a bounded trailing fragment is discarded only after process close", async (t) => {
  const worker = client(String.raw`
globalThis.handle = (record) => {
  if (record.op === "initialize") process.stdout.write(JSON.stringify({ok:true,op:"ready",principal_id:record.principal_id}) + "\n");
  else process.stdout.write('{"ok":', () => process.exit(70));
};`);
  t.after(() => worker.terminate().catch(() => {}));
  await start(worker);
  await assert.rejects(worker.capture(0, "scp-173"), WorkerTerminatedError);
});
