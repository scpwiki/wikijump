import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

const WIKIDOT_SITES_WITHOUT_STANDARD_PAGE_ACTIONS = new Set(["sandbox-for-codex"])
const escapeHtml = (value) =>
  value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;")

/**
 * @typedef {object} WikidotPageActionLabels
 * @property {string} rate
 * @property {string} ratePrefix
 * @property {string | null} ratingText
 * @property {string} discuss
 * @property {boolean} showRate
 * @property {boolean} showDiscuss
 * @property {string} edit
 * @property {string} tags
 * @property {string} history
 * @property {string} files
 * @property {string} print
 * @property {string} siteTools
 * @property {string} options
 */

/**
 * @param {readonly string[] | null | undefined} tags
 * @returns {boolean}
 */
export const isWikidotFragmentPage = (tags) => tags?.includes("fragment") ?? false

export const buildWikidotDiscussButtonHtml = (label) =>
  `<a href="javascript:;" class="btn btn-default" id="discuss-button" onclick="WIKIDOT.page.listeners.createPageDiscussion(event)">${escapeHtml(label)}</a>`

/**
 * @param {string | null | undefined} sourceSite
 * @returns {boolean}
 */
export const sourceShowsStandardWikidotPageActions = (sourceSite) => {
  return !WIKIDOT_SITES_WITHOUT_STANDARD_PAGE_ACTIONS.has(sourceSite ?? "")
}

/**
 * @param {{
 *   rating?: number | null
 *   comments?: number | null
 *   locale?: string | null
 *   showRate?: boolean
 *   showDiscuss?: boolean
 * }} snapshot
 * @returns {WikidotPageActionLabels}
 */
export const buildWikidotPageActionLabels = ({
  rating,
  comments,
  locale = "en",
  showRate = true,
  showDiscuss = true
}) => {
  const ratingText = rating === null || rating === undefined ? null : formatSigned(rating)
  const labels = isJapaneseWikidotLocale(locale)
    ? {
        edit: "編集",
        ratePrefix: "評価",
        tags: "タグ",
        discuss: "ディスカッション",
        history: "履歴",
        files: "ファイル",
        print: "印刷",
        siteTools: "サイトツール",
        options: "オプション"
      }
    : {
        edit: "Edit",
        ratePrefix: "Rate",
        tags: "Tags",
        discuss: "Discuss",
        history: "History",
        files: "Files",
        print: "Print",
        siteTools: "Site tools",
        options: "Options"
      }

  return {
    ratingText,
    showRate,
    showDiscuss,
    edit: labels.edit,
    ratePrefix: labels.ratePrefix,
    rate:
      ratingText === null ? labels.ratePrefix : `${labels.ratePrefix} (${ratingText})`,
    tags: labels.tags,
    discuss:
      comments === null || comments === undefined
        ? labels.discuss
        : `${labels.discuss} (${comments})`,
    history: labels.history,
    files: labels.files,
    print: labels.print,
    siteTools: labels.siteTools,
    options: labels.options
  }
}

/**
 * @param {number} value
 * @returns {string}
 */
export const formatSigned = (value) => {
  return value > 0 ? `+${value}` : `${value}`
}
