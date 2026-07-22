import { strict as assert } from "node:assert"
import test from "node:test"

import {
  WIKIDOT_FOOTER_LINKS,
  WIKIDOT_POWERED_BY,
  buildWikidotFooterLinks,
  buildWikidotLicenseHtml,
  buildWikidotLoginLabels,
  formatWikidotLicenseName,
  isImportedWikidotView,
  shouldUseWikidotLicenseHtml
} from "../src/lib/wikidot-footer.js"

test("uses Wikidot footer labels in source order", () => {
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS.map((link) => link.label),
    ["Help", "Terms of Service", "Privacy", "Report a bug", "Flag as objectionable"]
  )
  assert.equal(WIKIDOT_POWERED_BY, "Powered by Wikidot.com")
})

test("uses Japanese Wikidot footer and login labels for Japanese imported sites", () => {
  assert.deepEqual(
    buildWikidotFooterLinks("ja").map((link) => link.label),
    ["ヘルプ", "利用規約", "プライバシー", "バグを報告", "不快フラグを立てる"]
  )
  assert.deepEqual(buildWikidotLoginLabels("ja"), {
    createAccount: "アカウントを作成",
    or: "または",
    signIn: "サインイン"
  })
})

test("formats Wikidot license wording without Wikijump copy", () => {
  assert.equal(
    formatWikidotLicenseName("Creative Commons Attribution-ShareAlike 3.0"),
    "Creative Commons Attribution-ShareAlike 3.0 License"
  )
  assert.equal(
    formatWikidotLicenseName("Creative Commons Attribution-ShareAlike 3.0 License"),
    "Creative Commons Attribution-ShareAlike 3.0 License"
  )
  assert.equal(
    buildWikidotLicenseHtml({
      licenseName: "Creative Commons Attribution-ShareAlike 3.0",
      licenseUrl: "https://example.invalid/license"
    }),
    'Unless otherwise stated, the content of this page is licensed under <a href="https://example.invalid/license">Creative Commons Attribution-ShareAlike 3.0 License</a>'
  )
})

test("renders trusted custom and empty copyright Wikidot license modes", () => {
  const html = 'Codex 2026 <strong>Strong</strong> <a href="/page">Local</a>'
  assert.equal(buildWikidotLicenseHtml({ licenseKind: "other", licenseHtml: html }), html)
  assert.equal(buildWikidotLicenseHtml({ licenseKind: "copyright" }), "")
  assert.equal(shouldUseWikidotLicenseHtml(false, "standard"), false)
  assert.equal(shouldUseWikidotLicenseHtml(false, "other"), true)
  assert.equal(shouldUseWikidotLicenseHtml(false, "copyright"), true)
  assert.equal(shouldUseWikidotLicenseHtml(true, "standard"), true)
})

test("formats Japanese Wikidot license wording for SCP-JP", () => {
  assert.equal(
    formatWikidotLicenseName("Creative Commons Attribution-ShareAlike 3.0", "ja"),
    "クリエイティブ・コモンズ 表示 - 継承3.0ライセンス"
  )
  assert.equal(
    buildWikidotLicenseHtml({
      licenseName: "Creative Commons Attribution-ShareAlike 3.0",
      licenseUrl: "https://example.invalid/license",
      locale: "ja"
    }),
    '特に指定がない限り、このサイトのすべてのコンテンツは<a href="https://example.invalid/license">クリエイティブ・コモンズ 表示 - 継承3.0ライセンス</a> の元で利用可能です。'
  )
})

test("formats Japanese non-SCP-JP Wikidot license wording from page scope", () => {
  assert.equal(
    buildWikidotLicenseHtml({
      licenseName: "Creative Commons Attribution-ShareAlike 3.0",
      licenseUrl: "https://example.invalid/license",
      locale: "ja",
      sourceSite: "scpaiueouiuiuiui"
    }),
    '特に明記しない限り、このページのコンテンツは次のライセンスの下にあります: <a href="https://example.invalid/license">Creative Commons Attribution-ShareAlike 3.0 License</a>'
  )
})

test("selects Wikidot footer branding only for imported Wikidot views", () => {
  assert.equal(isImportedWikidotView(null), false)
  assert.equal(isImportedWikidotView({ site: { from_wikidot: false } }), false)
  assert.equal(isImportedWikidotView({ site: { from_wikidot: true } }), true)
  assert.equal(isImportedWikidotView({ page: { from_wikidot: true } }), true)
  assert.equal(isImportedWikidotView({ page_revision: { from_wikidot: true } }), true)
})
