import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";

const CLIENT_OPTION_KEYS = Object.freeze([
  "captureTimeoutMs",
  "exitGraceMs",
  "startupTimeoutMs",
]);
const EXECUTION_CAPABILITY_KEYS = Object.freeze([
  "child",
  "signalProcessGroup",
]);

export const WIKIDOT_XMLRPC_WORKER_SESSION_DEFAULTS = Object.freeze({
  captureTimeoutMs: 180_000,
  exitGraceMs: 5_000,
  startupTimeoutMs: 180_000,
});

export function normalizeWikidotXmlrpcWorkerSessionOptions(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error("worker session options must be a data object");
  }
  let keys;
  let prototype;
  try {
    keys = Reflect.ownKeys(value);
    prototype = Reflect.getPrototypeOf(value);
  } catch {
    throw new Error("worker session options must be a data object");
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some(
      (key) => typeof key !== "string" || !CLIENT_OPTION_KEYS.includes(key),
    )
  ) {
    throw new Error("worker session options have unexpected fields");
  }
  const result = { ...WIKIDOT_XMLRPC_WORKER_SESSION_DEFAULTS };
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor) ||
      !Number.isSafeInteger(descriptor.value) ||
      descriptor.value <= 0
    ) {
      throw new Error("worker session timeout must be a positive safe integer");
    }
    result[key] = descriptor.value;
  }
  return Object.freeze(result);
}

function assertWorkerChild(child) {
  if (
    child === null ||
    typeof child !== "object" ||
    utilTypes.isProxy(child) ||
    typeof child.once !== "function" ||
    typeof child.unref !== "function" ||
    child.stdin === null ||
    typeof child.stdin !== "object" ||
    typeof child.stdin.destroy !== "function" ||
    typeof child.stdin.end !== "function" ||
    typeof child.stdin.on !== "function" ||
    typeof child.stdin.write !== "function" ||
    child.stdout === null ||
    typeof child.stdout !== "object" ||
    typeof child.stdout.destroy !== "function" ||
    typeof child.stdout.on !== "function"
  ) {
    throw new Error(
      "worker session requires a spawned child-process capability",
    );
  }
}

/**
 * Snapshots the only authority the session receives over a worker. The
 * signalProcessGroup callback must address the worker's entire process group,
 * including descendants still alive after its leader has exited.
 */
export function openWikidotXmlrpcWorkerExecutionCapability(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error("worker session requires an execution capability");
  }
  let keys;
  let prototype;
  try {
    keys = Reflect.ownKeys(value);
    prototype = Reflect.getPrototypeOf(value);
  } catch {
    throw new Error("worker session requires an execution capability");
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !==
      stableStringify(EXECUTION_CAPABILITY_KEYS)
  ) {
    throw new Error("worker execution capability has unexpected fields");
  }
  const capability = {};
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error("worker execution capability must contain data fields");
    }
    capability[key] = descriptor.value;
  }
  assertWorkerChild(capability.child);
  if (typeof capability.signalProcessGroup !== "function") {
    throw new Error(
      "worker execution capability must signal its process group",
    );
  }
  const owner = Object.freeze({ child: capability.child });
  return Object.freeze({
    child: capability.child,
    signalProcessGroup(signal) {
      return Reflect.apply(capability.signalProcessGroup, owner, [signal]);
    },
  });
}
