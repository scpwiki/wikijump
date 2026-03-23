const SECONDS_PER_MINUTE = 60
const SECONDS_PER_HOUR = 3_600
const SECONDS_PER_DAY = 86_400
const MILLISECONDS_PER_SECOND = 1_000
const NANOSECONDS_PER_MILLISECOND = 1_000_000
const MILLISECONDS_PER_DAY = 86_400_000

type IcuModule = typeof import("icu")
type IcuLocale = ReturnType<IcuModule["Locale"]["fromString"]>

type ResolvedLocale = {
  icu: IcuModule
  name: string
  locale: IcuLocale
  localeH12: IcuLocale
}

type PaddingModifier = "default" | "space" | "none" | "zero"
type RelativeTimeUnit = "second" | "minute" | "hour" | "day"

let icuPromise: Promise<IcuModule> | null = null

const localeCache = new Map<string, ResolvedLocale>()
const icuFormatterCache = new Map<string, unknown>()
const intlDateTimeFormatterCache = new Map<string, Intl.DateTimeFormat>()
const intlRelativeTimeFormatterCache = new Map<string, Intl.RelativeTimeFormat>()

function normalizeDisplay(value: string) {
  return value.replace(/[\u00A0\u202F]/g, " ")
}

function getIcu() {
  if (!icuPromise) {
    icuPromise = import("icu")
  }

  return icuPromise
}

function getLocaleCandidates(element: HTMLElement) {
  const inheritedLanguage = element.closest("[lang]")?.getAttribute("lang")

  return [
    ...new Set(
      [
        element.lang,
        inheritedLanguage,
        ...navigator.languages,
        navigator.language,
        document.documentElement.lang,
        "en"
      ].filter((value): value is string => Boolean(value))
    )
  ]
}

function getCachedValue<T>(
  cache: Map<string, T> | Map<string, unknown>,
  key: string,
  create: () => T
) {
  const cachedValue = cache.get(key)
  if (cachedValue !== undefined) return cachedValue as T

  const value = create()
  cache.set(key, value)
  return value
}

async function resolveLocale(element: HTMLElement) {
  const icu = await getIcu()

  for (const candidate of getLocaleCandidates(element)) {
    const cachedLocale = localeCache.get(candidate)
    if (cachedLocale) return cachedLocale

    try {
      const locale = icu.Locale.fromString(candidate)
      const localeH12 = locale.clone()
      localeH12.setUnicodeExtension("hc", "h12")

      const resolvedLocale = {
        icu,
        name: locale.toString(),
        locale,
        localeH12
      }

      localeCache.set(candidate, resolvedLocale)
      return resolvedLocale
    } catch {
      continue
    }
  }

  throw new Error("No ICU4X locale available")
}

function toIcuIsoDate(icu: IcuModule, date: Date) {
  return new icu.IsoDate(date.getFullYear(), date.getMonth() + 1, date.getDate())
}

function toIcuTime(icu: IcuModule, date: Date) {
  return new icu.Time(
    date.getHours(),
    date.getMinutes(),
    date.getSeconds(),
    date.getMilliseconds() * NANOSECONDS_PER_MILLISECOND
  )
}

function formatLocalizedWeekday(
  date: Date,
  locale: ResolvedLocale,
  abbreviated: boolean
) {
  const formatter = getCachedValue(
    icuFormatterCache,
    `${locale.name}:weekday:${abbreviated ? "short" : "long"}`,
    () =>
      locale.icu.DateFormatter.createE(
        locale.locale,
        abbreviated ? locale.icu.DateTimeLength.Short : locale.icu.DateTimeLength.Long
      )
  )

  return normalizeDisplay(formatter.formatIso(toIcuIsoDate(locale.icu, date)))
}

function formatLocalizedMonth(date: Date, locale: ResolvedLocale, abbreviated: boolean) {
  const formatter = getCachedValue(
    icuFormatterCache,
    `${locale.name}:month:${abbreviated ? "medium" : "long"}`,
    () =>
      locale.icu.DateFormatter.createM(
        locale.locale,
        abbreviated ? locale.icu.DateTimeLength.Medium : locale.icu.DateTimeLength.Long,
        locale.icu.DateTimeAlignment.Auto
      )
  )

  return normalizeDisplay(formatter.formatIso(toIcuIsoDate(locale.icu, date)))
}

function formatLocalizedDate(date: Date, locale: ResolvedLocale) {
  const formatter = getCachedValue(icuFormatterCache, `${locale.name}:date:short`, () =>
    locale.icu.DateFormatter.createYmd(
      locale.locale,
      locale.icu.DateTimeLength.Short,
      locale.icu.DateTimeAlignment.Auto,
      locale.icu.YearStyle.Auto
    )
  )

  return normalizeDisplay(formatter.formatIso(toIcuIsoDate(locale.icu, date)))
}

