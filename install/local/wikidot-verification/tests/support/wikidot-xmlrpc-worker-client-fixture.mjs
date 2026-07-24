import {spawn} from "node:child_process";
import process from "node:process";

import {WikidotXmlrpcWorkerClient} from "../../src/wikidot-xmlrpc-worker-client.mjs";
import {buildWikidotXmlrpcPythonEnvironment} from "../../src/wikidot-xmlrpc-python-environment.mjs";

export const PRINCIPAL_ID = 5700026;
export const CREDENTIALS = Object.freeze({
  apiKey: "test-api-key",
  appName: "wikijump-reference-capture",
});
export const PYTHON_ENVIRONMENT = buildWikidotXmlrpcPythonEnvironment({
  dependencyEnvironmentSha256: "a".repeat(64),
  dependencyLockBlobOid: "1".repeat(40),
  dependencyLockFileSha256: "b".repeat(64),
  dependencyRecipeBlobOid: "2".repeat(40),
  dependencyRecipeSha256: "c".repeat(64),
  pythonExecutableSha256: "d".repeat(64),
  pythonImplementation: "cpython",
  pythonVersion: "3.13.13",
  venvConfigSha256: "e".repeat(64),
  workerBlobOid: "3".repeat(40),
  workerFileSha256: "f".repeat(64),
  workerRepositoryCommit: "4".repeat(40),
  workerRepositoryTree: "5".repeat(40),
});
export const BASE_CHILD = String.raw`
let buffer = "";
const defaultAttestation = {
  ok: true,
  op: "attestation",
  protocol_version: 2,
  runtime: { implementation: "cpython", version: [3, 13, 13] },
  worker: "wikidot_xmlrpc_capture_worker",
};
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;
  for (;;) {
    const newline = buffer.indexOf("\n");
    if (newline < 0) break;
    const record = JSON.parse(buffer.slice(0, newline));
    buffer = buffer.slice(newline + 1);
    if (record.op === "attest") {
      const response = globalThis.attest
        ? globalThis.attest(record)
        : defaultAttestation;
      if (response !== undefined)
        process.stdout.write(JSON.stringify(response) + "\n");
    } else globalThis.handle(record);
  }
});
process.stdin.on("end", () => globalThis.onEnd ? globalThis.onEnd() : process.exit(0));
`;

export function client(source, overrides = {}) {
  const child = spawn(process.execPath, ["-e", `${source}\n${BASE_CHILD}`], {
    detached: true,
    env: { PATH: process.env.PATH },
    stdio: ["pipe", "pipe", "ignore"],
  });
  return new WikidotXmlrpcWorkerClient(processExecution(child), {
    startupTimeoutMs: 2_000,
    captureTimeoutMs: 2_000,
    exitGraceMs: 1_000,
    ...overrides,
  });
}

export function processExecution(child) {
  return {
    child,
    signalProcessGroup(signal) {
      if (!child.pid) return;
      try {
        process.kill(-child.pid, signal);
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    },
  };
}

export function start(
  worker,
  principalId = PRINCIPAL_ID,
  environment = PYTHON_ENVIRONMENT,
  credentials = CREDENTIALS,
) {
  return worker.start(principalId, environment, credentials);
}
