import assert from "node:assert/strict"
import test from "node:test"

import {
  resolveShellLayoutValue,
  shouldUseWikidotShellValue,
  type ShellViewData,
  WIKIDOT_LAYOUT,
  WIKIJUMP_LAYOUT
} from "../src/lib/layout/wikidot-shell-decision.ts"

const heuristics = {
  compiled_top_bar_html: "<nav>top</nav>",
  site: {
    from_wikidot: true,
    top_bar_page: "nav:top",
    side_bar_page: "nav:side"
  }
}

test("page layout takes precedence over site layout and shell heuristics", () => {
  assert.equal(
    shouldUseWikidotShellValue({
      ...heuristics,
      page: { layout: WIKIJUMP_LAYOUT },
      site: { ...heuristics.site, layout: WIKIDOT_LAYOUT }
    }),
    false
  )
  assert.equal(
    shouldUseWikidotShellValue({
      page: { layout: WIKIDOT_LAYOUT },
      site: { layout: WIKIJUMP_LAYOUT }
    }),
    true
  )
})

test("explicit site Wikijump layout takes precedence over every shell heuristic", () => {
  const data: ShellViewData = {
    ...heuristics,
    page: { layout: null, from_wikidot: true },
    page_revision: { from_wikidot: true },
    compiled_side_bar_html: "<nav>side</nav>",
    site: { ...heuristics.site, layout: WIKIJUMP_LAYOUT }
  }

  assert.equal(shouldUseWikidotShellValue(data), false)
  assert.equal(resolveShellLayoutValue(data), WIKIJUMP_LAYOUT)
})

test("site Wikidot layout remains authoritative without a page override", () => {
  const data: ShellViewData = {
    page: { layout: null },
    site: { layout: WIKIDOT_LAYOUT }
  }

  assert.equal(shouldUseWikidotShellValue(data), true)
  assert.equal(resolveShellLayoutValue(data), WIKIDOT_LAYOUT)
})

test("shell heuristics apply only when page and site layouts are unspecified", () => {
  assert.equal(shouldUseWikidotShellValue({ compiled_top_bar_html: "<nav />" }), true)
  assert.equal(
    shouldUseWikidotShellValue({
      site: { top_bar_page: "nav:top", side_bar_page: "nav:side" }
    }),
    true
  )
  assert.equal(
    shouldUseWikidotShellValue({
      page_revision: { from_wikidot: true },
      site: { top_bar_page: "custom-nav" }
    }),
    true
  )
  assert.equal(
    shouldUseWikidotShellValue({ site: { top_bar_page: "custom-nav" } }),
    false
  )
  assert.equal(resolveShellLayoutValue(undefined), WIKIJUMP_LAYOUT)
})
