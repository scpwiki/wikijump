import assert from "node:assert/strict"
import crypto from "node:crypto"
import { existsSync } from "node:fs"
import fs from "node:fs/promises"
import test from "node:test"

import {
  FEED_ICON_SHA256,
  wikidotFeedIconPng
} from "../src/lib/server/wikidot-feed-icon.ts"

const sha256 = (bytes: Uint8Array | string) =>
  crypto.createHash("sha256").update(bytes).digest("hex")

test("the RSS chrome icon is a repository-authored SVG with a pinned render", async () => {
  const source = await fs.readFile(
    new URL("../../assets/feed-icon-14x14.svg", import.meta.url)
  )
  assert.equal(
    sha256(source),
    "8400559ef9d186d1d50671f63193b2e9343d0e26a03d1cd3232735ccbd786576"
  )
  assert.match(source.toString("utf8"), /width="14" height="14"/u)
  assert.doesNotMatch(source.toString("utf8"), /wikidot/iu)
})

test("the served icon bytes are the pinned 14x14 PNG render", () => {
  const png = wikidotFeedIconPng()
  assert.equal(sha256(png), FEED_ICON_SHA256)
  assert.deepEqual(
    [...png.subarray(0, 8)],
    [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
  )
  const view = new DataView(png.buffer, png.byteOffset, png.byteLength)
  assert.equal(view.getUint32(16), 14)
  assert.equal(view.getUint32(20), 14)
})

test("the icon route sets an explicit cache policy and image type without a proxy", async () => {
  const route = await fs.readFile(
    new URL(
      "../src/routes/common--theme/base/images/feed/feed-icon-14x14.png/+server.ts",
      import.meta.url
    ),
    "utf8"
  )
  assert.match(route, /"cache-control": "public, max-age=86400"/u)
  assert.match(route, /"content-type": "image\/png"/u)
  assert.doesNotMatch(route, /fetch\(|https?:\/\//u)
})

test("no Wikidot-hosted copy of the icon remains in the static tree", () => {
  assert.equal(
    existsSync(
      new URL(
        "../static/common--theme/base/images/feed/feed-icon-14x14.png",
        import.meta.url
      )
    ),
    false
  )
})
