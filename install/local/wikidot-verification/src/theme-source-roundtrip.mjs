import crypto from "node:crypto";

export function targetRoundTripSource(target, source) {
  if (typeof source !== "string") throw new Error("theme source must be a string");
  if (target === "wikidot" && source.endsWith("\n")) return source.slice(0, -1);
  return source;
}

export function targetRoundTripSourceSha256(target, source) {
  return crypto.createHash("sha256").update(targetRoundTripSource(target, source)).digest("hex");
}
