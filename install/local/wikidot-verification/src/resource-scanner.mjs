const LOCAL_FILES_URL_RE = /https?:\/\/[^\s"'()<>[\]{}]+\/local--files\/[^\s"'()<>[\]{}]+/gi;

const WIKIDOT_DOMAIN_SUFFIX = ".wikidot.com";

const KIND_GUESSES = {
  css: "css",
  scss: "css",
  sass: "css",
  js: "script",
  mjs: "script",
  ts: "script",
  jsx: "script",
  tsx: "script",
  png: "image",
  jpg: "image",
  jpeg: "image",
  gif: "image",
  webp: "image",
  svg: "image",
  avif: "image",
  bmp: "image",
  ico: "image",
  woff: "font",
  woff2: "font",
  ttf: "font",
  otf: "font",
  eot: "font",
  mp3: "audio",
  wav: "audio",
  mp4: "video",
  webm: "video",
};

const DEFAULT_OPTIONS = {
  fixtureSlug: "unknown-fixture",
  sourcePath: "unknown-source",
};

function isWikidotHost(host) {
  return host.endsWith(WIKIDOT_DOMAIN_SUFFIX);
}

function sanitizeOriginalUrl(rawUrl) {
  return rawUrl
    .replace(/^[\"'`]+/, "")
    .replace(/[\"'`;,\)\].:!?]+$/, "");
}

function canonicalizeUrl(url) {
  const normalized = sanitizeOriginalUrl(url);
  const parsed = new URL(normalized);
  const encodedPath = parsed.pathname;
  const search = parsed.search || "";
  return `${parsed.origin}${encodedPath}${search}`;
}

function kindFromFilename(filename) {
  const extension = (filename.includes(".")
    ? filename.slice(filename.lastIndexOf(".") + 1)
    : ""
  ).toLowerCase();
  return KIND_GUESSES[extension] || "unknown";
}

function localTargetPath(fixtureSlug, site, wikidotPath) {
  const safeSite = site.replaceAll(".", "_");
  return `resources/${fixtureSlug}/${safeSite}${wikidotPath}`;
}

export function scanForFixtureLocalResources({
  sourceText = "",
  fixtureSlug = DEFAULT_OPTIONS.fixtureSlug,
  sourcePath = DEFAULT_OPTIONS.sourcePath,
}) {
  const manifest = [];
  const outOfScope = [];
  const seen = new Set();

  const matches = sourceText.matchAll(LOCAL_FILES_URL_RE);
  for (const match of matches) {
    const raw = sanitizeOriginalUrl(match[0]);
    let parsed;
    try {
      parsed = new URL(raw);
    } catch (err) {
      continue;
    }

    if (!parsed.pathname.includes("/local--files/")) {
      continue;
    }

    const site = parsed.hostname;
    const canonical = canonicalizeUrl(raw);
    const wikidotPath = parsed.pathname.includes("/local--files/")
      ? parsed.pathname.slice(parsed.pathname.indexOf("/local--files/"))
      : "";
    const filename = wikidotPath.split("/").at(-1) || "";
    const normalizedSource = {
      fixture_slug: fixtureSlug,
      source_path: sourcePath,
      original_url: canonical,
      site,
      wikidot_path: wikidotPath,
      filename,
      kind_guess: kindFromFilename(filename),
      local_target_path: localTargetPath(fixtureSlug, site, wikidotPath),
      sha256: null,
    };

    if (seen.has(canonical)) {
      continue;
    }
    seen.add(canonical);

    if (!isWikidotHost(site)) {
      outOfScope.push(normalizedSource);
      continue;
    }

    manifest.push(normalizedSource);
  }

  manifest.sort((a, b) =>
    `${a.site}\u0000${a.wikidot_path}`.localeCompare(
      `${b.site}\u0000${b.wikidot_path}`,
    ),
  );

  return {
    manifest,
    out_of_scope: outOfScope,
  };
}

export function renderSampleManifestText({
  fixtureSlug,
  sourcePath,
  sourceText,
}) {
  return JSON.stringify(
    scanForFixtureLocalResources({fixtureSlug, sourcePath, sourceText}),
    null,
    2,
  );
}
