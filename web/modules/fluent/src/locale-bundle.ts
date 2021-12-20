import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle"
import { type Pattern } from "@fluent/bundle/esm/ast"
import { readable } from "svelte/store"
import { FluentComponent } from "./component"
import { LOCALE_COMPONENTS } from "./locales"

/** Class that handles translations/formatting for a specific locale. */
export class Locale {
  /** Internal bundle that stores and formats messages. */
  private declare bundle: FluentBundle

  /** Components that are still loading. */
  private declare pending: Map<FluentComponent, Promise<any>>

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
    this.pending = new Map()
    this.bundle = new FluentBundle(this.locale)
  }

  /**
   * Parses a message selector made via dot notation, e.g. `foo.attr`.
   *
   * @param selector - The selector to parse.
   */
  static parseSelector(selector: string): [message: string, attribute: null | string] {
    let id = selector
    let attribute: null | string = null

    // undefined behavior with more than one dot,
    // but that's not supported in Fluent anyway
    if (selector.indexOf(".") !== -1) {
      const split = selector.split(".")
      id = split[0]
      attribute = split[1]
    }

    return [id, attribute]
  }

  /**
   * Gets the pattern for the given selector.
   *
   * @param selector - The selector to get the pattern for.
   * @param fallback - The fallback pattern to use if the selector isn't found.
   */
  private getPattern(selector: string, fallback?: string): Pattern | null {
    const [id, attribute] = Locale.parseSelector(selector)
    const message = this.bundle.getMessage(id)

    const pattern = message
      ? attribute
        ? message.attributes[attribute] ?? null
        : message.value
      : null

    if (!pattern && fallback) {
      return this.getPattern(fallback)
    }

    return pattern
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
      if (this.loadedComponents.has(resource)) {
        // wait on the pending load, rather than just returning sync.
        if (this.pending.has(resource)) await this.pending.get(resource)!
        return
      }

      const supported = resource.which(this.supported)

      if (!supported) {
        console.warn(`Locale ${this.locale} isn't supported by ${resource.component}`)
        return
      }

      if (supported !== this.locale) {
        console.warn(`Fellback to locale ${supported} for ${resource.component}`)
      }

      this.loadedComponents.add(resource)

      const loading = resource.load(supported)

      this.pending.set(resource, loading)

      const errors = this.bundle.addResource(await loading)
      if (errors.length) errors.forEach(err => console.error(err))

      this.pending.delete(resource)
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
      if (loaded.component === component) {
        // wait on the pending load, rather than just returning sync.
        if (this.pending.has(loaded)) await this.pending.get(loaded)!
        return
      }
    }

    if (!LOCALE_COMPONENTS.has(component)) {
      throw new Error(`Unknown component: ${component}`)
    }

    await this.add(LOCALE_COMPONENTS.get(component)!)
  }

  /**
   * Makes a formatting observable for the given component. If the
   * component isn't loaded, it will be loaded automatically, and the
   * observable will be updated with a new formatting function. This allows
   * a UI to be formatted immediately, but still be reactive.
   *
   * The formatter returned by this function also has a special syntax
   * sugar, which allows for a beginning `"#"` character to be substituted
   * with the given component name.
   *
   * While the component is loading, the observable will use a fallback
   * string for any unknown messages.
   *
   * @param component - The name of the component to make a formatter for.
   */
  makeComponentFormatter(component: string) {
    // generates our component formatting function, with syntax sugar and
    // handling "Loading..." message fallback
    const makeFunction = (fallback: boolean) => (selector: string, data?: FluentData) => {
      // syntax sugar shorthand
      if (selector[0] === "#") selector = component + selector.slice(1)
      if (fallback) return this.format(selector, data, "message-loading")
      return this.format(selector, data)
    }

    return readable(makeFunction(true), set => {
      this.load(component).then(() => set(makeFunction(false)))
    })
  }

  /**
   * Checks if the given message ID is in this locale's bundle.
   *
   * @param selector - The ID of the message to check.
   */
  has(selector: string) {
    return this.getPattern(selector) !== null
  }

  /**
   * Formats a message via a selector.
   *
   * @param selector - The selector for getting the message.
   * @param data - Data to pass to the message's pattern when formatting.
   * @param fallback - A fallback message to use if the message isn't
   *   found. If this is given, a warning won't be shown for missing
   *   messages. This is useful for components that are loaded asynchronously.
   */
  format(selector: string, data?: FluentData, fallback?: string): string {
    const pattern = this.getPattern(selector, fallback)

    if (!pattern) {
      console.warn("Missing message:", selector)
      return selector
    }

    const errors: Error[] = []
    const result = this.bundle.formatPattern(pattern, data, errors)
    if (errors.length) errors.forEach(err => console.error(err))
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

export type FluentData = Record<string, FluentVariable>

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
