import assert from "node:assert/strict"
import { describe, it } from "node:test"

import {
  editPermissionDeniedDialog,
  editPermissionDeniedMessage,
  isEnglishWikidotLocale,
  isJapaneseWikidotLocale,
  toIntlLocales
} from "../src/lib/wikidot/wikidot-locale.js"

describe("Wikidot locale compatibility", () => {
  it("recognizes Wikidot Japanese locale identifiers", () => {
    assert.equal(isJapaneseWikidotLocale("ja-corrections"), true)
    assert.equal(isJapaneseWikidotLocale("ja_JP"), true)
    assert.equal(isJapaneseWikidotLocale("en"), false)
  })

  it("maps ja-corrections only at the Intl boundary", () => {
    const wikidotLocales = ["en-US", "ja-corrections", "en"]

    assert.deepEqual(toIntlLocales(wikidotLocales), ["en-US", "ja", "en"])
    assert.deepEqual(wikidotLocales, ["en-US", "ja-corrections", "en"])
    assert.doesNotThrow(() =>
      new Date("2026-07-22T00:00:00Z").toLocaleString(toIntlLocales(wikidotLocales))
    )
  })

  it("recognizes Wikidot English locale identifiers", () => {
    assert.equal(isEnglishWikidotLocale("en"), true)
    assert.equal(isEnglishWikidotLocale("en_US"), true)
    assert.equal(isEnglishWikidotLocale("ja"), false)
    assert.equal(isEnglishWikidotLocale("ko-KR"), false)
    assert.equal(isEnglishWikidotLocale(null), false)
  })

  it("returns the retained Wikidot denial dialog only for English locales", () => {
    assert.deepEqual(editPermissionDeniedDialog("en"), {
      title: "Permission error",
      message: "Please note:",
      extraMessage:
        "Sorry, you can not edit this page. Only members of this site, owner (creator) of this page, site administrators and perhaps selected moderators are allowed to do it.",
      actionLink: { href: "#action:login", label: "Sign in as Wikidot user" },
      inlineActionLink: true
    })
    assert.deepEqual(
      editPermissionDeniedDialog("en-US"),
      editPermissionDeniedDialog("en")
    )
    assert.equal(editPermissionDeniedDialog("ja"), null)
    assert.equal(editPermissionDeniedDialog("ko-KR"), null)
    assert.equal(editPermissionDeniedDialog("zh-TW"), null)
    assert.equal(editPermissionDeniedDialog(null), null)
  })

  it("localizes the page edit permission error for the EN/CN/KO/JP theme branches", () => {
    assert.equal(
      editPermissionDeniedMessage("ja-corrections"),
      "このページを編集する権限がありません。"
    )
    assert.equal(
      editPermissionDeniedMessage("ko-KR"),
      "이 페이지를 편집할 권한이 없습니다."
    )
    assert.equal(editPermissionDeniedMessage("zh-CN"), "您没有权限编辑此页面。")
    assert.equal(editPermissionDeniedMessage("zh-TW"), "您沒有權限編輯此頁面。")
    assert.equal(
      editPermissionDeniedMessage("en"),
      "You don't have permission to edit this page."
    )
  })
})
