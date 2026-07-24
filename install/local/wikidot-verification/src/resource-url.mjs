const LOCAL_FILES_URL_RE_SOURCE =
  String.raw`https?:\/\/[^\s"'|()<>[\]{}]+\/local--files\/[^\s"'|()<>[\]{}]+`;

export function createFixtureLocalResourceUrlRegExp() {
  return new RegExp(LOCAL_FILES_URL_RE_SOURCE, "gi");
}

export function matchFixtureLocalResourceUrls(sourceText) {
  return sourceText.matchAll(createFixtureLocalResourceUrlRegExp());
}

export function parseFixtureLocalResourceUrlToken(rawToken) {
  const withoutLeadingPunctuation = rawToken.replace(/^["'`]+/, "");
  const leadingText = rawToken.slice(
    0,
    rawToken.length - withoutLeadingPunctuation.length,
  );
  const resourceUrl = withoutLeadingPunctuation.replace(
    /["'`;,)\].:!?|]+$/,
    "",
  );
  const trailingText = withoutLeadingPunctuation.slice(resourceUrl.length);
  const parsed = new URL(resourceUrl);
  const canonicalUrl = `${parsed.origin}${parsed.pathname}${parsed.search}`;

  return {
    canonicalUrl,
    leadingText,
    parsed,
    resourceUrl,
    trailingText,
  };
}
