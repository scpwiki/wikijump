import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

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
 *   locale?: string | null
 * }} input
 * @returns {WikidotPageWatchLabel | null}
 */
export const buildWikidotPageWatchLabel = ({
  sourceSite,
  hasSession = false,
  locale = "en"
}) => {
  if (!hasSession || !sourceSite) {
    return null
  }

  if (isJapaneseWikidotLocale(locale)) {
    return {
      label: "このサイトのウォッチングを終了",
      helpLabel: "?",
      helpHref: "http://www.wikidot.com/faq:watching"
    }
  }

  return {
    label: `Stop watching site ${sourceSite}.wikidot.com`,
    helpLabel: "?",
    helpHref: "http://www.wikidot.com/faq:watching"
  }
}