function formatLocalizedDateTimeFullYear(date: Date, locale: ResolvedLocale) {
  const formatter = getCachedValue(
    icuFormatterCache,
    `${locale.name}:datetime:full-year`,
    () =>
      locale.icu.DateTimeFormatter.createYmdt(
        locale.locale,
        locale.icu.DateTimeLength.Short,
        locale.icu.TimePrecision.Second,
        locale.icu.DateTimeAlignment.Auto,
        locale.icu.YearStyle.Full
      )
  )

  return normalizeDisplay(
    formatter.formatIso(toIcuIsoDate(locale.icu, date), toIcuTime(locale.icu, date))
  )
}

function formatLocalizedTime(date: Date, locale: ResolvedLocale) {
  const formatter = getCachedValue(
    icuFormatterCache,
    `${locale.name}:time:medium`,
    () =>
      new locale.icu.TimeFormatter(
        locale.locale,
        locale.icu.DateTimeLength.Medium,
        locale.icu.TimePrecision.Second,
        locale.icu.DateTimeAlignment.Auto
      )
  )

  return normalizeDisplay(formatter.format(toIcuTime(locale.icu, date)))
}

function formatLocalizedTimeH12(date: Date, locale: ResolvedLocale) {
  const formatter = getCachedValue(
    icuFormatterCache,
    `${locale.name}:time:h12`,
    () =>
      new locale.icu.TimeFormatter(
        locale.localeH12,
        locale.icu.DateTimeLength.Medium,
        locale.icu.TimePrecision.Second,
        locale.icu.DateTimeAlignment.Auto
      )
  )

  return normalizeDisplay(formatter.format(toIcuTime(locale.icu, date)))
}

function formatLocalizedDayPeriod(
  date: Date,
  locale: ResolvedLocale,
  uppercase: boolean
) {
  const formatter = getCachedValue(
    intlDateTimeFormatterCache,
    `${locale.name}:day-period`,
    () =>
      new Intl.DateTimeFormat(locale.name, {
        hour: "numeric",
        hour12: true
      })
  )

  const dayPeriod = formatter
    .formatToParts(date)
    .find((part) => part.type === "dayPeriod")
  const value = normalizeDisplay(dayPeriod?.value ?? "")

  return uppercase ? value : normalizeDisplay(value.toLocaleLowerCase(locale.name))
}

function getLocalDateUtcValue(date: Date) {
  return Date.UTC(date.getFullYear(), date.getMonth(), date.getDate())
}

function getDayOfYear(date: Date) {
  return (
    Math.floor(
      (getLocalDateUtcValue(date) - Date.UTC(date.getFullYear(), 0, 1)) /
        MILLISECONDS_PER_DAY
    ) + 1
  )
}

function getIsoWeekday(date: Date) {
  const weekday = date.getDay()
  return weekday === 0 ? 7 : weekday
}

function getWeekNumberSunday(date: Date) {
  const dayIndex = getDayOfYear(date) - 1
  const firstDay = new Date(date.getFullYear(), 0, 1).getDay()
  const firstSundayOffset = (7 - firstDay) % 7

  if (dayIndex < firstSundayOffset) return 0
  return Math.floor((dayIndex - firstSundayOffset) / 7) + 1
}

function getWeekNumberMonday(date: Date) {
  const dayIndex = getDayOfYear(date) - 1
  const firstDay = new Date(date.getFullYear(), 0, 1).getDay()
  const firstMondayOffset = (8 - (firstDay === 0 ? 7 : firstDay)) % 7

  if (dayIndex < firstMondayOffset) return 0
  return Math.floor((dayIndex - firstMondayOffset) / 7) + 1
}

function getIsoWeekInfo(date: Date) {
  const weekday = getIsoWeekday(date)
  const thursday = new Date(date)
  thursday.setDate(date.getDate() + 4 - weekday)

  const weekYear = thursday.getFullYear()
  const firstThursday = new Date(weekYear, 0, 4)
  const firstThursdayWeekday = getIsoWeekday(firstThursday)
  firstThursday.setDate(firstThursday.getDate() + 4 - firstThursdayWeekday)

  const week =
    Math.round(
      (getLocalDateUtcValue(thursday) - getLocalDateUtcValue(firstThursday)) /
        (7 * MILLISECONDS_PER_DAY)
    ) + 1

  return {
    year: weekYear,
    week
  }
}

function applyPadding(
  rawValue: number | string,
  width: number,
  modifier: PaddingModifier,
  defaultFill: "0" | " "
) {
  const value = String(rawValue)
  const fill =
    modifier === "none"
      ? null
      : modifier === "space"
        ? " "
        : modifier === "zero"
          ? "0"
          : defaultFill

  if (!fill) return value
  return value.padStart(width, fill)
}

