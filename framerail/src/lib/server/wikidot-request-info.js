export const WIKIDOT_REQUEST_INFO_MARKER = "/*__WIKIDOT_REQUEST_INFO__*/"

const SITE_SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/u

const requireSafeInteger = (value, name) => {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${name} must be a non-negative safe integer`)
  }
  return value
}

const requireString = (value, name, maximumLength = 1024) => {
  const hasControlCharacter =
    typeof value === "string" &&
    [...value].some((character) => {
      const codePoint = character.codePointAt(0)
      return codePoint !== undefined && (codePoint < 32 || codePoint === 127)
    })
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > maximumLength ||
    hasControlCharacter
  ) {
    throw new Error(`${name} must be a non-empty string without control characters`)
  }
  return value
}

const requireSiteSlug = (value) => {
  if (typeof value !== "string" || !SITE_SLUG_PATTERN.test(value)) {
    throw new Error("site slug is outside the supported unix-name grammar")
  }
  return value
}

const requireRequestHost = (value) => {
  const host = requireString(value, "request host", 261).toLowerCase()
  let parsed
  try {
    parsed = new URL(`http://${host}`)
  } catch {
    throw new Error("request host is outside the supported host grammar")
  }
  if (
    parsed.username ||
    parsed.password ||
    parsed.pathname !== "/" ||
    parsed.search ||
    parsed.hash ||
    parsed.host !== host
  ) {
    throw new Error("request host is outside the supported host grammar")
  }
  return host
}

/** @param {Request} request */
export const requestHostFromRequest = (request) => {
  return requireRequestHost(request.headers.get("host") ?? new URL(request.url).host)
}

/**
 * @param {{
 *   domain: string
 *   site: {
 *     site_id: number
 *     slug: string
 *     locale: string
 *   }
 *   page: {
 *     page_id: number
 *     page_category_id: number
 *     slug: string
 *   }
 * }} input
 */
export const buildWikidotRequestInfo = ({ domain, site, page }) => {
  const siteUnixName = requireSiteSlug(site.slug)
  const pageUnixName = requireString(page.slug, "page slug")
  return {
    domain: requireRequestHost(domain),
    siteId: requireSafeInteger(site.site_id, "site ID"),
    siteUnixName,
    categoryId: requireSafeInteger(page.page_category_id, "page category ID"),
    requestPageName: pageUnixName,
    lang: requireString(site.locale, "site locale", 64),
    pageUnixName,
    pageId: requireSafeInteger(page.page_id, "page ID")
  }
}

const scriptString = (value) =>
  JSON.stringify(value)
    .replaceAll("<", "\\u003c")
    .replaceAll("\u2028", "\\u2028")
    .replaceAll("\u2029", "\\u2029")

const htmlAttribute = (value) =>
  value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")

/** @param {ReturnType<typeof buildWikidotRequestInfo> | undefined} info */
export const serializeWikidotRequestInfo = (info) => {
  if (!info) return ""
  const validated = buildWikidotRequestInfo({
    domain: info.domain,
    site: {
      site_id: info.siteId,
      slug: info.siteUnixName,
      locale: info.lang
    },
    page: {
      page_id: info.pageId,
      page_category_id: info.categoryId,
      slug: info.pageUnixName
    }
  })
  if (validated.requestPageName !== info.requestPageName) {
    throw new Error("request page name must equal page unix name")
  }
  return [
    "var WIKIREQUEST = {};",
    "WIKIREQUEST.info = {};",
    `WIKIREQUEST.info.domain = ${scriptString(validated.domain)};`,
    `WIKIREQUEST.info.siteId = ${validated.siteId};`,
    `WIKIREQUEST.info.siteUnixName = ${scriptString(validated.siteUnixName)};`,
    `WIKIREQUEST.info.categoryId = ${validated.categoryId};`,
    `WIKIREQUEST.info.requestPageName = ${scriptString(validated.requestPageName)};`,
    `WIKIREQUEST.info.lang = ${scriptString(validated.lang)};`,
    `WIKIREQUEST.info.pageUnixName = ${scriptString(validated.pageUnixName)};`,
    `WIKIREQUEST.info.pageId = ${validated.pageId};`
  ].join("\n")
}

/**
 * @param {string} html
 * @param {ReturnType<typeof buildWikidotRequestInfo> | undefined} info
 * @param {string | undefined} siteLocale
 */
export const injectWikidotRequestInfo = (html, info, siteLocale = undefined) => {
  const localizedHtml = siteLocale
    ? html.replace(
        '<html lang="en">',
        `<html lang="${htmlAttribute(requireString(siteLocale, "site locale", 64))}">`
      )
    : html
  const first = localizedHtml.indexOf(WIKIDOT_REQUEST_INFO_MARKER)
  if (first === -1) return localizedHtml
  if (
    localizedHtml.indexOf(
      WIKIDOT_REQUEST_INFO_MARKER,
      first + WIKIDOT_REQUEST_INFO_MARKER.length
    ) !== -1
  ) {
    throw new Error("WIKIREQUEST template marker must occur at most once")
  }
  return `${localizedHtml.slice(0, first)}${serializeWikidotRequestInfo(info)}${localizedHtml.slice(first + WIKIDOT_REQUEST_INFO_MARKER.length)}`
}
