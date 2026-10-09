import assert from "node:assert/strict"
import test from "node:test"

import {
  articleRouteWithQueryNoRender,
  queryNoRenderSelected
} from "../src/lib/server/page-norender-query.ts"

test("observed query values activate no-render", () => {
  for (const value of ["true", "t", "1", "false"]) {
    assert.equal(queryNoRenderSelected(`?norender=${value}`), true, value)
  }
  assert.equal(queryNoRenderSelected("?view=1&norender=true"), true)
  assert.equal(queryNoRenderSelected("?NORENDER=t"), true)
  assert.equal(queryNoRenderSelected("?norender=true&norender=0"), true)
})

test("zero, empty, bare, and unrecognized values leave the page rendered", () => {
  assert.equal(queryNoRenderSelected("?norender=0"), false)
  assert.equal(queryNoRenderSelected("?norender"), false)
  assert.equal(queryNoRenderSelected("?norender="), false)
  assert.equal(queryNoRenderSelected("?norender=yes"), false)
  assert.equal(queryNoRenderSelected("?norender=a%2Fb"), false)
  assert.equal(queryNoRenderSelected("?norender=0&norender=true"), false)
  assert.equal(queryNoRenderSelected("?other=true"), false)
  assert.equal(queryNoRenderSelected("?"), false)
  assert.equal(queryNoRenderSelected(""), false)
})

test("article route receives the canonical path selector", () => {
  assert.deepEqual(
    articleRouteWithQueryNoRender({ slug: "scp-173", extra: "" }, "?norender=true"),
    { slug: "scp-173", extra: "norender/true" }
  )
  assert.deepEqual(
    articleRouteWithQueryNoRender({ slug: "scp-173", extra: "" }, "?norender=false"),
    { slug: "scp-173", extra: "norender/true" }
  )
  assert.deepEqual(
    articleRouteWithQueryNoRender({ slug: "scp-173", extra: "edit/true" }, "?norender=t"),
    { slug: "scp-173", extra: "edit/true/norender/true" }
  )
})

test("routes without a selector are returned unchanged", () => {
  const route = { slug: "scp-173", extra: "" }
  assert.equal(articleRouteWithQueryNoRender(route, "?norender=0"), route)
  assert.equal(articleRouteWithQueryNoRender(route, "?norender"), route)
  assert.equal(articleRouteWithQueryNoRender(route, "?view=history"), route)
  assert.equal(articleRouteWithQueryNoRender(null, "?norender=true"), null)
})
