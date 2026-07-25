/**
 * @typedef {{
 *   priority?: string | null
 *   themes?: string[]
 *   css?: string | null
 *   origin?: string | null
 * }} WikidotStyleFrameInput
 */

const STYLEFRAME_MARKER = "wikidot-style-frame"
const STYLEFRAME_REGISTRY = "__wikijumpStyleFrameRegistry"
const STYLEFRAME_PRELOADED = "wikidotStylePreloaded"
const styleFrameSource =
  /<iframe\b[^>]*\bsrc=(['"])([^'"]*\/-\/wikidot-interwiki\/styleFrame\.html\?[^'"]*)\1[^>]*>/giu

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
 * Extract the stylesheet contract already declared by rendered styleFrame
 * iframes. Rendering these links in the document head makes their CSS part
 * of the initial document load, as it is on Wikidot, instead of allowing
 * the local iframe to apply the theme only after DOMContentLoaded.
 *
 * @param {(string | null | undefined)[]} renderedHtml
 * @param {string | null | undefined} origin
 * @returns {{
 *   href: string
 *   priority: string
 *   priorityValue: number
 *   order: number
 * }[]}
 */
export const extractWikidotStyleFrameStylesheets = (renderedHtml, origin) => {
  const stylesheets = []
  let order = 0
  for (const html of renderedHtml) {
    if (!html) continue
    for (const match of html.matchAll(styleFrameSource)) {
      const source = match[2].replaceAll("&amp;", "&")
      let parsed
      try {
        parsed = new URL(source, origin || "https://wikijump.invalid")
      } catch {
        continue
      }
      if (parsed.pathname !== "/-/wikidot-interwiki/styleFrame.html") continue
      const priority = parsed.searchParams.get("priority") ?? ""
      const numericPriority = Number.parseFloat(priority)
      const priorityValue = Number.isFinite(numericPriority) ? numericPriority : 0
      for (const theme of parsed.searchParams.getAll("theme")) {
        if (!theme.trim()) continue
        stylesheets.push({
          href: localizeWikidotThemeUrl(theme, origin),
          priority,
          priorityValue,
          order: order++
        })
      }
    }
  }
  return stylesheets.sort(
    (left, right) => left.priorityValue - right.priorityValue || left.order - right.order
  )
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
  const targetWindow = window.parent && window.parent !== window
    ? window.parent
    : window;
  const targetDocument = targetWindow.document;
  const frameElement = window.frameElement || null;
  const registryKey = ${safeScriptJson(STYLEFRAME_REGISTRY)};
  const registry = targetWindow[registryKey] || (targetWindow[registryKey] = {
    nextOwnerId: 1,
    owners: new Map()
  });
  const removeOwner = (owner) => {
    const registration = registry.owners.get(owner);
    targetDocument.querySelectorAll(
      '[data-wikidot-style-owner="' + owner + '"]'
    ).forEach((node) => node.remove());
    if (registration?.frame?.dataset.wikidotStyleOwner === owner) {
      delete registration.frame.dataset.wikidotStyleOwner;
    }
    registration?.observer?.disconnect();
    registry.owners.delete(owner);
  };
  registry.owners.forEach((registration, owner) => {
    if (registration.frame && !registration.frame.isConnected) removeOwner(owner);
  });
  const previousOwner = frameElement?.dataset.wikidotStyleOwner;
  if (previousOwner) removeOwner(previousOwner);
  const owner = marker + "-" + registry.nextOwnerId++;
  if (frameElement) frameElement.dataset.wikidotStyleOwner = owner;
  registry.owners.set(owner, { frame: frameElement, observer: null });
  const cleanup = () => removeOwner(owner);
  window.addEventListener?.("pagehide", cleanup, { once: true });
  window.addEventListener?.("unload", cleanup, { once: true });
  if (frameElement && typeof targetWindow.MutationObserver === "function") {
    const observer = new targetWindow.MutationObserver(() => {
      if (!frameElement.isConnected) cleanup();
    });
    observer.observe(targetDocument.documentElement, { childList: true, subtree: true });
    registry.owners.get(owner).observer = observer;
  }
  const head = targetDocument.head || targetDocument.documentElement;
  const markedStyleNodes = () => Array.from(
    head.querySelectorAll('[data-wikidot-style-frame="' + marker + '"]')
  );
  const generatedCssNodes = () => Array.from(
    head.querySelectorAll("[data-wikijump-generated-css]")
  );
  const stylePriority = (node) => {
    const value = Number.parseFloat(node.dataset.wikidotStylePriority || "");
    return Number.isFinite(value) ? value : 0;
  };
  const restoreStyleFrameOrder = () => {
    const desiredTail = [
      ...markedStyleNodes().sort(
        (left, right) => stylePriority(left) - stylePriority(right)
      ),
      ...generatedCssNodes()
    ];
    const currentNodes = Array.from(head.children);
    const tailOffset = currentNodes.length - desiredTail.length;
    const alreadyOrdered =
      tailOffset >= 0 &&
      desiredTail.every((node, index) => currentNodes[tailOffset + index] === node);
    if (alreadyOrdered) return;
    desiredTail.forEach((node) => head.appendChild(node));
  };
  const scheduleStyleFrameOrderRestore = () => {
    restoreStyleFrameOrder();
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
    element.dataset.wikidotStyleOwner = owner;
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
    const link = Array.from(
      head.querySelectorAll('link[data-wikidot-style-preloaded]')
    ).find((candidate) =>
      candidate.href === new URL(href, targetDocument.baseURI).href &&
      candidate.dataset.wikidotStylePriority === priority
    ) || targetDocument.createElement("link");
    delete link.dataset.${STYLEFRAME_PRELOADED};
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
