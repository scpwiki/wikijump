const WIKIDOT_SITES_WITH_SITE_WATCH_TEXT = new Set(["sandbox-for-codex"])

/**
 * @typedef {object} WikidotPageWatchLabel
 * @property {string} label
 * @property {string} helpLabel
 * @property {string} helpHref
 */

/**
 * @param {{
 *   sourceSite?: string | null
 *   hasSession?: boolean
 * }} input
 * @returns {WikidotPageWatchLabel | null}
 */
export const buildWikidotPageWatchLabel = ({ sourceSite, hasSession = false }) => {
  if (!hasSession || !WIKIDOT_SITES_WITH_SITE_WATCH_TEXT.has(sourceSite ?? "")) {
    return null
  }

  return {
    label: `Stop watching site ${sourceSite}.wikidot.com`,
    helpLabel: "?",
    helpHref: "http://www.wikidot.com/faq:watching"
  }
}
