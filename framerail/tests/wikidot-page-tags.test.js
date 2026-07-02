import { strict as assert } from "node:assert"
import test from "node:test"

import { wikidotTagSeparator } from "../src/lib/wikidot-page-tags.js"

test("separates Wikidot page tags in browser-visible text", () => {
  const tags = ["admonition", "artifact", "chemical"]
  const visibleText = tags
    .map((tag, index) => `${wikidotTagSeparator(index)}${tag}`)
    .join("")

  assert.equal(wikidotTagSeparator(0), "")
  assert.equal(wikidotTagSeparator(1), " ")
  assert.equal(visibleText, "admonition artifact chemical")
})
