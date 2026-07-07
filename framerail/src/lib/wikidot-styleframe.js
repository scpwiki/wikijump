/**
 * @typedef {{
 *   priority?: string | null
 *   themes?: string[]
 *   css?: string | null
 *   origin?: string | null
 * }} WikidotStyleFrameInput
 */

const STYLEFRAME_MARKER = "wikidot-style-frame"

/**
 * @param {string | null | undefined} value
 * @returns {string}
 */
const escapeHtml = (value) => {
  return `${value ?? ""}`
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
}

/**
 * @param {string} value
 * @returns {string}
 */
const safeInlineCss = (value) => value.replace(/<\/(style|script)/giu, "<\\/$1")

/**
 * @param {string} value
 * @returns {string}
 */
const safeScriptJson = (value) => JSON.stringify(value).replace(/</gu, "\\u003c")

/**
 * @param {string | null | undefined} value
 * @returns {boolean}
 */
export const isUsableStyleFrameCss = (value) => {
  if (!value) return false
  const trimmed = value.trim()
  return trimmed.length > 0 && !["{$css}", "$css"].includes(trimmed)
}

/**
 * Map Wikidot page-owned local files/code to the current local file host.
 * CDN theme URLs are intentionally preserved; offline mirroring is a later
 * policy slice, not hidden here.
 *
 * @param {string} themeUrl
 * @param {string | null | undefined} origin
 * @returns {string}
 */
export const localizeWikidotThemeUrl = (themeUrl, origin) => {
  if (!origin) return themeUrl
  let parsed
  let localOrigin
  try {
    parsed = new URL(themeUrl)
    localOrigin = new URL(origin)
  } catch {
    return themeUrl
  }

  if (
    !["scp-wiki.wikidot.com", "scp-wiki.wdfiles.com"].includes(parsed.hostname) ||
    !parsed.pathname.startsWith("/local--")
  ) {
    return themeUrl
  }

  const localHost = localOrigin.hostname.replace(
    /\.wikijump\.localhost$/u,
    ".wjfiles.localhost"
  )
  if (localHost === localOrigin.hostname) return themeUrl
  parsed.protocol = localOrigin.hostname.endsWith(".wikijump.localhost")
    ? "https:"
    : localOrigin.protocol
  parsed.host = localHost
  return parsed.toString()
}

/**
 * @param {WikidotStyleFrameInput} input
 * @returns {string}
 */
export const buildWikidotStyleFrameHtml = ({
  priority = "",
  themes = [],
  css = "",
  origin = null
}) => {
  const localizedThemes = themes
    .filter((theme) => theme.trim().length > 0)
    .map((theme) => localizeWikidotThemeUrl(theme, origin))
  const inlineCss = isUsableStyleFrameCss(css) ? safeInlineCss(css ?? "") : ""
  const themeList = localizedThemes
    .map((theme) => `<li>${escapeHtml(theme)}</li>`)
    .join("")
  const styleBlock = inlineCss ? `<style>${inlineCss}</style>` : ""
  const script = `(() => {
  const marker = ${safeScriptJson(STYLEFRAME_MARKER)};
  const priority = ${safeScriptJson(priority ?? "")};
  const priorityNumber = Number.parseFloat(priority);
  const priorityValue = Number.isFinite(priorityNumber) ? priorityNumber : 0;
  const themes = ${JSON.stringify(localizedThemes).replace(/</gu, "\\u003c")};
  const css = ${safeScriptJson(inlineCss)};
  const targetDocument = window.parent && window.parent !== window
    ? window.parent.document
    : document;
  const head = targetDocument.head || targetDocument.documentElement;
  const markedStyleNodes = () => Array.from(
    head.querySelectorAll('[data-wikidot-style-frame="' + marker + '"]')
  );
  const stylePriority = (node) => {
    const value = Number.parseFloat(node.dataset.wikidotStylePriority || "");
    return Number.isFinite(value) ? value : 0;
  };
  const restoreStyleFrameOrder = () => {
    markedStyleNodes()
      .sort((left, right) => stylePriority(left) - stylePriority(right))
      .forEach((node) => head.appendChild(node));
  };
  const scheduleStyleFrameOrderRestore = () => {
    setTimeout(restoreStyleFrameOrder, 0);
    setTimeout(restoreStyleFrameOrder, 250);
    if (typeof targetDocument.defaultView?.requestAnimationFrame === "function") {
      targetDocument.defaultView.requestAnimationFrame(() => {
        targetDocument.defaultView.requestAnimationFrame(restoreStyleFrameOrder);
      });
    }
  };
  const appendMarked = (element, id) => {
    element.dataset.wikidotStyleFrame = marker;
    element.dataset.wikidotStylePriority = priority;
    element.dataset.wikidotStyleId = id;
    const laterStyle = markedStyleNodes().find((node) => stylePriority(node) > priorityValue);
    if (laterStyle) {
      head.insertBefore(element, laterStyle);
    } else {
      head.appendChild(element);
    }
  };
  themes.forEach((href, index) => {
    const link = targetDocument.createElement("link");
    link.rel = "stylesheet";
    link.href = href;
    appendMarked(link, \`theme-\${index}\`);
  });
  if (css.trim().length > 0) {
    const style = targetDocument.createElement("style");
    style.textContent = css;
    appendMarked(style, "inline-css");
  }
  scheduleStyleFrameOrderRestore();
})();`

  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Local Wikidot style frame</title>
    <meta name="wikidot-style-priority" content="${escapeHtml(priority)}">
    <meta name="wikidot-style-theme-count" content="${localizedThemes.length}">
    <meta name="wikidot-style-inline-css" content="${inlineCss ? "true" : "false"}">
    ${styleBlock}
  </head>
  <body>
    <ul hidden>${themeList}</ul>
    <script>${script}</script>
  </body>
</html>
`
}
