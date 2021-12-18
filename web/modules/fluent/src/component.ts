import { FluentResource } from "@fluent/bundle"
import { type FluentImportMap } from "./locales"

export class FluentComponent {
  private declare cache: Map<string, FluentResource>

  private declare map: FluentImportMap

  declare component: string

  declare supportedLocales: string[]

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
