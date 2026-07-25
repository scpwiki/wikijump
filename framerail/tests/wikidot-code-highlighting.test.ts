import assert from "node:assert/strict"
import test from "node:test"

import { highlightWikidotCodeSource } from "../src/lib/wikidot/wikidot-code-highlighting.ts"

test("highlights CSS using the existing Wikijump token class contract", async () => {
  const highlighted = await highlightWikidotCodeSource(
    "#header h2 span { color: red; }",
    "css"
  )

  assert.equal(highlighted?.language, "css")
  assert.match(highlighted?.html ?? "", /wj-code-token/)
  assert.match(highlighted?.html ?? "", /wj-code-selector/)
  assert.match(highlighted?.html ?? "", /wj-code-property/)
})

test("loads optional grammars and normalizes common aliases", async () => {
  const highlighted = await highlightWikidotCodeSource(
    "def greet():\n    return True",
    "py"
  )

  assert.equal(highlighted?.language, "python")
  assert.match(highlighted?.html ?? "", /wj-code-keyword/)
  assert.match(highlighted?.html ?? "", /wj-code-boolean/)
})

test("escapes source markup before inserting highlighted HTML", async () => {
  const highlighted = await highlightWikidotCodeSource(
    "<script>alert(1)</script>",
    "html"
  )

  // Case-insensitive, and covering the closing tag: an escaper that let
  // `<SCRIPT>` or `</script>` through would satisfy a literal `<script>`
  // check while still emitting live markup.
  assert.doesNotMatch(highlighted?.html ?? "", /<\/?script/i)
  assert.match(highlighted?.html ?? "", /&lt;/)
})

test("leaves unknown and oversized source as DOM text", async () => {
  assert.equal(
    await highlightWikidotCodeSource("<script>alert(1)</script>", "unknown"),
    null
  )
  assert.equal(await highlightWikidotCodeSource("x".repeat(100_001), "css"), null)
})
