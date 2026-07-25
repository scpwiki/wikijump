import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotDiscussButtonHtml,
  buildWikidotPageActionLabels,
  formatSigned,
  isWikidotFragmentPage,
  sourceShowsStandardWikidotPageActions
} from "../src/lib/wikidot/wikidot-page-actions.js"

test("discussion action uses the frozen Wikidot DOM marker and escapes its label", () => {
  assert.equal(
    buildWikidotDiscussButtonHtml('Discuss <"unsafe">'),
    '<a href="javascript:;" class="btn btn-default" id="discuss-button" onclick="WIKIDOT.page.listeners.createPageDiscussion(event)">Discuss &lt;"unsafe"&gt;</a>'
  )
})

test("formats imported Wikidot action labels with source rating and comment counts", () => {
  assert.deepEqual(buildWikidotPageActionLabels({ rating: 19, comments: 2 }), {
    edit: "Edit",
    ratingText: "+19",
    showRate: true,
    showDiscuss: true,
    ratePrefix: "Rate",
    rate: "Rate (+19)",
    tags: "Tags",
    discuss: "Discuss (2)",
    history: "History",
    files: "Files",
    print: "Print",
    siteTools: "Site tools",
    options: "Options"
  })
})

test("formats non-positive Wikidot ratings without an extra plus sign", () => {
  assert.equal(formatSigned(0), "0")
  assert.equal(formatSigned(-3), "-3")
})

test("falls back to count-less labels when imported snapshot counts are unavailable", () => {
  assert.deepEqual(buildWikidotPageActionLabels({ rating: null, comments: null }), {
    edit: "Edit",
    ratingText: null,
    showRate: true,
    showDiscuss: true,
    ratePrefix: "Rate",
    rate: "Rate",
    tags: "Tags",
    discuss: "Discuss",
    history: "History",
    files: "Files",
    print: "Print",
    siteTools: "Site tools",
    options: "Options"
  })
})

test("formats Japanese Wikidot action labels with source counts", () => {
  assert.deepEqual(
    buildWikidotPageActionLabels({ rating: 35, comments: 4, locale: "ja" }),
    {
      edit: "編集",
      ratingText: "+35",
      showRate: true,
      showDiscuss: true,
      ratePrefix: "評価",
      rate: "評価 (+35)",
      tags: "タグ",
      discuss: "ディスカッション (4)",
      history: "履歴",
      files: "ファイル",
      print: "印刷",
      siteTools: "サイトツール",
      options: "オプション"
    }
  )
})

test("can suppress source-disabled Wikidot page actions", () => {
  assert.deepEqual(
    buildWikidotPageActionLabels({
      rating: 0,
      comments: 0,
      showRate: false,
      showDiscuss: false
    }),
    {
      edit: "Edit",
      ratingText: "0",
      showRate: false,
      showDiscuss: false,
      ratePrefix: "Rate",
      rate: "Rate (0)",
      tags: "Tags",
      discuss: "Discuss (0)",
      history: "History",
      files: "Files",
      print: "Print",
      siteTools: "Site tools",
      options: "Options"
    }
  )
})

test("detects imported source sites without standard page actions", () => {
  assert.equal(sourceShowsStandardWikidotPageActions("sandbox-for-codex"), false)
  assert.equal(sourceShowsStandardWikidotPageActions("scp-wiki"), true)
  assert.equal(sourceShowsStandardWikidotPageActions(null), true)
})

test("detects direct Wikidot fragment pages from page tags", () => {
  assert.equal(isWikidotFragmentPage(["fragment"]), true)
  assert.equal(isWikidotFragmentPage(["scp", "fragment"]), true)
  assert.equal(isWikidotFragmentPage(["scp"]), false)
  assert.equal(isWikidotFragmentPage([]), false)
  assert.equal(isWikidotFragmentPage(null), false)
})
