/**
 * @param {string | null | undefined} locale
 * @returns {boolean}
 */
export const isJapaneseWikidotLocale = (locale) => {
  const normalized = `${locale ?? ""}`.toLowerCase().replaceAll("_", "-")
  return normalized === "ja" || normalized.startsWith("ja-") || normalized === "jp"
}
