import assert from "node:assert/strict"
import crypto from "node:crypto"
import fs from "node:fs/promises"
import test from "node:test"

import {
  selectSiteIconResponse,
  storedSiteIconResponse
} from "../src/lib/site-icon-response.ts"

const importedSite = { slug: "scp-wiki", from_wikidot: true }

// Retained live Wikidot bytes for scp-wiki's favicon route (HTTP 200,
// image/gif, etag "e711917c63f9c9783d819cc402f3359d", captured 2026-10-12).
const FAVICON_SHA256 = "5dfe8fdaa35561856164200c8619fefa2f22d0668393fb777788df0a00c2bd0e"

const retainedFavicon = () =>
  fs.readFile(
    new URL("../../deepwell/tests/fixtures/site-icon-favicon-scp-wiki.gif", import.meta.url)
  )

test("stored bytes serve with Wikidot's header shape and byte identity", async () => {
  const data = new Uint8Array(await retainedFavicon())
  assert.equal(crypto.createHash("sha256").update(data).digest("hex"), FAVICON_SHA256)

  const response = storedSiteIconResponse({
    mime: "image/gif",
    sha256Hex: FAVICON_SHA256,
    data
  })
  assert.equal(response.status, 200)
  assert.equal(response.headers.get("content-type"), "image/gif; charset=utf-8")
  assert.equal(response.headers.get("etag"), `"${FAVICON_SHA256}"`)
  assert.equal(
    crypto.createHash("sha256").update(new Uint8Array(await response.arrayBuffer())).digest("hex"),
    FAVICON_SHA256
  )
})

test("stored bytes win over a local source so mirrors avoid Wikidot", async () => {
  const data = new Uint8Array(await retainedFavicon())
  const response = selectSiteIconResponse(
    importedSite,
    "/local--files/site/favicon.gif",
    { mime: "image/gif", sha256Hex: FAVICON_SHA256, data },
    "favicon",
    "favicon.gif"
  )
  assert.equal(response.status, 200)
})

test("a Wikidot source with nothing materialized is a 404, not a hotlink", () => {
  const cases = [
    ["favicon", "https://scp-wiki.wikidot.com/local--favicon/favicon.gif", "favicon.gif"],
    ["ios", "https://scp-wiki.wikidot.com/local--iosicon/iosicon.png", "iosicon.png"],
    ["windows", "https://scp-wiki.wikidot.com/local--wp8icon/wp8icon.png", "wp8icon.png"]
  ]
  for (const [kind, source, filename] of cases) {
    const response = selectSiteIconResponse(importedSite, source, null, kind, filename)
    assert.equal(response.status, 404, kind)
  }
})

test("a same-origin local source without bytes still redirects", () => {
  const response = selectSiteIconResponse(
    importedSite,
    "/local--files/site/favicon.gif",
    null,
    "favicon",
    "favicon.gif"
  )
  assert.equal(response.status, 302)
  assert.equal(response.headers.get("location"), "/local--files/site/favicon.gif")
})

test("non-canonical filenames are 404 even with stored bytes", async () => {
  const data = new Uint8Array(await retainedFavicon())
  const stored = { mime: "image/gif", sha256Hex: FAVICON_SHA256, data }
  assert.equal(
    selectSiteIconResponse(importedSite, "/local--files/site/favicon.gif", stored, "favicon", "other.gif")
      .status,
    404
  )
  assert.equal(
    selectSiteIconResponse(importedSite, "/local--files/site/x.png", null, "ios", "other.png").status,
    404
  )
  for (const filename of ["iosicon_57.png", "iosicon_72.png", "iosicon.png"]) {
    assert.equal(
      selectSiteIconResponse(importedSite, "/local--files/site/x.png", null, "ios", filename).status,
      302,
      filename
    )
  }
})

test("an unconfigured site answers 404", () => {
  assert.equal(
    selectSiteIconResponse(importedSite, null, null, "favicon", "favicon.gif").status,
    404
  )
  assert.equal(selectSiteIconResponse(null, null, null, "favicon", "favicon.gif").status, 404)
})
