import assert from "node:assert/strict"
import test from "node:test"

import {
  articleRouteWithQueryNoRender,
  queryNoRenderValue
} from "../src/lib/server/page-norender-query.ts"

test("query no-render value is read from the norender key only", () => {
  assert.equal(queryNoRenderValue("?norender=true"), "true")
  assert.equal(queryNoRenderValue("?norender=t"), "t")
  assert.equal(queryNoRenderValue("?norender=1"), "1")
  assert.equal(queryNoRenderValue("?norender=false"), "false")
  assert.equal(queryNoRenderValue("?norender=0"), "0")
  assert.equal(queryNoRenderValue("?NORENDER=true"), "true")
  assert.equal(queryNoRenderValue("?view=1&norender=true"), "true")
  assert.equal(queryNoRenderValue("?norender=true&norender=false"), "true")
  assert.equal(queryNoRenderValue("?other=true"), null)
  assert.equal(queryNoRenderValue("?"), null)
  assert.equal(queryNoRenderValue(""), null)
})

test("bare, empty, and path-like query values do not select no-render", () => {
  assert.equal(queryNoRenderValue("?norender"), null)
  assert.equal(queryNoRenderValue("?norender="), null)
  assert.equal(queryNoRenderValue("?norender=a%2Fb"), null)
  assert.equal(queryNoRenderValue("?norender=true/edit"), null)
  assert.equal(queryNoRenderValue("?norender=%3F"), null)
})

test("article route receives the query selector as a path argument", () => {
  assert.deepEqual(
    articleRouteWithQueryNoRender({ slug: "scp-173", extra: "" }, "?norender=true"),
    { slug: "scp-173", extra: "norender/true" }
  )
  assert.deepEqual(
    articleRouteWithQueryNoRender({ slug: "scp-173", extra: "edit/true" }, "?norender=t"),
    { slug: "scp-173", extra: "edit/true/norender/t" }
  )
})

test("routes without a selector are returned unchanged", () => {
  const route = { slug: "scp-173", extra: "" }
  assert.equal(articleRouteWithQueryNoRender(route, "?norender"), route)
  assert.equal(articleRouteWithQueryNoRender(route, "?view=history"), route)
  assert.equal(articleRouteWithQueryNoRender(null, "?norender=true"), null)
})
