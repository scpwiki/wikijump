import assert from "node:assert/strict"
import test from "node:test"

import { loadSiteInfo } from "../src/lib/server/load/site-info.ts"

test("loads trusted site headers", () => {
  const headers = new Headers({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "example"
  })

  assert.deepEqual(loadSiteInfo(headers), {
    siteId: 6000005,
    siteSlug: "example"
  })
})

test("rejects requests without an internal site id", () => {
  const headers = new Headers({
    "X-Wikijump-Site-Slug": "example"
  })

  assert.throws(
    () => loadSiteInfo(headers),
    /Missing wws internal header 'X-Wikijump-Site-Id'/
  )
})

test("rejects requests without an internal site slug", () => {
  const headers = new Headers({
    "X-Wikijump-Site-Id": "6000005"
  })

  assert.throws(
    () => loadSiteInfo(headers),
    /Missing wws internal header 'X-Wikijump-Site-Slug'/
  )
})
