import { uniqueLocales } from "./locales.js"

/**
 * @param {string[]} locales
 * @param {string} locale
 * @returns {string[]}
 */
export const addLocalePreference = (locales, locale) => {
  if (!locale) return uniqueLocales(locales)
  return uniqueLocales([...locales, locale])
}

/**
 * @param {string[]} locales
 * @param {string} locale
 * @returns {string[]}
 */
export const removeLocalePreference = (locales, locale) => {
  return uniqueLocales(locales).filter((value) => value !== locale)
}

/**
 * @param {string[]} locales
 * @param {string} locale
 * @param {-1 | 1} offset
 * @returns {string[]}
 */
export const moveLocalePreference = (locales, locale, offset) => {
  const ordered = uniqueLocales(locales)
  const index = ordered.indexOf(locale)
  const target = index + offset
  if (index < 0 || target < 0 || target >= ordered.length) return ordered

  ;[ordered[index], ordered[target]] = [ordered[target], ordered[index]]
  return ordered
}
