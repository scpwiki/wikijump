import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

const WIKIDOT_SITES_WITHOUT_STANDARD_PAGE_ACTIONS = new Set(["sandbox-for-codex"])

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
 * @property {string} append
 * @property {string} backlinks
 */

/**
 * @param {readonly string[] | null | undefined} tags
 * @returns {boolean}
 */
export const isWikidotFragmentPage = (tags) => tags?.includes("fragment") ?? false

/**
 * Open the live Wikidot printer-friendly child window for the current page
 * path. The path already begins with `/`, so the route keeps Wikidot's
 * exact doubled slash (`/printer--friendly//<page>`).
 *
 * @param {string} pagePath
 * @param {{ open: (url: string, target: string) => unknown }} [browser]
 */
export const openWikidotPrinterFriendly = (pagePath, browser = window) =>
  browser.open(`/printer--friendly/${pagePath}`, "_blank")

/**
 * @param {string | null | undefined} sourceSite
 * @returns {boolean}
 */
export const sourceShowsStandardWikidotPageActions = (sourceSite) => {
  return !WIKIDOT_SITES_WITHOUT_STANDARD_PAGE_ACTIONS.has(sourceSite ?? "")
}

/**
 * @typedef {{
 *   status?: string
 *   message?: string
 *   thread_id?: number
 *   thread_unix_title?: string
 * }} WikidotPageDiscussionResponse
 */

/**
 * Request the page discussion using Wikidot's ForumAction envelope.
 *
 * @param {typeof fetch} fetchImplementation
 * @param {number} pageId
 * @returns {Promise<WikidotPageDiscussionResponse>}
 */
export const requestWikidotPageDiscussion = async (fetchImplementation, pageId) => {
  if (!Number.isSafeInteger(pageId) || pageId <= 0) {
    throw new TypeError("A positive page ID is required to open its discussion.")
  }

  const form = new URLSearchParams({
    moduleName: "Empty",
    page_id: String(pageId),
    action: "ForumAction",
    event: "createPageDiscussionThread"
  })
  const response = await fetchImplementation("/ajax-module-connector.php", {
    method: "POST",
    credentials: "same-origin",
    headers: { "content-type": "application/x-www-form-urlencoded" },
    body: form.toString()
  })
  return response.json()
}

/** @param {WikidotPageDiscussionResponse} response */
export const wikidotPageDiscussionPath = (response) => {
  if (
    response?.status !== "ok" ||
    !Number.isSafeInteger(response?.thread_id) ||
    response.thread_id <= 0 ||
    typeof response.thread_unix_title !== "string" ||
    response.thread_unix_title.length === 0
  ) {
    return null
  }
  return `/forum/t-${response.thread_id}/${response.thread_unix_title}`
}

/**
 * @param {{
 *   sourceSite?: string | null
 *   ratingEnabled: boolean
 *   discussionEnabled: boolean
 *   discussionThreadId?: number | null
 *   sourceCommentCount?: number | null
 * }} input
 * @returns {{ showRate: boolean; showDiscuss: boolean }}
 */
export const wikidotPageActionVisibility = ({
  sourceSite,
  ratingEnabled,
  discussionEnabled,
  sourceCommentCount
}) => {
  const sourceShowsActions = sourceShowsStandardWikidotPageActions(sourceSite)
  return {
    showRate: sourceShowsActions && ratingEnabled,
    showDiscuss:
      sourceShowsActions && (discussionEnabled || (sourceCommentCount ?? 0) > 0)
  }
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
        options: "オプション",
        append: "追加",
        backlinks: "バックリンク"
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
        options: "Options",
        append: "Append",
        backlinks: "Backlinks"
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
    options: labels.options,
    append: labels.append,
    backlinks: labels.backlinks
  }
}

/**
 * @param {number} value
 * @returns {string}
 */
export const formatSigned = (value) => {
  return value > 0 ? `+${value}` : `${value}`
}
