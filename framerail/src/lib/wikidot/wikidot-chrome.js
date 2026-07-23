/**
 * @typedef {{
 *   site?: {
 *     slug?: string | null
 *     name?: string | null
 *     tagline?: string | null
 *     from_wikidot?: boolean | null
 *   } | null
 *   page?: { from_wikidot?: boolean | null } | null
 *   page_revision?: { from_wikidot?: boolean | null } | null
 *   wikidot_snapshot?: { source_site?: string | null } | null
 *   user_session?: {
 *     user?: { name?: string | null; slug?: string | null } | null
 *   } | null
 * }} WikidotChromeViewData
 */

export const SANDBOX_WIKIDOT_SOURCE_SITE = "sandbox-for-codex"

/**
 * @param {WikidotChromeViewData | null | undefined} data
 * @returns {boolean}
 */
export const shouldUseSandboxWikidotChrome = (data) => {
  const sourceSite = data?.wikidot_snapshot?.source_site
  const hasImportedPage = data?.page?.from_wikidot || data?.page_revision?.from_wikidot

  return (
    sourceSite === SANDBOX_WIKIDOT_SOURCE_SITE ||
    (data?.site?.slug === SANDBOX_WIKIDOT_SOURCE_SITE && !!hasImportedPage)
  )
}

/**
 * @param {WikidotChromeViewData | null | undefined} data
 * @returns {string | null | undefined}
 */
export const resolveWikidotSiteTitle = (data) => {
  if (shouldUseSandboxWikidotChrome(data)) return "Sandbox For Codex"

  return data?.site?.name
}

/**
 * @param {WikidotChromeViewData | null | undefined} data
 * @returns {string | null | undefined}
 */
export const resolveWikidotSiteTagline = (data) => {
  if (shouldUseSandboxWikidotChrome(data)) return null

  return data?.site?.tagline
}

/**
 * @param {WikidotChromeViewData | null | undefined} data
 * @returns {string | null}
 */
export const resolveWikidotSessionUserName = (data) => {
  return data?.user_session?.user?.name || data?.user_session?.user?.slug || null
}
