import { types as utilTypes } from "node:util";

import { stableStringify } from "./corpus-import-manifest.mjs";

export function exactDataRecord(value, expectedKeys, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value) || utilTypes.isProxy(value)) {
    throw new Error(`${label} must be a data object`);
  }
  let prototype;
  let keys;
  let descriptors;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
    descriptors = keys.map((key) => Reflect.getOwnPropertyDescriptor(value, key));
  } catch {
    throw new Error(`${label} must be a data object`);
  }
  if (!([Object.prototype, null].includes(prototype)) || keys.some((key) => typeof key !== "string") || stableStringify([...keys].sort()) !== stableStringify(expectedKeys)) {
    throw new Error(`${label} has unexpected fields or prototype`);
  }
  const snapshot = {};
  for (const [index, key] of keys.entries()) {
    const descriptor = descriptors[index];
    if (descriptor === undefined || !descriptor.enumerable || !("value" in descriptor)) {
      throw new Error(`${label} must contain only enumerable data fields`);
    }
    Object.defineProperty(snapshot, key, { enumerable: true, value: descriptor.value });
  }
  return Object.freeze(snapshot);
}
