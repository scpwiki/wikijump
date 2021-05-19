import { writable } from "svelte/store"

export interface MediaQueryStore {
  reducedMotion: boolean
  colorScheme: "light" | "dark"
  canHover: boolean
}

/**
 * Singleton class for reading pre-made media queries reactively.
 *
 * @see {@link Media}
 */
class MediaQueryHandler {
  store = writable<MediaQueryStore>({} as any)
  subscribe = this.store.subscribe

  private declare _reducedMotion: MediaQueryList
  private declare _colorSchemeLight: MediaQueryList
  private declare _colorSchemeDark: MediaQueryList
  private declare _canHover: MediaQueryList

  constructor() {
    this._reducedMotion = this.addQuery("(prefers-reduced-motion: reduce)")
    this._colorSchemeLight = this.addQuery("(prefers-color-scheme: light)")
    this._colorSchemeDark = this.addQuery("(prefers-color-scheme: dark)")
    this._canHover = this.addQuery("(any-hover: hover), (hover: hover)")
    this.refresh()
  }

  private refresh() {
    this.store.set({
      reducedMotion: this.reducedMotion,
      colorScheme: this.colorScheme,
      canHover: this.canHover
    })
  }

  private addQuery(query: string) {
    const mediaQuery = matchMedia(query)
    mediaQuery.addEventListener("change", () => this.refresh())
    return mediaQuery
  }

  /** True if `(prefers-reduced-motion: reduce)` is matched. */
  get reducedMotion() {
    return this._reducedMotion.matches
  }

  /** Whether `(prefers-color-scheme: ?)` is either "light" or "dark". */
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
