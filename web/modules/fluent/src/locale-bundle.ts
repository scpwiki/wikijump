import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle"
import { readable } from "svelte/store"
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

  /**
   * Every locale supported, with the primary locale being the first index
   * and the fallbacks being the rest.
   */
  declare supported: string[]

  /** A set containing every component that has already been loaded. */
  declare loadedComponents: Set<FluentComponent>

  /** @param locales - Locale strings. First locale is the primary one. */
  constructor(...locales: string[]) {
    this.locale = locales[0]
    this.fallbacks = locales.slice(1)
    this.supported = [...locales]
    this.loadedComponents = new Set()
    this.bundle = new FluentBundle(this.locale)
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
   * Loads a component, but synchronously returns an observable which
   * resolves to the locale's formatting function. When the component
   * loads, the store will be updated with a new (but otherwise identical)
   * function. This is useful for components that need to be loaded
   * asynchronously, but UI need sto be formatted immediately.
   *
   * If the UI is reactive, the observable update will cause the component
   * to rerender translation strings.
   *
   * @param component - The name of the component to load.
   */
  loadWithObservableFormatter(component: string) {
    // TODO: potentially prefix with component name as syntax sugar
    // check if we've already loaded this component
    // if we have, just return a store and don't bother loading anything
    for (const loaded of this.loadedComponents) {
      if (loaded.component === component) readable(this.format.bind(this))
    }

    console.log(this)

    // wrapped function that uses a fallback "Loading..." string
    const fallback = (id: string, data?: Record<string, FluentVariable>) => {
      return this.format(id, data, "message-loading")
    }

    return readable(fallback, set => {
      this.load(component).then(() => set(this.format.bind(this)))
    })
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
   * @param fallback - A fallback message to use if the message isn't
   *   found. If this is given, a warning won't be shown for missing
   *   messages. This is useful for components that are loaded asynchronously.
   */
  format(id: string, data?: Record<string, FluentVariable>, fallback?: string): string {
    // attributes have to be accessed via dot notation via our string,
    // but Fluent doesn't handle that for you automatically, so we have to do it ourselves

    let resolved = id
    let attribute: null | string = null

    // undefined behavior with more than one dot,
    // but that's not supported in Fluent anyway
    if (id.indexOf(".") !== -1) {
      const split = id.split(".")
      resolved = split[0]
      attribute = split[1]
    }

    const message = this.bundle.getMessage(resolved)

    // the pattern is either the value, or an attribute of the value,
    // depending on what we're looking for
    const value = message
      ? attribute
        ? message.attributes[attribute]
        : message.value
      : null

    if (!value) {
      if (fallback) return this.format(fallback)
      // no fallback, so warn
      console.warn("Missing message:", id)
      return id
    }

    const errors: Error[] = []
    const result = this.bundle.formatPattern(value, data, errors)
    errors.forEach(err => console.error(err))
    return result
  }

  /**
   * Formats a number.
   *
   * @param n - The number to format.
   * @param opts - Options for formatting.
   */
  number(n: number, opts?: Intl.NumberFormatOptions) {
    const formatter = new Intl.NumberFormat(this.supported, opts)
    return formatter.format(n)
  }

  /**
   * Formats a number as a unit, e.g. `20mm`.
   *
   * @param n - The number to format.
   * @param unit - The unit to use.
   * @param opts - Options for formatting.
   */
  unit(n: number, unit: UnitString, opts?: UnitFormatOptions) {
    const formatter = new Intl.NumberFormat(this.supported, {
      style: "unit",
      unit,
      ...opts
    })
    return formatter.format(n)
  }
}

export interface UnitFormatOptions {
  compactDisplay?: "short" | "long"
  notation?: "standard" | "scientific" | "engineering" | "compact"
  signDisplay?: "auto" | "never" | "always"
  unitDisplay?: "short" | "long" | "narrow"
  useGrouping?: boolean
}

// Sourced from:
// https://tc39.es/proposal-unified-intl-numberformat/section6/locales-currencies-tz_proposed_out.html#sec-issanctionedsimpleunitidentifier
/** All valid `Intl` units. */
export type Units =
  | "acre"
  | "bit"
  | "byte"
  | "celsius"
  | "centimeter"
  | "day"
  | "degree"
  | "fahrenheit"
  | "fluid-ounce"
  | "foot"
  | "gallon"
  | "gigabit"
  | "gigabyte"
  | "gram"
  | "hectare"
  | "hour"
  | "inch"
  | "kilobit"
  | "kilobyte"
  | "kilogram"
  | "kilometer"
  | "liter"
  | "megabit"
  | "megabyte"
  | "meter"
  | "mile"
  | "mile-scandinavian"
  | "milliliter"
  | "millimeter"
  | "millisecond"
  | "minute"
  | "month"
  | "ounce"
  | "percent"
  | "petabyte"
  | "pound"
  | "second"
  | "stone"
  | "terabit"
  | "terabyte"
  | "week"
  | "yard"
  | "year"

/** A valid `Intl` unit string, e.g. "kilobyte" or "kilobyte-per-minute". */
export type UnitString = Units | `${Units}-per-${Units}`
