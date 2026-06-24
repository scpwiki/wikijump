import {assertFixtureResourceManifestEntry} from "./resource-manifest.mjs";
import {
  createFixtureLocalResourceUrlRegExp,
  parseFixtureLocalResourceUrlToken,
} from "./resource-url.mjs";

function localResourceUrl(prefix, localTargetPath) {
  if (typeof prefix !== "string") {
    throw new TypeError("localUrlPrefix must be a string");
  }
  if (prefix.includes("\0") || prefix.includes("?") || prefix.includes("#")) {
    throw new Error("localUrlPrefix must not contain NUL, a query, or a fragment");
  }

  if (prefix.length === 0) {
    return localTargetPath;
  }
  return `${prefix.replace(/\/+$/, "")}/${localTargetPath}`;
}

export function substituteFixtureResourceUrls({
  sourceText,
  manifest,
  localUrlPrefix = "/",
}) {
  if (typeof sourceText !== "string") {
    throw new TypeError("sourceText must be a string");
  }
  if (!Array.isArray(manifest)) {
    throw new TypeError("manifest must be an array");
  }

  const replacements = new Map();
  for (const entry of manifest) {
    assertFixtureResourceManifestEntry(entry, {requireSha256: true});
    const replacement = localResourceUrl(
      localUrlPrefix,
      entry.local_target_path,
    );
    const previous = replacements.get(entry.original_url);
    if (previous !== undefined && previous !== replacement) {
      throw new Error(
        `manifest maps one original URL to multiple local paths: ${entry.original_url}`,
      );
    }
    replacements.set(entry.original_url, replacement);
  }

  let substitutions = 0;
  const text = sourceText.replace(
    createFixtureLocalResourceUrlRegExp(),
    (rawToken) => {
      let parsedToken;
      try {
        parsedToken = parseFixtureLocalResourceUrlToken(rawToken);
      } catch {
        return rawToken;
      }

      const replacement = replacements.get(parsedToken.canonicalUrl);
      if (replacement === undefined) {
        return rawToken;
      }

      substitutions += 1;
      return (
        `${parsedToken.leadingText}${replacement}` +
        `${parsedToken.parsed.search}${parsedToken.parsed.hash}` +
        parsedToken.trailingText
      );
    },
  );

  return {text, substitutions};
}
