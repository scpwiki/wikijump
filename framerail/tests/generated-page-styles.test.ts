import assert from "node:assert/strict"
import test from "node:test"

import { buildGeneratedPageStylesHead } from "../src/lib/generated-page-styles.ts"

test("builds ordered head styles for multiple generated CSS modules", () => {
  assert.equal(
    buildGeneratedPageStylesHead([".first { color: red; }", ".second { color: blue; }"]),
    '<style type="text/css" data-wikijump-generated-css="0">.first { color: red; }</style><style type="text/css" data-wikijump-generated-css="1">.second { color: blue; }</style>'
  )
})

test("keeps CSS in raw-text context when content attempts to close or forge a style", () => {
  const html = buildGeneratedPageStylesHead([
    '</style><meta name="injected"><style data-wikijump-generated-css="99">'
  ])

  assert.equal(html.match(/<style/g)?.length, 1)
  assert.equal(html.match(/<\/style>/g)?.length, 1)
  assert(!html.includes('<meta name="injected">'))
  assert(html.includes('\\3C /style>\\3C meta name="injected">'))
  assert(html.includes('\\3C style data-wikijump-generated-css="99">'))
})
