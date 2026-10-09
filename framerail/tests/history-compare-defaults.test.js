import assert from "node:assert/strict"
import test from "node:test"

import { defaultHistoryComparePair } from "../src/lib/history-compare-defaults.js"

test("defaults comparison to the newest two distinct page revisions in a mixed timeline", () => {
  const entries = [
    { history_kind: "page", timeline_number: 12, page_revision_number: 5 },
    { history_kind: "file", timeline_number: 11, page_revision_number: 5 },
    { history_kind: "file", timeline_number: 10, page_revision_number: null },
    { history_kind: "page", timeline_number: 9, page_revision_number: 4 },
    { history_kind: "page", timeline_number: 8, page_revision_number: 4 },
    { history_kind: "page", timeline_number: 7, page_revision_number: 3 }
  ]

  assert.deepEqual(defaultHistoryComparePair(entries), [9, 12])
})

test("leaves the default unset unless two distinct page revisions are available", () => {
  assert.equal(
    defaultHistoryComparePair([
      { history_kind: "file", timeline_number: 4, page_revision_number: 2 },
      { history_kind: "page", timeline_number: 3, page_revision_number: 2 },
      { history_kind: "page", timeline_number: 2, page_revision_number: 2 }
    ]),
    undefined
  )
})
