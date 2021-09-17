import type { Regex } from "./definition"

/**
 * Safely compiles a regular expression.
 *
 * @example
 *
 * ```ts
 * // returns null if features aren't supported (e.g. Safari)
 * const regex = re`/(?<=\d)\w+/d`
 * ```
 */
export function re(str: TemplateStringsArray | string, forceFlags = "") {
  const input = typeof str === "string" ? str : str.raw[0]
  const split = /^\/([^]+)\/([^]*)$/.exec(input)

  if (!split || !split[1]) return null

  let [, src = "", flags = ""] = split

  if (forceFlags) {
    // goofy looking, but this is just deduplicating the flags
    flags = [...new Set([...flags, ...forceFlags])].join("")
  }

  try {
    return new RegExp(src, flags)
  } catch (err) {
    console.warn("cm-tarnation: Recovered from failed RegExp construction")
    console.warn("cm-tarnation: RegExp source:", input)
    console.warn(err)
    return null
  }
}

/**
 * Tests if the given string is a "RegExp string", as in it's in the format
 * of a native `RegExp` statement.
 */
export function isRegExpString(str: string): str is Regex {
  const split = /^\/([^]+)\/([^]*)$/.exec(str)
  if (!split || !split[1]) return false
  return true
}

/** Returns if the given `RegExp` has any remembered capturing groups. */
export function hasCapturingGroups(regexp: RegExp) {
  // give an alternative that always matches
  const always = new RegExp(`|${regexp.source}`)
  // ... which means we can use it to get a successful match,
  // regardless of the original regex. this is a bit of a hack,
  // but we can use this to detect capturing groups.
  return always.exec("")!.length > 1
}

/**
 * Creates a lookbehind function from a `RegExp`. This function can only
 * test for a pattern's (non) existence, so no matches or capturing groups
 * are returned.
 *
 * @param pattern - A `RegExp` to be used as a pattern.
 * @param negative - Negates the pattern.
 */
export function createLookbehind(pattern: RegExp, negative?: boolean) {
  // can't be sticky, global, or multiline
  const flags = pattern.flags.replaceAll(/[ygm]/, "")

  // regexp that can only match at the end of a string
  const regex = new RegExp(`(?:${pattern.source})$`, flags)

  return (str: string, pos: number) => {
    const clipped = str.slice(0, pos)
    const result = regex.test(clipped)
    return negative ? !result : result
  }
}
