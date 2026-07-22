import assert from "node:assert/strict"
import { describe, it } from "node:test"

import { isJapaneseWikidotLocale, toIntlLocales } from "../src/lib/wikidot-locale.js"

describe("Wikidot locale compatibility", () => {
  it("recognizes Wikidot Japanese locale identifiers", () => {
    assert.equal(isJapaneseWikidotLocale("ja-corrections"), true)
    assert.equal(isJapaneseWikidotLocale("ja_JP"), true)
    assert.equal(isJapaneseWikidotLocale("en"), false)
  })

  it("maps ja-corrections only at the Intl boundary", () => {
    const wikidotLocales = ["en-US", "ja-corrections", "en"]

    assert.deepEqual(toIntlLocales(wikidotLocales), ["en-US", "ja", "en"])
    assert.deepEqual(wikidotLocales, ["en-US", "ja-corrections", "en"])
    assert.doesNotThrow(() =>
      new Date("2026-07-22T00:00:00Z").toLocaleString(toIntlLocales(wikidotLocales))
    )
  })
})
