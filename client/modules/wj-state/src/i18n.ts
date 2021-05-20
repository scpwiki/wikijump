// organize-imports-ignore
import * as i18n from "svelte-i18n"
import type { MessageFormatter } from "svelte-i18n/types/runtime/types"
import type { Readable } from "svelte/store"

// -- INIT

// we use `en` as a fallback language, so rather than dynamically importing it
// we will bundle it with the JS. this may be inefficient but it means that
// a user with a spotty internet connection won't see every single string as garbage.
// this should also prevent some edge case errors, hopefully
import langEN from "../../../../locales/en.yaml"
i18n.addMessages("en", langEN as any)

// register other languages here
// example: i18n.register("en", () => import("../../../../locales/en.yaml"))

i18n.init({
  fallbackLocale: "en",
  // TODO: remember language preferences and use those instead
  initialLocale: i18n.getLocaleFromNavigator()
})

// -- RE-EXPORTS

// observables
export { date, json, number, t, time } from "svelte-i18n"

// low-level formatters
export {
  getDateFormatter,
  getMessageFormatter,
  getTimeFormatter,
  getNumberFormatter
} from "svelte-i18n"

// library itself in-case the above exports aren't enough
export { i18n }

// -- EXTENSIONS/HELPERS

/** Reference to the `svelte-i18n`'s most recently created formatting function. */
export let format!: MessageFormatter
// subscribe to the `format` observable so we can update our mutable function
// this is a bit wacky, but it beats having to do this yourself every time
i18n.format.subscribe(formatter => (format = formatter))

/**
 * Formats a string of ICU syntax using the current locale. Using this
 * function is only recommended if the message you are formatting is
 * machine-translatable, such as dates, lists, and numbers.
 *
 * @example
 *
 * ```ts
 * // "20ms"
 * const message = formatMessage(
 *   "{perf, number, :: ,_ unit/millisecond unit-width-narrow }",
 *   { perf: 20 }
 * )
 * ```
 */
export function formatMessage(
  message: string,
  values?: Record<string, string | number | boolean | Date>
) {
  const formatter = i18n.getMessageFormatter(message)
  return String(formatter.format(values))
}

export interface UnitFormatOptions {
  compactDisplay?: string
  notation?: string
  signDisplay?: string
  unitDisplay?: string
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

// this mess is just kinda how you derive an observable like this
// there is probably an easier or maybe automatic way to do this

type UnitFn = (d: number, unit: UnitString, opts?: UnitFormatOptions) => string

export const unit: Readable<UnitFn> = {
  subscribe: sub =>
    i18n.number.subscribe(format =>
      sub((d, unit, opts) => format(d, { style: "unit", unit, ...opts }))
    )
}
