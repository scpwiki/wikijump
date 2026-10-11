/**
 * @param {string | null | undefined} locale
 * @returns {boolean}
 */
export const isJapaneseWikidotLocale = (locale) => {
  const normalized = `${locale ?? ""}`.toLowerCase().replaceAll("_", "-")
  return normalized === "ja" || normalized.startsWith("ja-") || normalized === "jp"
}

/**
 * @param {string | null | undefined} locale
 * @returns {string}
 */
export const editPermissionDeniedMessage = (locale) => {
  const normalized = `${locale ?? ""}`.toLowerCase().replaceAll("_", "-")
  if (isJapaneseWikidotLocale(normalized)) return "このページを編集する権限がありません。"

  const language = normalized.split("-", 1)[0]
  if (language === "ko") return "이 페이지를 편집할 권한이 없습니다."
  if (language === "zh") {
    return /^(zh-hant|zh-tw|zh-hk|zh-mo)(-|$)/u.test(normalized)
      ? "您沒有權限編輯此頁面。"
      : "您没有权限编辑此页面。"
  }

  return "You don't have permission to edit this page."
}

/**
 * Wikidot's permission dialog for a viewer who may not start creating a
 * missing page. Only the English sentence is evidenced here; the dialog's
 * sign-in link is not rendered by the native error popup.
 *
 * @returns {string}
 */
export const createPageDeniedMessage = () =>
  "Sorry, you can not create a new page in this category. Only members of this site, site administrators and perhaps selected moderators are allowed to do it."

/**
 * Wikidot exposes `ja-corrections` as a site language identifier, but it
 * is not a valid BCP 47 locale and JavaScript's Intl APIs reject it.
 * Preserve the raw identifier at the Wikidot compatibility boundary and
 * use Japanese only when passing locale preferences to Intl.
 *
 * @param {string[]} locales
 * @returns {string[]}
 */
export const toIntlLocales = (locales) =>
  locales.map((locale) =>
    locale.toLowerCase().replaceAll("_", "-") === "ja-corrections" ? "ja" : locale
  )
