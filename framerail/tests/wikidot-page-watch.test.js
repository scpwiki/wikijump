import { strict as assert } from "node:assert"
import test from "node:test"

import { buildWikidotPageWatchLabel } from "../src/lib/wikidot/wikidot-page-watch.js"

test("renders authenticated sandbox site watch text", () => {
  assert.deepEqual(
    buildWikidotPageWatchLabel({
      sourceSite: "sandbox-for-codex",
      hasSession: true
    }),
    {
      label: "Stop watching site sandbox-for-codex.wikidot.com",
      helpLabel: "?",
      helpHref: "http://www.wikidot.com/faq:watching"
    }
  )
})

test("omits sandbox site watch text without an authenticated session", () => {
  assert.equal(
    buildWikidotPageWatchLabel({
      sourceSite: "sandbox-for-codex",
      hasSession: false
    }),
    null
  )
})

test("renders authenticated Japanese site watch text", () => {
  assert.deepEqual(
    buildWikidotPageWatchLabel({
      sourceSite: "scpaiueouiuiuiui",
      hasSession: true,
      locale: "ja"
    }),
    {
      label: "このサイトのウォッチングを終了",
      helpLabel: "?",
      helpHref: "http://www.wikidot.com/faq:watching"
    }
  )
})

test("renders authenticated generic site watch text for other imported source sites", () => {
  assert.deepEqual(
    buildWikidotPageWatchLabel({
      sourceSite: "scp-wiki",
      hasSession: true
    }),
    {
      label: "Stop watching site scp-wiki.wikidot.com",
      helpLabel: "?",
      helpHref: "http://www.wikidot.com/faq:watching"
    }
  )
})

test("omits site watch text without a source site", () => {
  assert.equal(buildWikidotPageWatchLabel({ sourceSite: null, hasSession: true }), null)
})
