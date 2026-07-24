import assert from "node:assert/strict"
import { describe, it } from "node:test"

import { wikidotRatyAsset } from "../src/lib/server/wikidot-raty-assets.ts"

const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]

describe("Wikidot five-star rating assets", () => {
  for (const name of ["star-off.png", "star-on.png", "star-half.png"]) {
    it(`serves the captured ${name} PNG`, () => {
      const asset = wikidotRatyAsset(name)
      assert.ok(asset)
      assert.deepEqual([...new Uint8Array(asset, 0, PNG_SIGNATURE.length)], PNG_SIGNATURE)
    })
  }

  it("fails closed for unknown common image paths", () => {
    assert.equal(wikidotRatyAsset("unrelated.png"), undefined)
  })
})
