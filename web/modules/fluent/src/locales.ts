import { FluentComponent } from "./component"

export const FALLBACK_LOCALE = "en"

export type FluentImportMap = Record<string, () => Promise<string>>

export const LOCALE_COMPONENTS = new Map<string, FluentComponent>()

export const LOCALE_CMFTML_DOCUMENTATION = new Map<string, () => Promise<any>>()

export const LOCALE_LIST = new Set<string>()

const filenameRegex = /[^\/]+\.(ftl|yaml)$/
const componentRegex = /[^\/]+(?=\/[^\/]+\.(ftl|yaml)$)/
const localeRegex = /.+(?=\.(ftl|yaml)$)/

// only import folders one level deep
const sources = import.meta.glob("/../locales/fluent/*/*.ftl")

// populate map with all of the components
for (const [component, map] of Object.entries(makeDirectory(sources))) {
  LOCALE_COMPONENTS.set(component, new FluentComponent(component, map))
}

const blocks = import.meta.glob("/../locales/cmftml/*.yaml")

// kind of hacky, but we're reusing the makeDirectory function here
// the "component" is just going to be `cmftml-blocks` because that's the folder
// these files are in
const cmftmlImportMap = makeDirectory(blocks)["cmftml"]

// just in case
if (cmftmlImportMap) {
  // transfer into our map
  for (const [locale, importer] of Object.entries(cmftmlImportMap)) {
    LOCALE_CMFTML_DOCUMENTATION.set(locale, importer)
  }
}

function makeDirectory(
  sources: Record<string, () => Promise<Record<string, any>>>
): Record<string, FluentImportMap> {
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

    LOCALE_LIST.add(locale)

    // looks wacky but we're just getting the existing map,
    // and if there isn't one we make a new object for it and set it
    const map = directory[component] ?? (directory[component] = {})

    // the string will be on the `default` export
    map[locale] = async () => (await importer()).default
  }

  return directory
}
