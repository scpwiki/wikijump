import { strict as assert } from "node:assert"
import test from "node:test"

import { readFileSync } from "node:fs"

import {
  WIKIDOT_FOOTER_LINKS,
  WIKIDOT_FOOTER_LINKS_JA,
  WIKIDOT_POWERED_BY,
  buildWikidotAccountLabels,
  buildWikidotFooterHtml,
  buildWikidotFooterLinks,
  buildWikidotLicenseHtml,
  buildWikidotLoginLabels,
  formatWikidotLicenseName,
  isImportedWikidotView,
  shouldUseWikidotLicenseHtml,
  wikidotFooterControls
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
    '\n                    Unless otherwise stated, the content of this page is licensed under\n\n <a rel="license" href="https://example.invalid/license">Creative Commons Attribution-ShareAlike 3.0 License</a>\n                '
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
    '\n                    \n\n特に指定がない限り、このサイトのすべてのコンテンツは<a rel="license" href="https://example.invalid/license">クリエイティブ・コモンズ 表示 - 継承3.0ライセンス</a> の元で利用可能です。\n                '
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

test("carries Wikidot's footer control ids for their hovertips", () => {
  const ids = [
    "wikidot-help-button",
    "wikidot-tos-button",
    "wikidot-privacy-button",
    "bug-report-button",
    "abuse-report-button"
  ]
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS.map((link) => link.id),
    ids
  )
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS_JA.map((link) => link.id),
    ids
  )
})

// Retained live DOM of the standard Wikidot shells (`scp-173`, 2026-09-29).
// The served `#footer` and `#license-area` come straight from Wikidot's
// server template; only the footer's nitroAds consent script is left out.
const retainedShell = (directory) =>
  readFileSync(
    new URL(
      `../../install/local/theme-lab/evidence/wikidot-runtime-surface-20260929.normal/${directory}/live.dom.html`,
      import.meta.url
    ),
    "utf8"
  )
const innerHtml = (html, open, close) => {
  const start = html.indexOf(open) + open.length
  return html.slice(start, html.indexOf(close, start))
}

for (const [locale, directory] of [
  ["en", "scp-wiki-scp-173-normal-20260929-5d64b862fb46"],
  ["ja", "scp-jp-scp-173-normal-20260929-7984e750c954"]
]) {
  test(`reproduces the standard Wikidot footer and license area (${locale})`, () => {
    const html = retainedShell(directory)
    const footer = innerHtml(
      html,
      '<div id="footer" style="display: block; visibility: visible;">',
      '</div>\n            \n                <div id="license-area"'
    ).replace(/<script>[\s\S]*?<\/script>/u, "")
    assert.equal(buildWikidotFooterHtml(locale), footer)

    const license = innerHtml(
      html,
      '<div id="license-area" class="license-area">',
      "</div>"
    )
    assert.equal(
      buildWikidotLicenseHtml({
        licenseName: "Creative Commons Attribution-ShareAlike 3.0",
        licenseUrl: "https://creativecommons.org/licenses/by-sa/3.0/",
        locale
      }),
      license
    )
  })
}

test("binds the footer's inline handler names without compiling them", () => {
  let handlerReads = 0
  const control = (onclick) => {
    let handler = null
    return {
      get onclick() {
        handlerReads += 1
        return handler
      },
      set onclick(value) {
        handler = value
      },
      getAttribute: (name) => (name === "onclick" ? onclick : null)
    }
  }
  const report = control("WIKIDOT.page.listeners.pageBugReport(event)")
  const consent = control("window.__cmp('showModal');")
  const authored = control("alert(1)")
  const action = wikidotFooterControls({
    querySelectorAll: (selector) =>
      selector === "a[onclick]" ? [report, consent, authored] : []
  })
  assert.equal(handlerReads, 0)
  assert.equal(report.onclick(), false)
  assert.equal(consent.onclick(), false)
  assert.equal(authored.onclick, null)
  action.destroy()
  assert.equal(report.onclick, null)
  assert.equal(consent.onclick, null)
})

test("reproduces the anonymous EN/JA BugReportModule bodies", async () => {
  const { buildWikidotBugReportDialogHtml } =
    await import("../src/lib/wikidot/wikidot-footer.js")
  const fixture = JSON.parse(
    await (
      await import("node:fs/promises")
    ).readFile(new URL("./fixtures/wikidot-footer-dialogs.json", import.meta.url), "utf8")
  )
  for (const locale of ["en", "ja"])
    assert.equal(
      buildWikidotBugReportDialogHtml(locale),
      fixture.responses[locale]["report/BugReportModule"].body
    )
  assert.equal(
    buildWikidotBugReportDialogHtml("ja-corrections"),
    fixture.responses.ja["report/BugReportModule"].body
  )
})
