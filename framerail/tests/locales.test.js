import assert from "node:assert/strict"
import { describe, it } from "node:test"

import {
  limitLocalePreferences,
  MAX_LOCALE_LENGTH,
  MAX_LOCALE_PREFERENCES
} from "../src/lib/locales.js"

describe("limitLocalePreferences", () => {
  it("deduplicates and caps user locale lists", () => {
    const locales = [
      "en",
      " en ",
      "fr",
      ...Array.from({ length: 50 }, (_, i) => `zz-${i}`)
    ]

    const limited = limitLocalePreferences(locales)

    assert.equal(limited.length, MAX_LOCALE_PREFERENCES)
    assert.deepEqual(limited.slice(0, 2), ["en", "fr"])
    assert.equal(new Set(limited).size, limited.length)
  })

  it("drops empty and oversized locale values", () => {
    const oversized = "x".repeat(MAX_LOCALE_LENGTH + 1)

    assert.deepEqual(limitLocalePreferences(["", "  ", oversized, "ja"]), ["ja"])
  })

  it("reserves capacity for required locales without reordering existing values", () => {
    const preferences = Array.from(
      { length: MAX_LOCALE_PREFERENCES },
      (_, index) => `pref-${index}`
    )

    assert.deepEqual(
      limitLocalePreferences([...preferences, "site", "en"], ["site", "en"]),
      [...preferences.slice(0, MAX_LOCALE_PREFERENCES - 2), "site", "en"]
    )
    assert.deepEqual(
      limitLocalePreferences(["en", ...preferences, "site"], ["site", "en"]),
      ["en", ...preferences.slice(0, MAX_LOCALE_PREFERENCES - 2), "site"]
    )
  })
})
