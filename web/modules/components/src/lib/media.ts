import { Readable, writable } from "svelte/store"

const OPERATORS = [">", "<", ">=", "<=", "=="] as const

const BREAKPOINTS = [
  ["tiny", 350],
  ["narrow", 500],
  ["small", 850],
  ["normal", 1000],
  ["wide", 1400]
] as const

export type Operator = typeof OPERATORS[number]
export type BreakpointName = typeof BREAKPOINTS[number][0]
export type BreakpointString = `${Operator | ""}${BreakpointName}`

export interface MediaQueryStore {
  reducedMotion: boolean
  colorScheme: "light" | "dark"
  canHover: boolean
  breakpoint: BreakpointName
  orientation: "landscape" | "portrait"
}

class BreakpointMapping {
  private queries: Map<BreakpointName, MediaQueryList> = new Map()
  private map: Map<BreakpointName | number, BreakpointName | number> = new Map()

  constructor(callback: () => void) {
    let idx = 0
    for (const [name, pixels] of BREAKPOINTS) {
      const query = matchMedia(`(min-width: ${pixels}px)`)
      query.addEventListener("change", callback)
      this.queries.set(name, query)
      this.map.set(idx, name)
      this.map.set(name, idx)
      idx++
    }
  }

  get(id: number): BreakpointName
  get(name: BreakpointName): number
  get(key: BreakpointName | number): BreakpointName | number {
    return this.map.get(key)!
  }

  has(name: string): name is BreakpointName {
    return this.map.has(name as any)
  }

  active() {
    let current: BreakpointName = "tiny"
    for (const [name, query] of this.queries) {
      if (query.matches) current = name
    }
    return current
  }
}

/**
 * Singleton class for reading pre-made media queries reactively.
 *
 * @see {@link Media}
 */
class MediaQueryHandler {
  store = writable<MediaQueryStore>({} as any)
  subscribe = this.store.subscribe

  private _reducedMotion = this.addQuery("(prefers-reduced-motion: reduce)")
  private _colorSchemeLight = this.addQuery("(prefers-color-scheme: light)")
  private _colorSchemeDark = this.addQuery("(prefers-color-scheme: dark)")
  private _canHover = this.addQuery("(any-hover: hover), (hover: hover)")
  private _orientationLandscape = this.addQuery("(orientation: landscape)")
  private _orientationPortrait = this.addQuery("(orientation: portrait)")

  private breakpoints = new BreakpointMapping(() => this.refresh())

  constructor() {
    this.refresh()
  }

  /** True if `(prefers-reduced-motion: reduce)` is matched. */
  get reducedMotion() {
    return this._reducedMotion.matches
  }

  /**
   * Whether the query `(prefers-color-scheme: ?)` matches "light" or
   * "dark". Defaults to "light" if this can't be determined.
   */
  get colorScheme(): "light" | "dark" {
    return this._colorSchemeLight.matches
      ? "light"
      : this._colorSchemeDark.matches
      ? "dark"
      : "light"
  }

  /** True if `(any-hover: hover), (hover: hover)` is matched. */
  get canHover() {
    return this._canHover.matches
  }

  /** The currently active breakpoint. */
  get breakpoint() {
    return this.breakpoints.active()
  }

  /**
   * The current device orientation. Defaults to "landscape" if for some
   * reason it can't be determined.
   */
  get orientation(): "landscape" | "portrait" {
    return this._orientationLandscape.matches
      ? "landscape"
      : this._orientationPortrait.matches
      ? "portrait"
      : "landscape"
  }

  private addQuery(query: string) {
    const mediaQuery = matchMedia(query)
    mediaQuery.addEventListener("change", () => this.refresh())
    return mediaQuery
  }

  private refresh() {
    this.store.set({
      reducedMotion: this.reducedMotion,
      colorScheme: this.colorScheme,
      canHover: this.canHover,
      breakpoint: this.breakpoint,
      orientation: this.orientation
    })
  }

  /** Tests if a query passes or not. */
  test(query: string) {
    const mediaQuery = matchMedia(query)
    return mediaQuery.matches
  }

  /**
   * Evaluates a `include-media`-compatible breakpoint string and returns
   * if it matches. Throws with incompatible queries.
   *
   * You can use this function directly - but if needed,
   * {@link matchBreakpoint | you can use it reactively}.
   *
   * @see {@link matchBreakpoint}
   */
  matchBreakpoint(query: BreakpointString) {
    // figure out what operator to use

    let operator: Operator = "=="

    const match = /^[=<>]+/.exec(query)

    if (match && OPERATORS.includes(match[0] as any)) {
      operator = match[0] as Operator
    } else if (match) {
      throw new Error(`Bad operator (${match[0]}) given in breakpoint string!`)
    }

    // strip off operator to get breakpoint name
    const breakpoint = query.replace(/^[=<>]+/, "")

    if (!this.breakpoints.has(breakpoint)) {
      throw new Error(`Bad breakpoint (${breakpoint}) given in breakpoint string!`)
    }

    const current = this.breakpoints.active()
    const curIdx = this.breakpoints.get(current)
    const brkIdx = this.breakpoints.get(breakpoint)

    // prettier-ignore
    switch (operator) {
      case "==": return curIdx === brkIdx
      case ">":  return curIdx >   brkIdx
      case "<":  return curIdx <   brkIdx
      case ">=": return curIdx >=  brkIdx
      case "<=": return curIdx <=  brkIdx
      default: throw new Error(`Query (${query}) was somehow impossible to evaluate!`)
    }
  }
}

/**
 * Handler and helper singleton for reading media queries directly or
 * reactively. Queries can be read reactively because this object fulfills
 * the observable protocol.
 *
 * @example
 *
 * ```svelte
 * <!--
 *   Sets the class of the element between "dark" and "light", even when the user
 *   changes their preferred color scheme settings.
 * -->
 * <div class={$Media.colorScheme} />
 * ```
 */
export const Media = new MediaQueryHandler()

const matchBreakpointStore = writable<MediaQueryHandler["matchBreakpoint"]>(query =>
  Media.matchBreakpoint(query)
)

// update with new function reference whenever the breakpoint changes
let lastBreakpoint: BreakpointName
Media.subscribe(store => {
  const current = store.breakpoint
  if (lastBreakpoint !== current) {
    matchBreakpointStore.set(query => Media.matchBreakpoint(query))
  }
  lastBreakpoint = current
})

/**
 * Store that allows reactively listening for breakpoint changes, while
 * still maintaining the ability to use breakpoint queries/matchers.
 *
 * ```text
 * operators:
 *   "==" # default if none given
 *   "<"
 *   "<="
 *   ">"
 *   ">="
 *
 * breakpoints:
 *   "tiny"    # exceptionally narrow phones
 *   "narrow"  # approx. phones
 *   "small"   # approx. tablets, landscape phones, laptops
 *   "normal"  # approx. desktops
 *   "wide"    # approx. extra wide monitors
 * ```
 *
 * @example
 *
 * ```svelte
 * <!-- works even if the screen size changes -->
 * {#if $matchBreakpoint("<=small")}
 *   <span>Displays only on small devices</span>
 * {/if}
 * ```
 */
export const matchBreakpoint: Readable<MediaQueryHandler["matchBreakpoint"]> = {
  subscribe: matchBreakpointStore.subscribe
}
