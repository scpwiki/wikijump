import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotHovertipContainersHtml,
  positionWikidotHovertip,
  wikidotHovertipTexts
} from "../src/lib/wikidot/wikidot-hovertips.js"

// Raw server HTML of scp-wiki scp-173 (2026-10-11): the sixteen controls in
// Wikidot's order, edit-button alone in the first container.
const WIKIDOT_CONTROL_ORDER = [
  "edit-button",
  "edit-sections-button",
  "edit-append-button",
  "history-button",
  "discuss-button",
  "files-button",
  "site-tools-button",
  "backlinks-button",
  "rename-move-button",
  "view-source-button",
  "parent-page-button",
  "abuse-report-button",
  "bug-report-button",
  "wikidot-help-button",
  "wikidot-tos-button",
  "wikidot-privacy-button"
]

test("emits Wikidot's two hidden tip containers in source order", () => {
  for (const locale of ["en", "ja"]) {
    const html = buildWikidotHovertipContainersHtml(locale)
    assert.match(
      html,
      /^<div id="page-options-bottom-tips" style="display: none;"><div id="edit-button-hovertip">[^<]+<\/div><\/div><div id="page-options-bottom-2-tips" style="display: none;">/u
    )
    assert.deepEqual(
      [...html.matchAll(/<div id="([a-z-]+)-hovertip">/gu)].map((match) => match[1]),
      WIKIDOT_CONTROL_ORDER
    )
  }
})

test("uses the live English and Japanese tip texts", () => {
  const english = new Map([
    ...wikidotHovertipTexts("en").first,
    ...wikidotHovertipTexts("en").second
  ])
  const japanese = new Map([
    ...wikidotHovertipTexts("ja").first,
    ...wikidotHovertipTexts("ja").second
  ])
  assert.equal(english.get("edit-button"), "Click here to edit contents of this page.")
  assert.equal(
    english.get("edit-sections-button"),
    "Click here to toggle editing of individual sections of the page (if possible). Watch headings for an &quot;edit&quot; link when available."
  )
  assert.equal(english.get("wikidot-privacy-button"), "Wikidot.com Privacy Policy.")
  assert.equal(
    japanese.get("edit-button"),
    "ページを編集するにはこのボタンをクリックしてください。"
  )
  assert.equal(japanese.get("wikidot-privacy-button"), "Wikidot.com プライバシーポリシー")
})

test("positions tips with Wikidot's centered rule", () => {
  const viewport = { width: 1366, height: 900 }
  const tip = { width: 260, height: 32 }
  // Centered under the pointer, 20px below it.
  assert.deepEqual(
    positionWikidotHovertip({ pageY: 500, clientX: 400, clientY: 300 }, tip, viewport),
    { left: 270, top: 520 }
  )
  // Clamped 20px from the left edge (the live SCP-9506 Edit hover lands at
  // left 62px only because the pointer was further right).
  assert.deepEqual(
    positionWikidotHovertip({ pageY: 500, clientX: 30, clientY: 300 }, tip, viewport),
    { left: 20, top: 520 }
  )
  // Clamped 20px from the right edge.
  assert.deepEqual(
    positionWikidotHovertip({ pageY: 500, clientX: 1360, clientY: 300 }, tip, viewport),
    { left: 1086, top: 520 }
  )
  // Flipped above the pointer near the viewport bottom.
  assert.deepEqual(
    positionWikidotHovertip({ pageY: 2000, clientX: 400, clientY: 880 }, tip, viewport),
    { left: 270, top: 2000 - (32 + 30) + 20 }
  )
})
