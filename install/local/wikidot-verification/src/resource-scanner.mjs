import {
  buildFixtureResourceTargetPath,
  isWikidotResourceHost,
} from "./resource-manifest.mjs";
import {
  matchFixtureLocalResourceUrls,
  parseFixtureLocalResourceUrlToken,
} from "./resource-url.mjs";

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

function kindFromFilename(filename) {
  const extension = (filename.includes(".")
    ? filename.slice(filename.lastIndexOf(".") + 1)
    : ""
  ).toLowerCase();
  return KIND_GUESSES[extension] || "unknown";
}

export function scanForFixtureLocalResources({
  sourceText = "",
  fixtureSlug = DEFAULT_OPTIONS.fixtureSlug,
  sourcePath = DEFAULT_OPTIONS.sourcePath,
}) {
  const manifest = [];
  const outOfScope = [];
  const seen = new Set();

  const matches = matchFixtureLocalResourceUrls(sourceText);
  for (const match of matches) {
    let parsedToken;
    try {
      parsedToken = parseFixtureLocalResourceUrlToken(match[0]);
    } catch {
      continue;
    }
    const {canonicalUrl: canonical, parsed} = parsedToken;

    if (!parsed.pathname.includes("/local--files/")) {
      continue;
    }

    const site = parsed.hostname;
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
      sha256: null,
    };

    if (seen.has(canonical)) {
      continue;
    }
    seen.add(canonical);

    if (!isWikidotResourceHost(site)) {
      outOfScope.push({...normalizedSource, local_target_path: null});
      continue;
    }

    let localTargetPath;
    try {
      localTargetPath = buildFixtureResourceTargetPath({
        fixtureSlug,
        site,
        wikidotPath,
        urlSearch: parsed.search,
      });
    } catch {
      outOfScope.push({...normalizedSource, local_target_path: null});
      continue;
    }

    manifest.push({...normalizedSource, local_target_path: localTargetPath});
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
