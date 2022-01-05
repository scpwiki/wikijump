import { dedupe, Pref } from "@wikijump/util"
import {
  Locale,
  type FluentData,
  type UnitFormatOptions,
  type UnitString
} from "./locale-bundle"
import { FALLBACK_LOCALE } from "./locales"

const navigatorLocales = dedupe([navigator.language, ...navigator.languages])

const initialLocale = Pref.get("locale", navigatorLocales[0] ?? FALLBACK_LOCALE)
const fallbackLocales = [...navigatorLocales]

// avoid our initial locale also being in our fallbacks
if (fallbackLocales[0] === initialLocale) fallbackLocales.shift()

// push fallback locale at the end if we don't have it anywhere
if (initialLocale !== FALLBACK_LOCALE && !fallbackLocales.includes(FALLBACK_LOCALE)) {
  fallbackLocales.push(FALLBACK_LOCALE)
}

/** Default locale using browser language settings. */
export const defaultLocale = new Locale(initialLocale, ...fallbackLocales)

export default defaultLocale

/**
 * Formats a message via a selector.
 *
 * @param selector - The selector for getting the message.
 * @param data - Data to pass to the message's pattern when formatting.
 * @param fallback - A fallback message to use if the message isn't found.
 *   If this is given, a warning won't be shown for missing messages. This
 *   is useful for components that are loaded asynchronously.
 */
export function format(selector: string, data?: FluentData, fallback?: string) {
  return defaultLocale.format(selector, data, fallback)
}

/**
 * Formats a number.
 *
 * @param n - The number to format.
 * @param opts - Options for formatting.
 */
export function number(n: number, opts?: Intl.NumberFormatOptions) {
  return defaultLocale.number(n, opts)
}

/**
 * Formats a number as a unit, e.g. `20mm`.
 *
 * @param n - The number to format.
 * @param unit - The unit to use.
 * @param opts - Options for formatting.
 */
export function unit(n: number, unit: UnitString, opts?: UnitFormatOptions) {
  return defaultLocale.unit(n, unit, opts)
}

/**
 * Formats a timestamp, date string, or `Date`.
 *
 * @param date - The date to format.
 * @param opts - Options for formatting.
 */
export function date(date: number | string | Date, opts?: Intl.DateTimeFormatOptions) {
  return defaultLocale.date(date, opts)
}

// top level await shenanigans
// this causes the browser to wait on this script
// before it continues rendering the page any further
await defaultLocale.load("base")
