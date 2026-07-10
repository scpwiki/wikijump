import { strict as assert } from "node:assert"
import test from "node:test"

import { Layout } from "../src/lib/types.js"
import { resolveShellLayout, shouldUseWikidotShell } from "../src/lib/wikidot-shell.js"

test("site-level Wikijump layout takes precedence over Wikidot shell heuristics", () => {
  const data = {
    site: {
      layout: Layout.WIKIJUMP,
      top_bar_page: "nav:top",
      side_bar_page: "nav:side"
    },
    page: { layout: null },
    compiled_top_bar_html: "<nav>top</nav>",
    compiled_side_bar_html: "<nav>side</nav>"
  }

  assert.equal(shouldUseWikidotShell(data), false)
  assert.equal(resolveShellLayout(data), Layout.WIKIJUMP)
})

test("page-level Wikidot layout takes precedence over site-level Wikijump layout", () => {
  const data = {
    site: { layout: Layout.WIKIJUMP },
    page: { layout: Layout.WIKIDOT }
  }

  assert.equal(shouldUseWikidotShell(data), true)
  assert.equal(resolveShellLayout(data), Layout.WIKIDOT)
})
