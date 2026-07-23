import assert from "node:assert/strict"
import test from "node:test"

import { discussionFormValues, discussionUpdateValue } from "../src/lib/admin-forum.js"

test("forum discussion form preserves Wikidot default, enable, and disable states", () => {
  assert.deepEqual(
    discussionFormValues({
      category_id: 1,
      slug: "article",
      per_page_discussion: null
    }),
    { categoryId: 1, state: "default" }
  )
  assert.equal(
    discussionFormValues({
      category_id: 2,
      slug: "article",
      per_page_discussion: true
    }).state,
    "enable"
  )
  assert.equal(
    discussionFormValues({
      category_id: 3,
      slug: "article",
      per_page_discussion: false
    }).state,
    "disable"
  )
})

test("_default has an effective disabled fallback instead of inheriting itself", () => {
  assert.equal(
    discussionFormValues({
      category_id: 1,
      slug: "_default",
      per_page_discussion: null
    }).state,
    "disable"
  )
})

test("forum discussion updates map default to null inheritance", () => {
  assert.equal(discussionUpdateValue({ state: "default" }), null)
  assert.equal(discussionUpdateValue({ state: "enable" }), true)
  assert.equal(discussionUpdateValue({ state: "disable" }), false)
})
