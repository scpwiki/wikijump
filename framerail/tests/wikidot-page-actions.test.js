import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotPageActionLabels,
  formatSigned
} from "../src/lib/wikidot-page-actions.js"

test("formats imported Wikidot action labels with source rating and comment counts", () => {
  assert.deepEqual(buildWikidotPageActionLabels({ rating: 19, comments: 2 }), {
    ratingText: "+19",
    rate: "Rate (+19)",
    discuss: "Discuss (2)"
  })
})

test("formats non-positive Wikidot ratings without an extra plus sign", () => {
  assert.equal(formatSigned(0), "0")
  assert.equal(formatSigned(-3), "-3")
})

test("falls back to count-less labels when imported snapshot counts are unavailable", () => {
  assert.deepEqual(buildWikidotPageActionLabels({ rating: null, comments: null }), {
    ratingText: null,
    rate: "Rate",
    discuss: "Discuss"
  })
})
