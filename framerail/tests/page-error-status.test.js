import assert from "node:assert/strict"
import test from "node:test"

import { pageErrorStatus } from "../src/lib/server/load/page/page-error-status.js"

test("reserved admin permission views stay unauthorized across route aliases", () => {
  for (const slug of ["_admin", "_ADMIN", "/_admin", "//_ADMIN//"]) {
    assert.equal(pageErrorStatus("permissions", slug), 401, slug)
  }
})

test("ordinary page errors keep their existing HTTP status", () => {
  assert.equal(pageErrorStatus("permissions", "example"), 403)
  assert.equal(pageErrorStatus("missing", "_admin"), 404)
})
