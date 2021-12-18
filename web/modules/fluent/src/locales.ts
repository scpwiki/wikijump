import { FluentComponent } from "./component"

export const FALLBACK_LOCALE = "en"
export const SUPPORTED_LOCALES = ["en"] as const

const filenameRegex = /[^\/]+\.ftl$/
const componentRegex = /[^\/]+(?>=\/[^\/]+\.ftl$)/
const localeRegex = /.+(?>=\.ftl$)/

// only import folders one level deep
// TODO: do we need nested directories?
const sources = import.meta.glob("@root/locales/fluent/*/*.ftl?raw")

export type SupportedLocale = typeof FALLBACK_LOCALE | typeof SUPPORTED_LOCALES[number]

export type FluentImportMap = Record<string, null | (() => Promise<string>)>

const directory: Record<string, FluentImportMap> = {}
for (const [path, importer] of Object.entries(sources)) {
  // our path is going to be something like:
  // ../../locales/fluent/component/en.ftl
  // we want everything after `fluent/`.

  const filename = filenameRegex.exec(path)?.[0]
  const component = componentRegex.exec(path)?.[0]
  const locale = filename ? localeRegex.exec(filename)?.[0] : null

  if (!filename || !component || !locale) continue

  if (!isSupportedLocale(locale)) continue

  // looks wacky but we're just getting the existing map,
  // and if there isn't one we make a new object for it and set it
  const map = directory[component] ?? (directory[component] = {} as any)

  // the string will be on the `default` export
  map[locale] = async () => (await importer()).default
}

export const LOCALE_COMPONENTS = new Map<string, FluentComponent>()

// populate map with all of the components we found
for (const [component, map] of Object.entries(directory)) {
  LOCALE_COMPONENTS.set(component, new FluentComponent(component, map))
}

/**
 * Checks if a locale is supported.
 *
 * @param locale - The locale string to check.
 */
export function isSupportedLocale(locale: string): locale is SupportedLocale {
  // TODO: more complicated parsing
  return SUPPORTED_LOCALES.includes(locale as any)
}
