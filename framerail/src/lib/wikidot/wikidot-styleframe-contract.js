export const STYLEFRAME_MARKER = "wikidot-style-frame"
export const STYLEFRAME_REGISTRY = "__wikijumpStyleFrameRegistry"
export const STYLEFRAME_PRELOADED = "wikidotStylePreloaded"

/**
 * @param {string | null | undefined} value
 * @returns {string}
 */
export const escapeStyleFrameHtml = (value) => {
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
export const safeInlineStyleFrameCss = (value) => {
  return value.replace(/<\/(style|script)/giu, "<\\/$1")
}

/**
 * @param {unknown} value
 * @returns {string}
 */
export const safeStyleFrameScriptJson = (value) => {
  return JSON.stringify(value).replace(/</gu, "\\u003c")
}

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