function formatYear(year: number, modifier: PaddingModifier) {
  return applyPadding(year, 4, modifier, "0")
}

function formatHour12(date: Date, modifier: PaddingModifier, defaultFill: "0" | " ") {
  const hour = date.getHours() % 12 || 12
  return applyPadding(hour, 2, modifier, defaultFill)
}

function formatTimezoneGmt(date: Date) {
  const totalMinutes = -date.getTimezoneOffset()
  const sign = totalMinutes < 0 ? "-" : "+"
  const absoluteMinutes = Math.abs(totalMinutes)
  const hours = Math.floor(absoluteMinutes / 60)
  const minutes = absoluteMinutes % 60

  if (minutes === 0) {
    return `GMT${sign}${String(hours).padStart(2, "0")}`
  }

  return `GMT${sign}${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`
}

function getRelativeTimeValue(date: Date) {
  const dateSeconds = Math.trunc(date.getTime() / MILLISECONDS_PER_SECOND)
  const nowSeconds = Math.trunc(Date.now() / MILLISECONDS_PER_SECOND)
  const deltaSeconds = dateSeconds - nowSeconds
  const absoluteDelta = Math.abs(deltaSeconds)

  if (absoluteDelta < SECONDS_PER_MINUTE) {
    return {
      value: deltaSeconds,
      unit: "second" as RelativeTimeUnit,
      numeric: "always" as const
    }
  }

  if (absoluteDelta < SECONDS_PER_HOUR) {
    return {
      value: Math.trunc(deltaSeconds / SECONDS_PER_MINUTE),
      unit: "minute" as RelativeTimeUnit,
      numeric: "always" as const
    }
  }

  if (absoluteDelta < SECONDS_PER_DAY) {
    return {
      value: Math.trunc(deltaSeconds / SECONDS_PER_HOUR),
      unit: "hour" as RelativeTimeUnit,
      numeric: "always" as const
    }
  }

  return {
    value: Math.trunc(deltaSeconds / SECONDS_PER_DAY),
    unit: "day" as RelativeTimeUnit,
    numeric: "auto" as const
  }
}

function formatRelativeTime(date: Date, locale: ResolvedLocale) {
  const { value, unit, numeric } = getRelativeTimeValue(date)
  const formatter = getCachedValue(
    intlRelativeTimeFormatterCache,
    `${locale.name}:relative:${unit}:${numeric}`,
    () =>
      new Intl.RelativeTimeFormat(locale.name, {
        numeric,
        style: "long"
      })
  )

  return normalizeDisplay(formatter.format(value, unit))
}

function invalidFormatError(format: string, message: string) {
  return new Error(`invalid strftime format string '${format}': ${message}`)
}

function formatUnlocalizedDirective(
  date: Date,
  directive: string,
  modifier: PaddingModifier,
  format: string
) {
  const year = date.getFullYear()
  const month = date.getMonth() + 1
  const day = date.getDate()
  const hours = date.getHours()
  const minutes = date.getMinutes()
  const seconds = date.getSeconds()
  const isoWeekInfo = getIsoWeekInfo(date)

  switch (directive) {
    case "%":
      return "%"
    case "C":
      return applyPadding(Math.trunc(year / 100), 2, modifier, "0")
    case "d":
      return applyPadding(day, 2, modifier, "0")
    case "D":
      return `${applyPadding(month, 2, "default", "0")}/${applyPadding(day, 2, "default", "0")}/${applyPadding(year % 100, 2, "default", "0")}`
    case "e":
      return applyPadding(day, 2, modifier === "default" ? "space" : modifier, " ")
    case "F":
      return `${formatYear(year, "default")}-${applyPadding(month, 2, "default", "0")}-${applyPadding(day, 2, "default", "0")}`
    case "g":
      return applyPadding(isoWeekInfo.year % 100, 2, modifier, "0")
    case "G":
      return formatYear(isoWeekInfo.year, modifier)
    case "H":
      return applyPadding(hours, 2, modifier, "0")
    case "I":
      return formatHour12(date, modifier, "0")
    case "j":
      return applyPadding(getDayOfYear(date), 3, modifier, "0")
    case "k":
      return applyPadding(hours, 2, modifier === "default" ? "space" : modifier, " ")
    case "l":
      return formatHour12(date, modifier === "default" ? "space" : modifier, " ")
    case "m":
      return applyPadding(month, 2, modifier, "0")
    case "M":
      return applyPadding(minutes, 2, modifier, "0")
    case "n":
      return "\n"
    case "R":
      return `${applyPadding(hours, 2, "default", "0")}:${applyPadding(minutes, 2, "default", "0")}`
    case "S":
      return applyPadding(seconds, 2, modifier, "0")
    case "T":
      return `${applyPadding(hours, 2, "default", "0")}:${applyPadding(minutes, 2, "default", "0")}:${applyPadding(seconds, 2, "default", "0")}`
    case "t":
      return "\t"
    case "u":
      return String(getIsoWeekday(date))
    case "w":
      return String(date.getDay())
    case "U":
      return applyPadding(getWeekNumberSunday(date), 2, modifier, "0")
    case "V":
      return applyPadding(isoWeekInfo.week, 2, modifier, "0")
    case "W":
      return applyPadding(getWeekNumberMonday(date), 2, modifier, "0")
    case "y":
      return applyPadding(year % 100, 2, modifier, "0")
    case "Y":
      return formatYear(year, modifier)
    default:
      throw invalidFormatError(format, `unsupported directive %${directive}`)
  }
}

