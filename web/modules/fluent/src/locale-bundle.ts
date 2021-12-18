import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle"
import { FluentComponent } from "./component"
import { LOCALE_COMPONENTS } from "./locales"

/** Class that handles translations/formatting for a specific locale. */
export class Locale {
  /** Internal bundle that stores and formats messages. */
  private declare bundle: FluentBundle

  /** The primary locale. */
  declare locale: string

  /**
   * Fallback locales that are used only if the primary locale isn't
   * available for a message.
   */
  declare fallbacks: string[]

  /** A set containing every component that has already been loaded. */
  declare loadedComponents: Set<FluentComponent>

  /** @param locales - Locale strings. First locale is the primary one. */
  constructor(...locales: string[]) {
    this.locale = locales[0]
    this.fallbacks = locales.slice(1)
    this.loadedComponents = new Set()
    this.bundle = new FluentBundle(this.locale)
  }

  /**
   * Every locale supported, with the primary locale being the first index
   * and the fallbacks being the rest.
   */
  get supported() {
    return [this.locale, ...this.fallbacks]
  }

  /**
   * Adds a new resource or component to the locale.
   *
   * @param resource - The resource/component to add.
   */
  async add(resource: FluentResource | FluentComponent) {
    if (resource instanceof FluentResource) {
      const errors = this.bundle.addResource(resource)
      errors.forEach(err => console.error(err))
    } else {
      if (this.loadedComponents.has(resource)) return

      const supported = resource.which(this.supported)

      if (!supported) {
        console.warn(`Locale ${this.locale} isn't supported by ${resource.component}`)
        return
      }

      if (supported !== this.locale) {
        console.warn(`Fellback to locale ${supported} for ${resource.component}`)
      }

      const supportedResource = await resource.load(supported)

      const errors = this.bundle.addResource(supportedResource)
      errors.forEach(err => console.error(err))

      this.loadedComponents.add(resource)
    }
  }

  /**
   * Loads and adds a component by name.
   *
   * @param component - The name of the component to load.
   */
  async load(component: string) {
    // check if we've already loaded this component
    for (const loaded of this.loadedComponents) {
      if (loaded.component === component) return
    }

    if (!LOCALE_COMPONENTS.has(component)) {
      throw new Error(`Unknown component: ${component}`)
    }

    await this.add(LOCALE_COMPONENTS.get(component)!)
  }

  /**
   * Checks if the given message ID is in this locale's bundle.
   *
   * @param id - The ID of the message to check.
   */
  has(id: string) {
    return this.bundle.hasMessage(id)
  }

  /**
   * Formats a message via its ID.
   *
   * @param id - The ID of the message.
   * @param data - Data to pass to the message's pattern when formatting.
   */
  format(id: string, data?: Record<string, FluentVariable>) {
    const message = this.bundle.getMessage(id)
    if (!message || !message.value) throw new Error(`Invalid message ID: ${id}`)
    const errors: Error[] = []
    const result = this.bundle.formatPattern(message.value, data, errors)
    errors.forEach(err => console.error(err))
    return result
  }
}
