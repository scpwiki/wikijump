import assert from "node:assert/strict"
import test from "node:test"

import {
  highlightWikidotCodeSource,
  replaceWikidotCodeHtml
} from "../src/lib/wikidot/wikidot-code-highlighting.ts"

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

test("replaces highlighted markup through an inert parsed document", () => {
  const firstNode = { name: "first" }
  const secondNode = { name: "second" }
  const parserCalls: [string, DOMParserSupportedType][] = []
  const replacementCalls: unknown[][] = []
  const originalDescriptor = Object.getOwnPropertyDescriptor(globalThis, "DOMParser")

  class FakeDOMParser {
    parseFromString(html: string, type: DOMParserSupportedType): Document {
      parserCalls.push([html, type])
      return {
        body: { childNodes: [firstNode, secondNode] }
      } as unknown as Document
    }
  }

  Object.defineProperty(globalThis, "DOMParser", {
    configurable: true,
    value: FakeDOMParser
  })

  try {
    const code = {
      replaceChildren(...nodes: unknown[]) {
        replacementCalls.push(nodes)
      }
    } as unknown as HTMLElement

    replaceWikidotCodeHtml(code, '<span class="wj-code-token">safe</span>')

    assert.deepEqual(parserCalls, [
      ['<span class="wj-code-token">safe</span>', "text/html"]
    ])
    assert.deepEqual(replacementCalls, [[firstNode, secondNode]])
  } finally {
    if (originalDescriptor) {
      Object.defineProperty(globalThis, "DOMParser", originalDescriptor)
    } else {
      Reflect.deleteProperty(globalThis, "DOMParser")
    }
  }
})

test("leaves unknown and oversized source as DOM text", async () => {
  assert.equal(
    await highlightWikidotCodeSource("<script>alert(1)</script>", "unknown"),
    null
  )
  assert.equal(await highlightWikidotCodeSource("x".repeat(100_001), "css"), null)
})
