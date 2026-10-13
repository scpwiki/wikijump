import assert from "node:assert/strict"
import test from "node:test"

import {
  comparableHistoryTimelineNumbers,
  defaultHistoryComparePair,
  isComparableHistoryRevision
} from "../src/lib/history-compare-defaults.js"

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

test("only page revisions with a backing snapshot are comparable", () => {
  assert.equal(
    isComparableHistoryRevision({
      history_kind: "page",
      timeline_number: 3,
      page_revision_number: 2
    }),
    true
  )
  // File events are timeline entries, not page source revisions, even
  // when they carry a mapped page snapshot for viewing.
  assert.equal(
    isComparableHistoryRevision({
      history_kind: "file",
      timeline_number: 4,
      page_revision_number: 2
    }),
    false
  )
  assert.equal(
    isComparableHistoryRevision({
      history_kind: "file",
      timeline_number: 5,
      page_revision_number: null
    }),
    false
  )
  assert.equal(
    isComparableHistoryRevision({
      history_kind: "page",
      timeline_number: 6,
      page_revision_number: null
    }),
    false
  )
})

test("compare selector offers only comparable page revisions, oldest first", () => {
  const entries = [
    { history_kind: "file", timeline_number: 12, page_revision_number: 5 },
    { history_kind: "page", timeline_number: 11, page_revision_number: 5 },
    { history_kind: "file", timeline_number: 10, page_revision_number: null },
    { history_kind: "page", timeline_number: 9, page_revision_number: 4 },
    { history_kind: "page", timeline_number: 8, page_revision_number: null }
  ]

  assert.deepEqual(comparableHistoryTimelineNumbers(entries), [9, 11])
})

test("a single page revision buried in file events offers no comparison pair", () => {
  // SCP-9506 shape from #2143: one page creation plus file rows 20-2.
  const entries = [
    { history_kind: "page", timeline_number: 0, page_revision_number: 0 },
    ...Array.from({ length: 19 }, (_, index) => ({
      history_kind: "file",
      timeline_number: index + 1,
      page_revision_number: 0
    }))
  ]

  assert.deepEqual(comparableHistoryTimelineNumbers(entries), [0])
  assert.equal(defaultHistoryComparePair(entries), undefined)
})
