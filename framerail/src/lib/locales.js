import { parse } from "accept-language-parser"

const FALLBACK_LOCALE = "en"

export const MAX_LOCALE_PREFERENCES = 16
export const MAX_LOCALE_LENGTH = 64

/**
 * @param {string[]} locales
 * @returns {string[]}
 */
export const uniqueLocales = (locales) => {
  const seen = new Set()
  const unique = []

  for (const locale of locales) {
    if (!locale || seen.has(locale)) continue
    seen.add(locale)
    unique.push(locale)
  }

  return unique
}

/**
 * Returns a bounded, deduplicated locale preference list for request-time
 * translation and persisted user preferences. Locale syntax is still
 * validated by Deepwell; this guard prevents oversized valid lists from
 * amplifying work.
 *
 * @param {string[]} locales
 * @param {string[]} [requiredLocales]
 * @returns {string[]}
 */
export const limitLocalePreferences = (locales, requiredLocales = []) => {
  const limited = []
  const seen = new Set()
  const required = new Set(
    requiredLocales
      .map((locale) => locale?.trim())
      .filter((locale) => locale && locale.length <= MAX_LOCALE_LENGTH)
  )
  const remainingRequired = new Set(required)

  for (const locale of locales) {
    const value = locale?.trim()
    if (!value || value.length > MAX_LOCALE_LENGTH || seen.has(value)) continue

    if (
      !required.has(value) &&
      limited.length >= MAX_LOCALE_PREFERENCES - remainingRequired.size
    ) {
      continue
    }

    seen.add(value)
    limited.push(value)
    remainingRequired.delete(value)
    if (limited.length >= MAX_LOCALE_PREFERENCES) break
  }

  return limited
}

/**
 * @param {string[]} locales
 * @param {string} fallbackLocale
 * @returns {string[]}
 */
export const withFallbackLocale = (locales, fallbackLocale = FALLBACK_LOCALE) => {
  return uniqueLocales([...locales, fallbackLocale])
}

/**
 * @param {ReturnType<typeof parse>[number]} lang
 * @returns {string}
 */
const formatParsedLocale = (lang) => {
  const parts = [lang.code]
  if (lang.script) parts.push(lang.script)
  if (lang.region) parts.push(lang.region)
  return parts.join("-")
}

/**
 * @param {Request} req
 * @returns {string[]}
 */
export const parseAcceptLangHeader = (req) => {
  const language = req.headers.get("Accept-Language")
  if (language === null) return []

  const locales = parse(language)
    .sort((a, b) => b.quality - a.quality)
    .map(formatParsedLocale)
    .filter((locale) => locale !== "*" && !locale.includes("*"))

  return limitLocalePreferences(locales)
}
