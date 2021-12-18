import { FluentResource } from "@fluent/bundle"
import { type FluentImportMap } from "./locales"

/** Represents a "component", which is a set of resources in any number of locales. */
export class FluentComponent {
  /** Cache for already loaded resources. */
  private declare cache: Map<string, FluentResource>

  /** Import map used to map locales to resources. */
  private declare map: FluentImportMap

  /** The name of this component. */
  declare component: string

  /** The list of locales supported by this component. */
  declare supportedLocales: string[]

  /**
   * @param component - The name of this component.
   * @param map - The map to use to load resources.
   */
  constructor(component: string, map: FluentImportMap) {
    this.cache = new Map()
    this.map = map
    this.component = component
    this.supportedLocales = Object.keys(map)
  }

  /**
   * Checks if this component has the given locale.
   *
   * @param locale - The locale to check.
   */
  has(locale: string) {
    return this.supportedLocales.includes(locale)
  }

  /**
   * Given a list of locales, this function will return the first locale
   * string that is supported by this component.
   *
   * @param locales - The list of locales to check.
   */
  which(locales: string[]) {
    for (const locale of locales) {
      if (this.has(locale)) return locale
    }
    return null
  }

  /**
   * Loads and returns the Fluent resource for the given locale.
   *
   * @param locale - The locale to get the data for.
   */
  async load(locale: string) {
    if (!this.has(locale)) throw new Error(`Unsupported locale (${locale}) in resource`)
    if (this.cache.has(locale)) return this.cache.get(locale)!
    const resource = new FluentResource(await this.map[locale]!())
    this.cache.set(locale, resource)
    return resource
  }
}
