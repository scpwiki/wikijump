import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotPageInfoText,
  formatWikidotSourceDate
} from "../src/lib/wikidot-page-info.js"

test("formats imported Wikidot page revision metadata like the source shell", () => {
  const updatedAt = "2024-03-27T20:18:44Z"
  const now = Date.parse("2026-07-01T12:00:00+09:00")

  assert.equal(formatWikidotSourceDate(Date.parse(updatedAt)), "28 Mar 2024, 05:18")
  assert.equal(
    buildWikidotPageInfoText({ revision: 4, updatedAt, now }),
    "page revision: 4, last edited: 28 Mar 2024, 05:18 (825 days ago)"
  )
})

test("rejects invalid imported Wikidot page revision timestamps", () => {
  assert.equal(
    buildWikidotPageInfoText({
      revision: 4,
      updatedAt: "not a timestamp"
    }),
    null
  )
})
