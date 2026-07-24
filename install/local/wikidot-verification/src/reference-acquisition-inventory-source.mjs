import {sha256Hex} from "./canonical-json.mjs";

const FATAL_UTF8_DECODER = new TextDecoder("utf-8", {fatal: true});
const SHA256_RE = /^[0-9a-f]{64}$/u;
const RFC3339_RE =
  /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|[+-](\d{2}):(\d{2}))$/u;

export const REFERENCE_ACQUISITION_UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
export const REFERENCE_ACQUISITION_FAMILY_RE = /^[A-Z][A-Z0-9-]*$/u;
export const REFERENCE_ACQUISITION_REQUESTED_LAYERS = Object.freeze([
  "xmlrpc_page",
  "http_document",
  "browser_document",
]);

export function assertReferenceAcquisitionObject(value, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
}

export function assertReferenceAcquisitionNonEmptyString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
}

export function assertReferenceAcquisitionSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

export function assertReferenceAcquisitionNonNegativeSafeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
}

export function assertTimestamp(value, label) {
  assertReferenceAcquisitionNonEmptyString(value, label);
  const match = RFC3339_RE.exec(value);
  if (match === null) {
    throw new Error(`${label} must be an RFC 3339 date-time`);
  }
  const [
    ,
    yearText,
    monthText,
    dayText,
    hourText,
    minuteText,
    secondText,
    offsetHourText = "0",
    offsetMinuteText = "0",
  ] = match;
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [
    0,
    31,
    leapYear ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];
  if (
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > daysInMonth[month] ||
    Number(hourText) > 23 ||
    Number(minuteText) > 59 ||
    Number(secondText) > 59 ||
    Number(offsetHourText) > 23 ||
    Number(offsetMinuteText) > 59 ||
    Number.isNaN(Date.parse(value))
  ) {
    throw new Error(`${label} must be an RFC 3339 date-time`);
  }
}

export function assertCanonicalFullname(fullname, label) {
  assertReferenceAcquisitionNonEmptyString(fullname, label);
  if (
    fullname.includes("/") ||
    fullname.includes("\\") ||
    /[\u0000-\u001f\u007f]/u.test(fullname)
  ) {
    throw new Error(`${label} contains an unsafe path character`);
  }
  const url = new URL("https://example.invalid/");
  url.pathname = `/${fullname}`;
  let roundTrip;
  try {
    roundTrip = decodeURIComponent(url.pathname.slice(1));
  } catch (error) {
    throw new Error(
      `${label} contains invalid percent encoding: ${error.message}`,
    );
  }
  if (roundTrip !== fullname) {
    throw new Error(
      `${label} does not round-trip through a canonical URL path`,
    );
  }
}

function toBuffer(value, label) {
  if (typeof value === "string") {
    return Buffer.from(value, "utf8");
  }
  if (value instanceof Uint8Array) {
    return Buffer.from(value.buffer, value.byteOffset, value.byteLength);
  }
  throw new Error(`${label} must be a string or Uint8Array`);
}

function decodeUtf8(bytes, label) {
  try {
    return FATAL_UTF8_DECODER.decode(bytes);
  } catch (error) {
    throw new Error(`${label} must contain valid UTF-8: ${error.message}`);
  }
}

export function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function validateOrigin(sourceOrigin) {
  let parsed;
  try {
    parsed = new URL(sourceOrigin);
  } catch (error) {
    throw new Error(`sourceOrigin must be an absolute URL: ${error.message}`);
  }
  if (
    parsed.protocol !== "https:" ||
    parsed.port !== "" ||
    parsed.username !== "" ||
    parsed.password !== "" ||
    parsed.pathname !== "/" ||
    parsed.search !== "" ||
    parsed.hash !== ""
  ) {
    throw new Error("sourceOrigin must be a credential-free HTTPS origin");
  }
  const suffix = ".wikidot.com";
  if (
    !parsed.hostname.endsWith(suffix) ||
    parsed.hostname.length === suffix.length
  ) {
    throw new Error("sourceOrigin must identify a Wikidot site hostname");
  }
  return {
    origin: parsed.origin,
    sourceSite: parsed.hostname.slice(0, -suffix.length),
  };
}

export function parseReferenceAcquisitionManifest(
  manifestBytes,
  expectedManifestSha256,
  expectedCount,
) {
  const bytes = toBuffer(manifestBytes, "manifestBytes");
  assertReferenceAcquisitionSha256(
    expectedManifestSha256,
    "expectedManifestSha256",
  );
  const actualSha256 = sha256Hex(bytes);
  if (actualSha256 !== expectedManifestSha256) {
    throw new Error(
      `manifest SHA-256 mismatch: expected ${expectedManifestSha256}, got ${actualSha256}`,
    );
  }
  const text = decodeUtf8(bytes, "manifestBytes");
  if (!text.endsWith("\n") || text.includes("\r")) {
    throw new Error(
      "manifestBytes must use LF lines and end with exactly one LF",
    );
  }
  const lines = text.slice(0, -1).split("\n");
  if (lines.some((line) => line.length === 0)) {
    throw new Error(
      "manifestBytes must not contain blank lines or extra terminal LFs",
    );
  }
  if (lines.length !== expectedCount) {
    throw new Error(
      `manifest row count mismatch: expected ${expectedCount}, got ${lines.length}`,
    );
  }
  return {
    bytes,
    sha256: actualSha256,
    rows: lines.map((line, index) => {
      try {
        return {
          input: JSON.parse(line),
          inputLineSha256: sha256Hex(line),
          lineNumber: index + 1,
        };
      } catch (error) {
        throw new Error(
          `manifest line ${index + 1} is not valid JSON: ${error.message}`,
        );
      }
    }),
  };
}

export function parseReferenceAcquisitionSummary(
  summaryBytes,
  expectedSummarySha256,
) {
  const bytes = toBuffer(summaryBytes, "summaryBytes");
  assertReferenceAcquisitionSha256(
    expectedSummarySha256,
    "expectedSummarySha256",
  );
  const actualSha256 = sha256Hex(bytes);
  if (actualSha256 !== expectedSummarySha256) {
    throw new Error(
      `summary SHA-256 mismatch: expected ${expectedSummarySha256}, got ${actualSha256}`,
    );
  }
  let summary;
  try {
    summary = JSON.parse(decodeUtf8(bytes, "summaryBytes"));
  } catch (error) {
    throw new Error(
      `summaryBytes must contain one JSON document: ${error.message}`,
    );
  }
  assertReferenceAcquisitionObject(summary, "summary");
  return {summary, sha256: actualSha256};
}
