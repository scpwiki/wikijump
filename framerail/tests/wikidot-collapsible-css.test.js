import { strict as assert } from "node:assert"
import { readFile } from "node:fs/promises"
import test from "node:test"

const wikidotLayoutPath = new URL(
  "../src/lib/sigma-esque/wikidot.svelte",
  import.meta.url
)

test("uses Wikidot-style folded and unfolded labels for details collapsibles", async () => {
  const source = await readFile(wikidotLayoutPath, "utf8")

  assert.match(
    source,
    /details\.collapsible-block:not\(\[open\]\) \.collapsible-block-unfolded-link\s*\{\s*display: none;\s*\}/
  )
  assert.match(
    source,
    /details\.collapsible-block\[open\]\s*>\s*summary\s*\.collapsible-block-link:not\(\.collapsible-block-unfolded-link\)\s*\{\s*display: none;\s*\}/
  )
})