function formatDirective(
  date: Date,
  locale: ResolvedLocale,
  directive: string,
  modifier: PaddingModifier,
  format: string
) {
  switch (directive) {
    case "%":
      return "%"
    case "a":
      return formatLocalizedWeekday(date, locale, true)
    case "A":
      return formatLocalizedWeekday(date, locale, false)
    case "b":
      return formatLocalizedMonth(date, locale, true)
    case "B":
      return formatLocalizedMonth(date, locale, false)
    case "c":
      return formatLocalizedDateTimeFullYear(date, locale)
    case "O":
      return formatRelativeTime(date, locale)
    case "p":
      return formatLocalizedDayPeriod(date, locale, true)
    case "P":
      return formatLocalizedDayPeriod(date, locale, false)
    case "r":
      return formatLocalizedTimeH12(date, locale)
    case "X":
      return formatLocalizedTime(date, locale)
    case "x":
      return formatLocalizedDate(date, locale)
    case "Z":
    case "z":
      return formatTimezoneGmt(date)
    default:
      return formatUnlocalizedDirective(date, directive, modifier, format)
  }
}

function formatStrftime(date: Date, format: string, locale: ResolvedLocale) {
  let rendered = ""
  let literalStart = 0

  for (let index = 0; index < format.length; index += 1) {
    if (format[index] !== "%") continue

    rendered += format.slice(literalStart, index)

    index += 1
    if (index >= format.length) {
      throw invalidFormatError(format, "unexpected end of input after '%'")
    }

    let modifier: PaddingModifier = "default"
    if (format[index] === "_" || format[index] === "-" || format[index] === "0") {
      modifier = format[index] === "_" ? "space" : format[index] === "-" ? "none" : "zero"
      index += 1
    }

    if (index >= format.length) {
      throw invalidFormatError(format, "unexpected end of input after padding modifier")
    }

    rendered += formatDirective(date, locale, format[index], modifier, format)
    literalStart = index + 1
  }

  rendered += format.slice(literalStart)
  return rendered
}

function formatDateOrDefault(date: Date, format: string | null, locale: ResolvedLocale) {
  if (!format) {
    return formatLocalizedDateTimeFullYear(date, locale)
  }

  try {
    return formatStrftime(date, format, locale)
  } catch {
    return formatLocalizedDateTimeFullYear(date, locale)
  }
}

function parseTimestampSeconds(value: string) {
  const timestamp = Number(value)
  if (!Number.isFinite(timestamp)) return null

  const date = new Date(timestamp * MILLISECONDS_PER_SECOND)
  if (Number.isNaN(date.getTime())) return null

  return date
}

function readTimestampSeconds(element: HTMLElement) {
  const datasetTimestamp = element.dataset.timestamp
  if (datasetTimestamp) return datasetTimestamp

  const timestampClass = Array.from(element.classList).find((className) =>
    className.startsWith("time_")
  )

  if (!timestampClass) return null
  return timestampClass.slice("time_".length)
}

function readFormatClass(element: HTMLElement) {
  const formatClass = Array.from(element.classList).find((className) =>
    className.startsWith("format_")
  )

  if (!formatClass) return null

  try {
    return decodeURIComponent(formatClass.slice("format_".length))
  } catch {
    return null
  }
}

async function formatFtmlDateElement(element: HTMLElement) {
  const timestamp = readTimestampSeconds(element)
  if (!timestamp) return

  const date = parseTimestampSeconds(timestamp)
  if (!date) return

  const locale = await resolveLocale(element)
  element.textContent = formatDateOrDefault(date, readFormatClass(element), locale)
}

export async function runDateFormatter() {
  const ftmlDateElements = Array.from(
    document.querySelectorAll<HTMLElement>(".wj-date[data-timestamp], .odate")
  )

  await Promise.all(
    ftmlDateElements.map(async (element) => {
      try {
        await formatFtmlDateElement(element)
      } catch {
        return
      }
    })
  )
}
