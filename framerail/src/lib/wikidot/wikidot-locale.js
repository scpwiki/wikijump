/**
 * @param {string | null | undefined} locale
 * @returns {boolean}
 */
export const isJapaneseWikidotLocale = (locale) => {
  const normalized = `${locale ?? ""}`.toLowerCase().replaceAll("_", "-")
  return normalized === "ja" || normalized.startsWith("ja-") || normalized === "jp"
}

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
