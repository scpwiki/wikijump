/** The location of static assets, from host root. Can't end with a `/`. */
export const STATIC_FOLDER = "/files-common"

/**
 * A record of asset identifiers to their known paths. This allows for
 * referencing an asset without hardcoding a path to it in the codebase.
 */
// prettier-ignore
export const NAMED_ASSETS = {
  "BAD_AVATAR":     "/media/bad-avatar.png",
  "DEFAULT_AVATAR": "/media/default-avatar.png",
  "KARMA_SVG":      "/media/karma.svg"
} as const

/**
 * Represents a path to a {@link Asset}. This is either as a path to a
 * generic asset starting with a `/`, or as an identifier to a known asset.
 *
 * @see {@link NAMED_ASSETS}
 */
export type AssetPath = `/${string}` | keyof typeof NAMED_ASSETS

/**
 * Represents a path to an asset file, along with utility methods for
 * interacting with that asset. Will coerce itself to a `string` in most
 * cases, but TypeScript may be unhappy about this, regardless if it would
 * actually work. Use the `raw` property to get the full, raw path in these cases.
 */
export class Asset {
  /** The full path to the asset. */
  private declare path: AssetPath

  /**
   * @param path - The path to the asset, not including the pathing
   *   required to reach the {@link STATIC_FOLDER}.
   */
  constructor(path: AssetPath) {
    if (!path) throw new Error("Empty asset URL")

    // failsafe against weird calls
    if (path.startsWith(STATIC_FOLDER)) {
      this.path = path
    }
    // special named assets
    else if (path in NAMED_ASSETS) {
      // @ts-ignore - we know that the key exists, TS doesn't for some reason
      this.path = `${STATIC_FOLDER}${NAMED_ASSETS[path]}`
    }
    // generic asset
    else {
      this.path = `${STATIC_FOLDER}${path}`
    }
  }

  /** The full, raw path to the asset. */
  get raw() {
    return this.path
  }

  /** The JS URL object for the asset. */
  get url() {
    return new URL(this.path, window.location.hostname)
  }

  /**
   * Returns a `fetch` response for this asset. May return `null` if the
   * `fetch` fails or if the response doesn't have a `2XX` status code.
   *
   * @param init - The `fetch` options.
   */
  async fetch(init?: RequestInit): Promise<Response | null> {
    try {
      const response = await fetch(this.path, init)
      if (!response.ok) return null
      return response
    } catch {
      return null
    }
  }

  /**
   * Fetches the asset as text. May return `null` if the `fetch` fails or
   * if the response doesn't have a `2XX` status code.
   *
   * @param init - The `fetch` options.
   */
  async text(init?: RequestInit): Promise<string | null> {
    try {
      const response = await this.fetch(init)
      if (!response) return null
      return await response.text()
    } catch {
      return null
    }
  }

  /**
   * Fetches the asset as JSON. May return `null` if the `fetch` fails or
   * if the response doesn't have a `2XX` status code.
   *
   * @param init - The `fetch` options.
   */
  async json<T extends JSONObject = any>(init?: RequestInit): Promise<T | null> {
    try {
      const response = await this.fetch(init)
      if (!response) return null
      return await response.json()
    } catch {
      return null
    }
  }

  // -- COERCION
  // values are `protected` so that they don't show up in IDEs as public methods

  protected valueOf() {
    return this.path
  }

  protected toString() {
    return this.path
  }

  protected toJSON() {
    return this.path
  }

  protected [Symbol.toPrimitive]() {
    return this.path
  }

  protected [Symbol.iterator]() {
    return this.path[Symbol.iterator]()
  }

  protected [Symbol.toStringTag]() {
    return this.path
  }
}

/**
 * Returns a {@link Asset} for the given path.
 *
 * @param path - The path to the asset, not including the pathing required
 *   to reach the {@link STATIC_FOLDER}.
 */
export function asset(path: AssetPath) {
  return new Asset(path)
}

export default asset
