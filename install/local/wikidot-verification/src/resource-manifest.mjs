import {createHash} from "node:crypto";
import path from "node:path";

const WIKIDOT_DOMAIN_SUFFIX = ".wikidot.com";
const FIXTURE_SLUG_RE = /^[A-Za-z0-9][A-Za-z0-9._:-]*$/;
const SHA256_RE = /^[0-9a-f]{64}$/;

function assertString(value, name) {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${name} must be a non-empty string`);
  }
}

function assertSafeFixtureSlug(fixtureSlug) {
  assertString(fixtureSlug, "fixture_slug");
  if (!FIXTURE_SLUG_RE.test(fixtureSlug)) {
    throw new Error(
      "fixture_slug may contain only ASCII letters, digits, '.', '_', ':', and '-'",
    );
  }
}

function assertHostname(site) {
  assertString(site, "site");
  if (site !== site.toLowerCase() || site.includes("\\") || site.includes("\0")) {
    throw new Error("site must be a lowercase hostname without a port");
  }

  let parsed;
  try {
    parsed = new URL(`https://${site}/`);
  } catch (error) {
    throw new Error(`site is not a valid hostname: ${site}`, {cause: error});
  }

  if (parsed.hostname !== site || parsed.host !== site) {
    throw new Error(`site is not a canonical hostname: ${site}`);
  }
}

function assertSafeWikidotPath(wikidotPath) {
  assertString(wikidotPath, "wikidot_path");
  if (!wikidotPath.startsWith("/local--files/")) {
    throw new Error("wikidot_path must start with /local--files/");
  }
  if (wikidotPath.includes("\\") || wikidotPath.includes("\0")) {
    throw new Error("wikidot_path contains an unsafe character");
  }

  for (const segment of wikidotPath.split("/")) {
    if (segment.length === 0) {
      continue;
    }

    let decoded;
    try {
      decoded = decodeURIComponent(segment);
    } catch (error) {
      throw new Error(`wikidot_path contains invalid percent encoding: ${segment}`, {
        cause: error,
      });
    }

    if (
      decoded === "." ||
      decoded === ".." ||
      decoded.includes("/") ||
      decoded.includes("\\") ||
      decoded.includes("\0")
    ) {
      throw new Error(`wikidot_path contains an unsafe segment: ${segment}`);
    }
  }
}

function assertRelativeResourcePath(localTargetPath) {
  assertString(localTargetPath, "local_target_path");
  if (
    path.posix.isAbsolute(localTargetPath) ||
    localTargetPath.includes("\\") ||
    localTargetPath.includes("\0")
  ) {
    throw new Error("local_target_path must be a safe relative POSIX path");
  }

  const segments = localTargetPath.split("/");
  if (segments[0] !== "resources" || segments.some((part) => part === "." || part === "..")) {
    throw new Error("local_target_path must remain below resources/");
  }
}

function isWithinPath(rootPath, candidatePath) {
  const relative = path.relative(rootPath, candidatePath);
  return (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) &&
      relative !== ".." &&
      !path.isAbsolute(relative))
  );
}

function queryTargetSuffix(urlSearch) {
  if (urlSearch === "") {
    return "";
  }
  if (typeof urlSearch !== "string" || !urlSearch.startsWith("?")) {
    throw new Error("urlSearch must be empty or start with '?'");
  }
  if (urlSearch.includes("\0")) {
    throw new Error("urlSearch contains an unsafe character");
  }

  const digest = createHash("sha256").update(urlSearch).digest("hex").slice(0, 16);
  return `.__query-${digest}`;
}

export function isWikidotResourceHost(hostname) {
  return hostname.endsWith(WIKIDOT_DOMAIN_SUFFIX);
}

export function buildFixtureResourceTargetPath({
  fixtureSlug,
  site,
  wikidotPath,
  urlSearch = "",
}) {
  assertSafeFixtureSlug(fixtureSlug);
  assertHostname(site);
  assertSafeWikidotPath(wikidotPath);

  const safeSite = site.replaceAll(".", "_");
  return `resources/${fixtureSlug}/${safeSite}${wikidotPath}${queryTargetSuffix(urlSearch)}`;
}

export function assertFixtureResourceManifestEntry(
  entry,
  {requireSha256 = false} = {},
) {
  if (entry === null || typeof entry !== "object" || Array.isArray(entry)) {
    throw new TypeError("manifest entry must be an object");
  }

  assertSafeFixtureSlug(entry.fixture_slug);
  assertString(entry.source_path, "source_path");
  assertString(entry.original_url, "original_url");
  assertHostname(entry.site);
  assertSafeWikidotPath(entry.wikidot_path);
  assertString(entry.filename, "filename");
  assertString(entry.kind_guess, "kind_guess");
  assertRelativeResourcePath(entry.local_target_path);

  let parsed;
  try {
    parsed = new URL(entry.original_url);
  } catch (error) {
    throw new Error(`original_url is invalid: ${entry.original_url}`, {cause: error});
  }

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("original_url must use http or https");
  }
  if (parsed.port) {
    throw new Error("original_url must use the protocol's default port");
  }
  if (parsed.username || parsed.password || parsed.hash) {
    throw new Error("original_url must not contain credentials or a fragment");
  }
  if (!isWikidotResourceHost(parsed.hostname)) {
    throw new Error(`original_url host is out of scope: ${parsed.hostname}`);
  }
  if (parsed.hostname !== entry.site) {
    throw new Error("manifest site does not match original_url hostname");
  }
  if (parsed.pathname !== entry.wikidot_path) {
    throw new Error("manifest wikidot_path does not match original_url pathname");
  }

  const canonicalUrl = `${parsed.origin}${parsed.pathname}${parsed.search}`;
  if (canonicalUrl !== entry.original_url) {
    throw new Error("original_url is not canonical");
  }

  const expectedFilename = entry.wikidot_path.split("/").at(-1) || "";
  if (entry.filename !== expectedFilename) {
    throw new Error("manifest filename does not match wikidot_path");
  }

  const expectedTargetPath = buildFixtureResourceTargetPath({
    fixtureSlug: entry.fixture_slug,
    site: entry.site,
    wikidotPath: entry.wikidot_path,
    urlSearch: parsed.search,
  });
  if (entry.local_target_path !== expectedTargetPath) {
    throw new Error("local_target_path does not match the deterministic manifest path");
  }

  if (
    entry.sha256 !== null &&
    (typeof entry.sha256 !== "string" || !SHA256_RE.test(entry.sha256))
  ) {
    throw new Error("sha256 must be null or a lowercase 64-character hex digest");
  }
  if (requireSha256 && entry.sha256 === null) {
    throw new Error(`resource has not been materialized: ${entry.original_url}`);
  }

  return entry;
}

export function resolveFixtureResourcePath(rootPath, localTargetPath) {
  assertString(rootPath, "rootPath");
  assertRelativeResourcePath(localTargetPath);

  const resolvedRoot = path.resolve(rootPath);
  const resolvedTarget = path.resolve(resolvedRoot, ...localTargetPath.split("/"));
  if (!isWithinPath(resolvedRoot, resolvedTarget)) {
    throw new Error("local_target_path escapes the configured root");
  }

  return resolvedTarget;
}

export function assertResolvedPathWithin(rootPath, candidatePath, label = "path") {
  const resolvedRoot = path.resolve(rootPath);
  const resolvedCandidate = path.resolve(candidatePath);
  if (!isWithinPath(resolvedRoot, resolvedCandidate)) {
    throw new Error(`${label} escapes the configured root`);
  }
}
