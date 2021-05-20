import { Readable, writable } from "svelte/store"

export type BreakpointName = "narrow" | "small" | "normal" | "wide"
export type BreakpointString = `${">" | "<" | ">=" | "<=" | ""}${BreakpointName}`

export interface MediaQueryStore {
  reducedMotion: boolean
  colorScheme: "light" | "dark"
  canHover: boolean
  breakpoint: BreakpointName
  orientation: "landscape" | "portrait"
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

  // TODO: do we want to automate this using some configuration file?
  private breakpointQueries = [
    this.addQuery("(min-width: 400px)"),
    this.addQuery("(min-width: 800px)"),
    this.addQuery("(min-width: 1000px)"),
    this.addQuery("(min-width: 1400px)")
  ] as const
  private breakpointNames = ["narrow", "small", "normal", "wide"] as const
  private breakpointMap = { "narrow": 0, "small": 1, "normal": 2, "wide": 3 } as const

  constructor() {
    this.refresh()
  }

  private addQuery(query: string) {
    const mediaQuery = matchMedia(query)
    mediaQuery.addEventListener("change", () => this.refresh())
    return mediaQuery
  }

  private activeBreakpoint() {
    // setting the type normally causes TS to over-simplify the type to just "narrow"
    // for some reason... an assertion fixes that, apparently
    let current = "narrow" as BreakpointName
    this.breakpointQueries.forEach((query, idx) => {
      if (query.matches) current = this.breakpointNames[idx]
    })
    return current
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
    return this.activeBreakpoint()
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
    type Operator = ">" | "<" | ">=" | "<=" | "=="
    let operator: Operator = "=="
    const match = /^[=<>]+/.exec(query)
    if (match && [">", "<", ">=", "<=", "=="].includes(match[0])) {
      operator = match[0] as Operator
    } else if (match) {
      throw new Error(`Bad operator (${match[0]}) given in breakpoint string!`)
    }

    // strip off operator to get breakpoint name
    const breakpoint = query.replace(/^[=<>]+/, "") as BreakpointName
    if (!this.breakpointNames.includes(breakpoint)) {
      throw new Error(`Bad breakpoint (${breakpoint}) given in breakpoint string!`)
    }

    const current = this.activeBreakpoint()

    const curIdx = this.breakpointMap[current]
    const brkIdx = this.breakpointMap[breakpoint]

    if (operator === "==") return curIdx === brkIdx
    if (operator === "<") return curIdx < brkIdx
    if (operator === "<=") return curIdx <= brkIdx
    if (operator === ">") return curIdx > brkIdx
    if (operator === ">=") return curIdx >= brkIdx

    throw new Error(`Breakpoint query (${query}) was somehow impossible to evaluate!`)
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
 *   "narrow"  # approx. phones
 *   "small"   # approx. tablets
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
