import { strict as assert } from "node:assert"
import test from "node:test"

import {
  WIKIDOT_FOOTER_LINKS,
  WIKIDOT_FOOTER_LINKS_JA,
  WIKIDOT_POWERED_BY,
  buildWikidotAccountLabels,
  buildWikidotFooterLinks,
  buildWikidotLicenseHtml,
  buildWikidotLoginLabels,
  formatWikidotLicenseName,
  isImportedWikidotView,
  shouldUseWikidotLicenseHtml
} from "../src/lib/wikidot/wikidot-footer.js"

test("uses Wikidot footer labels in source order", () => {
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS.map((link) => link.label),
    ["Help", "Terms of Service", "Privacy", "Report a bug", "Flag as objectionable"]
  )
  assert.equal(WIKIDOT_POWERED_BY, "Powered by Wikidot.com")
})

test("binds every Wikidot footer item to its live Wikidot destination", () => {
  // Frozen evidence: live Wikidot footer, anonymous page on sandbox-for-codex,
  // retained capture 2026-09-15; live DOM for SCP-Wiki and SCP-JP scp-173,
  // captured 2026-09-29. Help, Terms of Service and Privacy are absolute
  // www.wikidot.com URLs; the report controls are inert anchors whose
  // listeners request the in-page report modules.
  const expected = [
    ["Help", "https://www.wikidot.com/doc"],
    ["Terms of Service", "https://www.wikidot.com/legal:terms-of-service"],
    ["Privacy", "https://www.wikidot.com/legal:privacy-policy"],
    ["Report a bug", "javascript:;"],
    ["Flag as objectionable", "javascript:;"]
  ]
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS.map((link) => [link.label, link.href]),
    expected
  )
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS_JA.map((link) => [link.label, link.href]),
    [
      ["ヘルプ", "https://www.wikidot.com/doc"],
      ["利用規約", "https://www.wikidot.com/legal:terms-of-service"],
      ["プライバシー", "https://www.wikidot.com/legal:privacy-policy"],
      ["バグを報告", "javascript:;"],
      ["不快フラグを立てる", "javascript:;"]
    ]
  )
})

test("keeps Wikidot footer destinations distinct and off the current site", () => {
  for (const link of [...WIKIDOT_FOOTER_LINKS, ...WIKIDOT_FOOTER_LINKS_JA]) {
    assert.notEqual(link.href, "/", `${link.label} must not point at the site root`)
    assert.doesNotMatch(
      link.href,
      /^\/[^/]/u,
      `${link.label} must not be a same-site page path`
    )
  }

  // No advertised action is collapsed onto one generic homepage: the three
  // navigating items hold three distinct Wikidot destinations, and the two
  // inert report controls are inert anchors on Wikidot too.
  const navigating = WIKIDOT_FOOTER_LINKS.filter((link) => link.href !== "javascript:;")
  assert.equal(navigating.length, 3)
  assert.equal(
    new Set(navigating.map((link) => link.href)).size,
    navigating.length,
    "distinct labels must not share one destination"
  )
})

test("uses Wikidot footer labels in source order for Japanese imported sites", () => {
  assert.deepEqual(
    buildWikidotFooterLinks("ja").map((link) => link.label),
    ["ヘルプ", "利用規約", "プライバシー", "バグを報告", "不快フラグを立てる"]
  )
  assert.deepEqual(buildWikidotLoginLabels("ja"), {
    createAccount: "アカウントを作成",
    or: "または",
    signIn: "サインイン"
  })
  assert.deepEqual(buildWikidotAccountLabels("ja"), {
    myAccount: "マイアカウント",
    settings: "設定",
    signOut: "サインアウト"
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
