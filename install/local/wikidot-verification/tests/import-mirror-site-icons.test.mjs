import assert from "node:assert/strict"
import test from "node:test"

import { declaredSiteIcons, iconMime } from "../scripts/import-mirror-site-icons.mjs"

// The head served by scp-wiki.wikidot.com on 2026-10-11 (retained SCP-9506
// capture scp9506-live-20261012-chromium149).
const SCP_WIKI_HEAD = `<html><head>
<link rel="shortcut icon" href="/local--favicon/favicon.gif"/>
<link rel="icon" type="image/gif" href="/local--favicon/favicon.gif"/>
<link rel="apple-touch-icon" href="/local--iosicon/iosicon_57.png" />
<link rel="apple-touch-icon" sizes="72x72" href="/local--iosicon/iosicon_72.png" />
<link rel="apple-touch-icon" sizes="114x114" href="/local--iosicon/iosicon.png" />
</head><body><link rel="icon" href="/local--favicon/not-in-head.gif"></body></html>`

test("declared icons follow the source head and keep the largest iOS size", () => {
  assert.deepEqual(declaredSiteIcons(SCP_WIKI_HEAD, "https://scp-wiki.wikidot.com"), {
    favicon: "https://scp-wiki.wikidot.com/local--favicon/favicon.gif",
    ios: "https://scp-wiki.wikidot.com/local--iosicon/iosicon.png"
  })
})

test("a Windows tile declared through msapplication-TileImage is imported", () => {
  const head = `<head><meta name="msapplication-TileImage" content="/local--wp8icon/wp8icon.png"></head>`
  assert.deepEqual(declaredSiteIcons(head, "http://example.wikidot.com"), {
    windows: "http://example.wikidot.com/local--wp8icon/wp8icon.png"
  })
})

test("icons on other origins or outside the icon routes are ignored", () => {
  const head = `<head>
<link rel="icon" href="https://cdn.example.com/local--favicon/favicon.gif">
<link rel="icon" href="/local--files/start/favicon.png">
</head>`
  assert.deepEqual(declaredSiteIcons(head, "https://scp-wiki.wikidot.com"), {})
})

test("the observed media type is kept, including a GIF route serving PNG bytes", () => {
  assert.equal(iconMime("image/gif; charset=utf-8"), "image/gif")
  assert.equal(iconMime("image/png"), "image/png")
  assert.throws(() => iconMime("text/html; charset=utf-8"), /unsupported site icon media type/u)
})
