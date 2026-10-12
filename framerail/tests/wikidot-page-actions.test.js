import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotPageActionLabels,
  formatSigned,
  isWikidotFragmentPage,
  openWikidotPrinterFriendly,
  requestWikidotPageDiscussion,
  sourceShowsStandardWikidotPageActions,
  wikidotPageActionVisibility,
  wikidotPageDiscussHref,
  wikidotPageDiscussionPath
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

test("page discussion requests use the frozen Wikidot ForumAction form", async () => {
  const calls = []
  const result = await requestWikidotPageDiscussion(async (url, options) => {
    calls.push({ url, options })
    return {
      json: async () => ({
        status: "ok",
        thread_id: 18232631,
        thread_unix_title: "lane-4-discussion"
      })
    }
  }, 1469071756)

  assert.equal(calls[0].url, "/ajax-module-connector.php")
  assert.equal(calls[0].options.method, "POST")
  assert.equal(calls[0].options.credentials, "same-origin")
  assert.equal(
    calls[0].options.headers["content-type"],
    "application/x-www-form-urlencoded"
  )
  assert.deepEqual(
    new URLSearchParams(calls[0].options.body),
    new URLSearchParams({
      moduleName: "Empty",
      page_id: "1469071756",
      action: "ForumAction",
      event: "createPageDiscussionThread"
    })
  )
  assert.deepEqual(result, {
    status: "ok",
    thread_id: 18232631,
    thread_unix_title: "lane-4-discussion"
  })
  assert.equal(wikidotPageDiscussionPath(result), "/forum/t-18232631/lane-4-discussion")
})

test("invalid Wikidot discussion responses do not create navigation paths", () => {
  assert.equal(wikidotPageDiscussionPath({ status: "no_page" }), null)
  assert.equal(
    wikidotPageDiscussionPath({
      status: "not_ok",
      thread_id: 18232631,
      thread_unix_title: "lane-4-discussion"
    }),
    null
  )
  assert.equal(
    wikidotPageDiscussionPath({
      status: "ok",
      thread_id: 0,
      thread_unix_title: "missing"
    }),
    null
  )
})

test("page discussion requests reject invalid page IDs", async () => {
  await assert.rejects(
    requestWikidotPageDiscussion(async () => assert.fail("must not request"), 0),
    /positive page ID/u
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

test("Discuss links straight to an existing thread by the page name, as Wikidot does", () => {
  assert.equal(
    wikidotPageDiscussHref({ discussionThreadId: 17768232, pageSlug: "scp-9506" }),
    "/forum/t-17768232/scp-9506"
  )
  assert.equal(
    wikidotPageDiscussHref({ discussionThreadId: 102848, pageSlug: "forum:start" }),
    "/forum/t-102848/forum:start"
  )
})

test("Discuss keeps the create-thread action without a thread or a Wikidot page name", () => {
  for (const input of [
    { discussionThreadId: null, pageSlug: "scp-9506" },
    { discussionThreadId: 0, pageSlug: "scp-9506" },
    { discussionThreadId: 1.5, pageSlug: "scp-9506" },
    { discussionThreadId: 17768232, pageSlug: "" },
    { discussionThreadId: 17768232, pageSlug: "../admin" },
    { discussionThreadId: 17768232, pageSlug: "a:b:c" },
    { discussionThreadId: 17768232, pageSlug: "SCP-9506" },
    { discussionThreadId: 17768232, pageSlug: null }
  ]) {
    assert.equal(wikidotPageDiscussHref(input), null, JSON.stringify(input))
  }
})
