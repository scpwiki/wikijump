import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle"
import { FluentComponent } from "./component"

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

  /** @param locales - Locale strings. First locale is the primary one. */
  constructor(...locales: string[]) {
    this.locale = locales[0]
    this.fallbacks = locales.slice(1)
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
      const supported = resource.which(this.supported)

      if (!supported) {
        console.warn(`Locale ${this.locale} isn't supported by ${resource.component}`)
        return
      }

      if (supported !== this.locale) {
        console.warn(`Fellback to locale ${this.supported} for ${resource.component}`)
      }

      const supportedResource = await resource.load(supported)

      const errors = this.bundle.addResource(supportedResource)
      errors.forEach(err => console.error(err))
    }
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
