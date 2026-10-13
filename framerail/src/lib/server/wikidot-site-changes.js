const BROWSER_FIELDS = new Set(["page", "perpage", "pageId", "categoryId", "options"])
const BROWSER_OPTIONS = new Set([
  "{}",
  '{"all":true}',
  '{"source":true}',
  '{"files":true}'
])
// The rendered #rev-perpage control offers 10/20/50/100/200; "-1", "0", and
// "1" are the observed control/empty shapes accepted alongside them.
const BROWSER_PERPAGE = new Set(["-1", "0", "1", "10", "20", "50", "100", "200"])
const CONTROL_FIELDS = new Set([
  "moduleName",
  "wikidot_token7",
  "callbackIndex",
  "eventSource"
])
const WIKIDOT_PY_FIELDS = new Set(["page", "perpage", "options"])
const FORBIDDEN_WIKIDOT_PY_EXTRA_FIELDS = new Set([
  "pageId",
  "categoryId",
  "module_body",
  "action",
  "event"
])

/** @param {string | undefined} value */
const isPositiveSafeDecimal = (value) => {
  if (value === undefined || !/^[1-9][0-9]*$/u.test(value)) return false
  return Number.isSafeInteger(Number.parseInt(value, 10))
}

/** @param {string | undefined} value */
const normalizeWikidotPyPage = (value) => {
  if (isPositiveSafeDecimal(value)) return value
  return value !== undefined && /^[A-Za-z-]{1,64}$/u.test(value) ? "1" : null
}

/** @param {string | undefined} value */
const normalizeWikidotPyPerPage = (value) => {
  if (value === "20" || value === "1000") return value
  return value !== undefined && /^[A-Za-z-]{1,64}$/u.test(value) ? value : null
}

/**
 * Classify and normalize the two evidenced SiteChanges request families.
 *
 * @param {Map<string, string>} fields
 * @returns {{
 *   pageId?: string
 *   page: string
 *   perpage: string
 *   categoryId?: string
 *   options: string
 * } | null}
 */
export const classifyWikidotSiteChangesRequest = (fields) => {
  const parameters = new Map()
  for (const [key, value] of fields) {
    if (CONTROL_FIELDS.has(key)) continue
    parameters.set(key, value)
  }

  const isBrowser =
    parameters.size === BROWSER_FIELDS.size &&
    [...BROWSER_FIELDS].every((name) => parameters.has(name))
  if (isBrowser) {
    const page = parameters.get("page")
    const perpage = parameters.get("perpage")
    const pageId = parameters.get("pageId")
    const categoryId = parameters.get("categoryId")
    const rawOptions = parameters.get("options")
    const options =
      rawOptions === "{'all':true}"
        ? '{"all":true}'
        : rawOptions === "{'source':true}"
          ? '{"source":true}'
          : rawOptions === "{'files':true}"
            ? '{"files":true}'
            : rawOptions
    if (
      isPositiveSafeDecimal(page) &&
      BROWSER_PERPAGE.has(perpage) &&
      isPositiveSafeDecimal(pageId) &&
      (categoryId === "" || isPositiveSafeDecimal(categoryId)) &&
      options !== undefined &&
      BROWSER_OPTIONS.has(options)
    ) {
      return { pageId, page, perpage, categoryId, options }
    }
    return null
  }

  const extras = [...parameters].filter(([name]) => !WIKIDOT_PY_FIELDS.has(name))
  const hasSupportedExtra =
    extras.length === 0 ||
    (parameters.has("options") &&
      extras.length === 1 &&
      !FORBIDDEN_WIKIDOT_PY_EXTRA_FIELDS.has(extras[0][0]) &&
      /^[A-Za-z][A-Za-z0-9_-]{0,63}$/u.test(extras[0][0]) &&
      extras[0][1].length <= 256)
  const isWikidotPy =
    parameters.has("page") && parameters.has("perpage") && hasSupportedExtra
  if (isWikidotPy) {
    const page = parameters.get("page")
    const perpage = parameters.get("perpage")
    const options = parameters.get("options")
    const normalizedOptions =
      options === undefined ||
      options === "{}" ||
      options === "{'all':true}" ||
      options === '{"all":true}'
        ? '{"all":true}'
        : options === "{'source':true}" || options === '{"source":true}'
          ? '{"source":true}'
          : options === '{"files":true}' || options === "{'files':true}"
            ? '{"files":true}'
            : null
    const normalizedPage = normalizeWikidotPyPage(page)
    const normalizedPerpage = normalizeWikidotPyPerPage(perpage)
    if (
      normalizedPage === null ||
      normalizedPerpage === null ||
      normalizedOptions === null
    ) {
      return null
    }
    return {
      page: normalizedPage,
      perpage: normalizedPerpage,
      options: normalizedOptions
    }
  }

  return null
}
