import crypto from "node:crypto";

export function sha256Hex(bufferOrString) {
  return crypto.createHash("sha256").update(bufferOrString).digest("hex");
}

export function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function stableStringify(value) {
  if (value === null || typeof value !== "object") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }
  const entries = Object.entries(value).sort(([left], [right]) => codePointCompare(left, right));
  return `{${entries.map(([key, entryValue]) => `${JSON.stringify(key)}:${stableStringify(entryValue)}`).join(",")}}`;
}
