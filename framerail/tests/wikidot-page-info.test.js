import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotPageInfoText,
  formatWikidotSourceDate
} from "../src/lib/wikidot/wikidot-page-info.js"

test("formats imported Wikidot page revision metadata like the source shell", () => {
  const updatedAt = "2024-03-27T20:18:44Z"
  const now = Date.parse("2026-07-01T12:00:00+09:00")

  assert.equal(formatWikidotSourceDate(Date.parse(updatedAt)), "28 Mar 2024, 05:18")
  assert.equal(
    buildWikidotPageInfoText({ revision: 4, updatedAt, now }),
    "page revision: 4, last edited: 28 Mar 2024, 05:18 (825 days ago)"
  )
})

test("formats recent Wikidot page revision metadata with elapsed hours", () => {
  const updatedAt = "2026-07-01T05:52:50Z"
  const now = Date.parse("2026-07-01T16:30:00Z")

  assert.equal(
    buildWikidotPageInfoText({ revision: 234, updatedAt, now }),
    "page revision: 234, last edited: 1 Jul 2026, 14:52 (10 hours ago)"
  )
})

test("formats Japanese Wikidot page revision metadata like SCP-JP", () => {
  const updatedAt = "2023-01-07T06:16:27Z"
  const now = Date.parse("2026-07-04T00:16:27Z")

  assert.equal(
    buildWikidotPageInfoText({ revision: 6, updatedAt, now, locale: "ja" }),
    "ページリビジョン: 6, 最終更新: 7 Jan 2023, 15:16 (1273 days 前)"
  )
})

test("formats singular recent Wikidot page revision metadata units", () => {
  const updatedAt = "2026-07-01T05:52:50Z"

  assert.equal(
    buildWikidotPageInfoText({
      revision: 234,
      updatedAt,
      now: Date.parse("2026-07-01T06:53:00Z")
    }),
    "page revision: 234, last edited: 1 Jul 2026, 14:52 (1 hour ago)"
  )
  assert.equal(
    buildWikidotPageInfoText({
      revision: 234,
      updatedAt,
      now: Date.parse("2026-07-01T05:53:50Z")
    }),
    "page revision: 234, last edited: 1 Jul 2026, 14:52 (1 minute ago)"
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
