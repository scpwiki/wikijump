import type { Locale } from "./types"

// prettier-ignore
const DEFAULT_LOCALE: Locale = {
  pattern: /\p{L}(?![\p{L}\p{Nd}'’])|[\p{L}\p{Nd}][\p{L}\p{Nd}'’]*[\p{L}\p{Nd}]/gu,
  filters: [
      /^\p{L}$/u,               // single characters
      /^\p{Nd}+$/u,             // pure numbers
      /^\p{Nd}+\p{L}+$/u,       // e.g. 5GW, 20mm, etc.
      /^[\p{Lu}\p{Nd}]{2,5}$/u, // SCP, MTF, etc. capitalized initialisms
  ]
}

export const LOCALES = makeLocales({})

export function getLocale(locale: string) {
  return LOCALES[locale] ?? DEFAULT_LOCALE
}

function makeLocales(locales: Record<string, Partial<Locale>>): Record<string, Locale> {
  const out: Record<string, Locale> = {}
  for (const key in locales) {
    const { pattern = DEFAULT_LOCALE.pattern, filters = [] } = locales[key]
    out[key] = { pattern, filters: [...DEFAULT_LOCALE.filters, ...filters] }
  }
  return out
}
