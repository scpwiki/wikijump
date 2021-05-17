import { writable } from "svelte/store"

export interface MediaQueryStore {
  reducedMotion: boolean
  colorScheme: "light" | "dark"
  canHover: boolean
}

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

  get reducedMotion() {
    return this._reducedMotion.matches
  }

  get colorScheme(): "light" | "dark" {
    return this._colorSchemeLight.matches
      ? "light"
      : this._colorSchemeDark.matches
      ? "dark"
      : "light"
  }

  get canHover() {
    return this._canHover.matches
  }
}

export const Media = new MediaQueryHandler()
