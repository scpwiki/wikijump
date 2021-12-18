import { FluentComponent } from "./component"

export const FALLBACK_LOCALE = "en"

const filenameRegex = /[^\/]+\.ftl$/
const componentRegex = /[^\/]+(?=\/[^\/]+\.ftl$)/
const localeRegex = /.+(?=\.ftl$)/

// only import folders one level deep
// TODO: do we need nested directories?
const sources = import.meta.glob("/../locales/fluent/*/*.ftl")

export type FluentImportMap = Record<string, null | (() => Promise<string>)>

const directory: Record<string, FluentImportMap> = {}
for (const [path, importer] of Object.entries(sources)) {
  // our path is going to be something like:
  // ../../locales/fluent/foo/en.ftl
  //                        ^  ^ locale
  //                        ^ component

  const filename = filenameRegex.exec(path)?.[0]
  const component = componentRegex.exec(path)?.[0]
  const locale = filename ? localeRegex.exec(filename)?.[0] : null

  if (!filename || !component || !locale) continue

  // looks wacky but we're just getting the existing map,
  // and if there isn't one we make a new object for it and set it
  const map = directory[component] ?? (directory[component] = {})

  // the string will be on the `default` export
  map[locale] = async () => (await importer()).default
}

export const LOCALE_COMPONENTS = new Map<string, FluentComponent>()

// populate map with all of the components we found
for (const [component, map] of Object.entries(directory)) {
  LOCALE_COMPONENTS.set(component, new FluentComponent(component, map))
}
