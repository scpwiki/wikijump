import { dedupe, Pref } from "@wikijump/util"
import { Locale, type FluentData } from "./locale-bundle"
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

// TODO: can this be made reactive like svelte-i18n?

/** Default locale using browser data. */
export const defaultLocale = new Locale(initialLocale, ...fallbackLocales)

export default defaultLocale

/** Formats a message via its ID. Alias for the `defaultLocale.format` function. */
export function t(id: string, data?: FluentData) {
  return defaultLocale.format(id, data)
}

// top level await shenanigans
// this causes the browser to wait on this script
// until it continues rendering the page any further
await defaultLocale.load("base")
