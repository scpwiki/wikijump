import { isWikidotResourceHost } from "./resource-manifest.mjs";

const SHA256_RE = /^[0-9a-f]{64}$/u;

function assertString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
}

function decodePathSegment(segment, label) {
  let decoded;
  try {
    decoded = decodeURIComponent(segment);
  } catch (error) {
    throw new Error(`${label} has invalid encoding: ${error.message}`);
  }
  if (
    decoded === "." ||
    decoded === ".." ||
    decoded.includes("/") ||
    decoded.includes("\\") ||
    decoded.includes("\0")
  ) {
    throw new Error(`${label} has an unsafe segment`);
  }
  return decoded;
}

export function normalizeAcquisitionAttachment(attachment, rowLabel, seenUrls) {
  if (
    attachment === null ||
    typeof attachment !== "object" ||
    Array.isArray(attachment)
  ) {
    throw new Error(`${rowLabel} attachment must be an object`);
  }
  for (const field of [
    "filename",
    "original_url",
    "wikidot_path",
    "sha256",
    "mime",
  ]) {
    assertString(attachment[field], `${rowLabel} attachment.${field}`);
  }
  if (!SHA256_RE.test(attachment.sha256)) {
    throw new Error(
      `${rowLabel} attachment.sha256 must be a lowercase SHA-256 digest`,
    );
  }
  if (!Number.isSafeInteger(attachment.size) || attachment.size < 0) {
    throw new Error(
      `${rowLabel} attachment.size must be a non-negative safe integer`,
    );
  }
  if (
    attachment.filename.includes("/") ||
    attachment.filename.includes("\\") ||
    attachment.filename.includes("\0")
  ) {
    throw new Error(
      `${rowLabel} attachment.filename contains an unsafe character`,
    );
  }
  let originalUrl;
  try {
    originalUrl = new URL(attachment.original_url);
  } catch (error) {
    throw new Error(
      `${rowLabel} attachment.original_url is invalid: ${error.message}`,
    );
  }
  if (!["http:", "https:"].includes(originalUrl.protocol)) {
    throw new Error(
      `${rowLabel} attachment.original_url must use HTTP or HTTPS`,
    );
  }
  if (
    originalUrl.port !== "" ||
    originalUrl.username !== "" ||
    originalUrl.password !== "" ||
    originalUrl.hash !== ""
  ) {
    throw new Error(
      `${rowLabel} attachment.original_url must not contain credentials, a fragment, or a non-default port`,
    );
  }
  if (!isWikidotResourceHost(originalUrl.hostname)) {
    throw new Error(`${rowLabel} attachment.original_url host is out of scope`);
  }
  if (
    !attachment.wikidot_path.startsWith("/local--files/") ||
    attachment.wikidot_path.includes("\\") ||
    attachment.wikidot_path.includes("\0")
  ) {
    throw new Error(
      `${rowLabel} attachment.wikidot_path is not a safe /local--files/ path`,
    );
  }
  for (const segment of attachment.wikidot_path.split("/")) {
    decodePathSegment(segment, `${rowLabel} attachment.wikidot_path`);
  }
  if (originalUrl.pathname !== attachment.wikidot_path) {
    throw new Error(
      `${rowLabel} attachment URL pathname does not match wikidot_path`,
    );
  }
  const canonicalUrl = `${originalUrl.origin}${originalUrl.pathname}${originalUrl.search}`;
  if (canonicalUrl !== attachment.original_url) {
    throw new Error(`${rowLabel} attachment.original_url is not canonical`);
  }
  const encodedFilename = attachment.wikidot_path.split("/").at(-1);
  if (
    decodePathSegment(encodedFilename, `${rowLabel} attachment filename`) !==
    attachment.filename
  ) {
    throw new Error(
      `${rowLabel} attachment filename does not match wikidot_path`,
    );
  }
  if (seenUrls.has(canonicalUrl)) {
    throw new Error(`${rowLabel} has duplicate attachment URL ${canonicalUrl}`);
  }
  seenUrls.add(canonicalUrl);
  return {
    filename: attachment.filename,
    mime: attachment.mime,
    original_url: attachment.original_url,
    sha256: attachment.sha256,
    size: attachment.size,
    wikidot_path: attachment.wikidot_path,
  };
}
