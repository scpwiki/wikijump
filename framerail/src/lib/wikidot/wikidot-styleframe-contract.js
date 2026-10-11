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
  parsed.hostname = localHost
  parsed.port = localOrigin.port
  return parsed.toString()
}
