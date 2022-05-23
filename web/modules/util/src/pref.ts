import { writable, type Updater, type Writable } from "svelte/store"

/**
 * `localStorage`-based smart preference handler, with support for
 * observables or proxied objects.
 */
export class PreferenceHandler {
  /**
   * @param prefix - The namespace to use for the preference handler.
   *   Defaults to `"_user-pref_"`
   */
  constructor(private prefix = "_user-pref_") {}

  /**
   * Returns the name given with a prefix namespace, to prevent collisions
   * with other items in `localStorage`.
   */
  private processName(name: string) {
    if (name.startsWith(this.prefix)) return name
    return (name = this.prefix + name)
  }

  /**
   * Attempt to retrieve the preference with the given name. If it isn't
   * found, the fallback value will instead be returned.
   *
   * @param name - The name/key for the preference.
   * @param fallback - The default value to return if the preference doesn't exist.
   */
  get<T = JSONValue>(name: string, fallback: T): T
  get<T = JSONValue>(name: string, fallback?: T): T | undefined
  get<T = JSONValue>(name: string, fallback?: T): T | undefined {
    name = this.processName(name)
    const storedPreference = localStorage.getItem(name)
    if (storedPreference) return JSON.parse(storedPreference) as T
    else return fallback
  }

  /**
   * Sets the preference with the given name to the given value. Passing an
   * empty string will remove the preference from storage.
   *
   * The `value` given **must** be in such a format that it can be
   * stringified by `JSON.stringify`. Otherwise, data _will_ be lost.
   *
   * @param name - The name/key for the preference.
   * @param value - The value to set. An empty string removes the preference.
   */
  set<T = JSONValue>(name: string, value: T) {
    name = this.processName(name)
    if (!value) localStorage.removeItem(name)
    else localStorage.setItem(name, JSON.stringify(value))
    return value
  }

  /**
   * Removes a preference from storage.
   *
   * @param name - The name/key for the preference.
   */
  remove(name: string) {
    name = this.processName(name)
    localStorage.removeItem(name)
  }

  /**
   * Returns if the requested preference is available in storage.
   *
   * @param name - The name/key for the preference.
   */
  has(name: string) {
    name = this.processName(name)
    return Boolean(localStorage.getItem(name))
  }

  /** Returns the list of preferences currently stored. */
  stored() {
    const len = localStorage.length
    const keys = new Set<string>()
    for (let idx = 0; idx < len; idx++) {
      const name = localStorage.key(idx)
      if (name?.startsWith(this.prefix)) {
        keys.add(name.substr(this.prefix.length))
      }
    }
    return [...keys]
  }

  /**
   * Returns a writable observable store that maps to the given preference.
   * Works with deep accesses and writes.
   *
   * @example
   *
   * ```svelte
   * <script lang="ts">
   *   const settings = Pref.bind("user-settings", { foo: "bar" })
   *   let current = $settings.foo // "bar"
   *   $settings.foo = "foobar" // updates localStorage!
   *   current = $settings.foo // "foobar"
   * </script>
   * ```
   *
   * @param name - The name/key for the preference.
   * @param fallback - The default value to return if the preference doesn't exist.
   */
  bind<T = JSONValue>(name: string, fallback?: T): Writable<T> {
    const handlerGet = this.get.bind(this)
    const handlerSet = this.set.bind(this)
    const store = writable<T>(this.get(name, fallback ?? ({} as T)))
    return {
      subscribe: store.subscribe,
      set: (val: T) => {
        // make a new object so we don't mutate old ones
        store.set(typeof val === "object" && !Array.isArray(val) ? { ...val } : val)
        handlerSet(name, val)
      },
      update(cur: Updater<T>) {
        cur.call(this, handlerGet(name, fallback ?? ({} as T)))
      }
    }
  }

  /**
   * Retrieves a 'wrapped' proxy object, or creates one if needed. Setting
   * items on this object will automatically cause the object to be stored.
   * This can be used to store a record of preferences without needing to
   * use an observable store.
   *
   * @example
   *
   * ```ts
   * type Settings = { foo: true; bar: { foo: false } }
   * const settings = Pref.wrap<Settings>("settings")
   * settings.foo = false // updates localStorage
   * settings.bar.foo = true // also updates localStorage, despite deeply accessing
   * ```
   */
  wrap<T extends JSONObject>(name: string, fallback: T): T {
    const wrapped = this.get(name, fallback)
    const handler: ProxyHandler<T> = {
      // handle nested objects by proxying them with the same handler
      get: (target, prop) => {
        const val = Reflect.get(target, prop)
        return typeof val === "object" ? new Proxy(val, handler) : val
      },
      // fire setter function whenever the object has a property set (even recursively)
      set: (target, prop, val) => {
        Reflect.set(target, prop, val)
        this.set(name, wrapped)
        return true
      }
    }
    return new Proxy(wrapped, handler)
  }
}

/**
 * Pre-made {@link PreferenceHandler} with the default `"_user-pref_"` prefix set.
 *
 * @see {@link PreferenceHandler}
 */
export const Pref = new PreferenceHandler()
