import { strict as assert } from "node:assert"
import test from "node:test"

import {
  resolveShellLayoutValue,
  shouldUseWikidotShellValue,
  WIKIDOT_LAYOUT,
  WIKIJUMP_LAYOUT
} from "../src/lib/wikidot-shell-decision.js"

test("site-level Wikijump layout takes precedence over Wikidot shell heuristics", () => {
  const data = {
    site: {
      layout: WIKIJUMP_LAYOUT,
      top_bar_page: "nav:top",
      side_bar_page: "nav:side"
    },
    page: { layout: null },
    compiled_top_bar_html: "<nav>top</nav>",
    compiled_side_bar_html: "<nav>side</nav>"
  }

  assert.equal(shouldUseWikidotShellValue(data), false)
  assert.equal(resolveShellLayoutValue(data), WIKIJUMP_LAYOUT)
})

test("page-level Wikidot layout takes precedence over site-level Wikijump layout", () => {
  const data = {
    site: { layout: WIKIJUMP_LAYOUT },
    page: { layout: WIKIDOT_LAYOUT }
  }

  assert.equal(shouldUseWikidotShellValue(data), true)
  assert.equal(resolveShellLayoutValue(data), WIKIDOT_LAYOUT)
})
