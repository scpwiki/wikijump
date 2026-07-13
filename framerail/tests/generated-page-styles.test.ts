import assert from "node:assert/strict"
import test from "node:test"

import {
  buildGeneratedPageStylesHead,
  getCjkFontPreloadHref,
  getPageFontPreloadHrefs
} from "../src/lib/generated-page-styles.ts"

test("builds ordered head styles for multiple generated CSS modules", () => {
  assert.equal(
    buildGeneratedPageStylesHead([".first { color: red; }", ".second { color: blue; }"]),
    '<style type="text/css" data-wikijump-generated-css="0">.first { color: red; }</style><style type="text/css" data-wikijump-generated-css="1">.second { color: blue; }</style>'
  )
})

test("keeps CSS in raw-text context when content attempts to close or forge a style", () => {
  const html = buildGeneratedPageStylesHead([
    '</style><meta name="injected"><style data-wikijump-generated-css="99">'
  ])

  assert.equal(html.match(/<style/g)?.length, 1)
  assert.equal(html.match(/<\/style>/g)?.length, 1)
  assert(!html.includes('<meta name="injected">'))
  assert(html.includes('\\3C /style>\\3C meta name="injected">'))
  assert(html.includes('\\3C style data-wikijump-generated-css="99">'))
})

test("preloads the CJK font selected by the document locale when rendered content needs it", () => {
  assert.equal(
    getCjkFontPreloadHref("en", ["これは日本語です"]),
    "/fonts/variable/NotoSansJP-VF.woff2"
  )
  assert.equal(
    getCjkFontPreloadHref("zh-HK", ["繁體中文"]),
    "/fonts/variable/NotoSansHK-VF.woff2"
  )
  assert.equal(
    getCjkFontPreloadHref("zh-Hant", ["繁體中文"]),
    "/fonts/variable/NotoSansTC-VF.woff2"
  )
  assert.equal(
    getCjkFontPreloadHref("zh-Hans", ["简体中文"]),
    "/fonts/variable/NotoSansSC-VF.woff2"
  )
  assert.equal(
    getCjkFontPreloadHref("ko", ["漢字"]),
    "/fonts/variable/NotoSansKR-VF.woff2"
  )
})

test("does not preload a multi-megabyte CJK font without an ideograph", () => {
  assert.equal(getCjkFontPreloadHref("ja", ["ひらがな and Latin text"]), null)
  assert.equal(getCjkFontPreloadHref("en", [null, undefined, ""]), null)
})

test("detects supplementary-plane CJK ideographs", () => {
  assert.equal(getCjkFontPreloadHref("ja", ["𠀀"]), "/fonts/variable/NotoSansJP-VF.woff2")
})

test("preloads the two fonts used by every rendered article shell", () => {
  assert.deepEqual(getPageFontPreloadHrefs("en", "<p>Latin text</p>"), [
    "/fonts/variable/PublicSans-VariableFont.woff2",
    "/fonts/variable/RedHatDisplayVF.woff2"
  ])
})

test("preloads italic and monospace variants only when rendered markup uses them", () => {
  assert.deepEqual(
    getPageFontPreloadHrefs("en", "<p><em>emphasis</em> and <code>code</code></p>"),
    [
      "/fonts/variable/PublicSans-VariableFont.woff2",
      "/fonts/variable/RedHatDisplayVF.woff2",
      "/fonts/variable/PublicSans-Italic-VariableFont.woff2",
      "/fonts/variable/RedHatDisplayVF-Italic.woff2",
      "/fonts/variable/CascadiaMono.woff2",
      "/fonts/variable/CascadiaMonoItalic.woff2"
    ]
  )
})

test("places the locale-selected CJK font after smaller shell fonts", () => {
  assert.deepEqual(getPageFontPreloadHrefs("ja", "<p>日本語</p>"), [
    "/fonts/variable/PublicSans-VariableFont.woff2",
    "/fonts/variable/RedHatDisplayVF.woff2",
    "/fonts/variable/NotoSansJP-VF.woff2"
  ])
})
