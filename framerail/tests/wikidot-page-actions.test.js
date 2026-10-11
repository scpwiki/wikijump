import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotDiscussButtonHtml,
  buildWikidotPageActionLabels,
  formatSigned,
  isWikidotFragmentPage,
  openWikidotPrinterFriendly,
  sourceShowsStandardWikidotPageActions,
  wikidotPageActionVisibility
} from "../src/lib/wikidot/wikidot-page-actions.js"

test("Print opens the live printer-friendly child window without printing the opener", () => {
  const calls = []
  let printed = 0
  openWikidotPrinterFriendly("/doc-wiki-syntax:buttons", {
    open: (url, target) => calls.push([url, target]),
    print: () => (printed += 1)
  })
  assert.deepEqual(calls, [["/printer--friendly//doc-wiki-syntax:buttons", "_blank"]])
  assert.equal(printed, 0)
})

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
    options: "Options",
    append: "Append",
    backlinks: "Backlinks"
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
    options: "Options",
    append: "Append",
    backlinks: "Backlinks"
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
      options: "オプション",
      append: "追加",
      backlinks: "バックリンク"
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
      options: "Options",
      append: "Append",
      backlinks: "Backlinks"
    }
  )
})

test("disabled page discussions stay hidden even when an old thread pointer exists", () => {
  assert.deepEqual(
    wikidotPageActionVisibility({
      sourceSite: "scp-wiki",
      ratingEnabled: true,
      discussionEnabled: false,
      discussionThreadId: 42
    }),
    { showRate: true, showDiscuss: false }
  )
})

test("an imported source comment count keeps its discussion action visible", () => {
  assert.deepEqual(
    wikidotPageActionVisibility({
      sourceSite: "scp-wiki",
      ratingEnabled: true,
      discussionEnabled: false,
      sourceCommentCount: 95
    }),
    { showRate: true, showDiscuss: true }
  )
})

test("source sites without standard actions stay hidden despite imported comments", () => {
  assert.deepEqual(
    wikidotPageActionVisibility({
      sourceSite: "sandbox-for-codex",
      ratingEnabled: true,
      discussionEnabled: false,
      sourceCommentCount: 95
    }),
    { showRate: false, showDiscuss: false }
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
