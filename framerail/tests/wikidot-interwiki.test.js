import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotInterwikiFrameHtml,
  extractWikidotInterwikiLinks
} from "../src/lib/wikidot-interwiki.js"
import {
  buildWikidotStyleFrameHtml,
  isUsableStyleFrameCss,
  localizeWikidotThemeUrl
} from "../src/lib/wikidot-styleframe.js"

const cromPage = {
  translations: [
    { url: "http://scp-wiki-cn.wikidot.com/1231-warning" },
    { url: "https://fondationscp.wikidot.com/1231-warning" },
    { url: "https://scp-jp.wikidot.com/1231-warning" },
    { url: "https://scpko.wikidot.com/1231-warning" },
    { url: "https://scpfoundation.net/1231-warning" },
    { url: "https://scp-vn.wikidot.com/1231-warning" }
  ],
  translationOf: null
}

test("builds SCP interwiki language links from Crom translations", () => {
  assert.deepEqual(
    extractWikidotInterwikiLinks({
      community: "scp",
      lang: "en",
      sourcePath: "1231-warning",
      page: cromPage
    }).map((link) => link.label),
    ["中文", "Français", "日本語", "한국어", "Русский", "Tiếng Việt"]
  )
})

test("renders Wikidot-compatible interwiki visible text for translated SCP pages", () => {
  const html = buildWikidotInterwikiFrameHtml({
    community: "scp",
    lang: "en",
    pagename: "1231-warning",
    page: cromPage
  })

  assert.match(html, /In other languages/)
  assert.doesNotMatch(html, /IN OTHER LANGUAGES/)
  assert.match(html, /中文<\/a><\/div> <div class="menu-item" name="fr"/)
  assert.match(html, /中文/)
  assert.match(html, /Français/)
  assert.match(html, /日本語/)
  assert.match(html, /한국어/)
  assert.match(html, /Русский/)
  assert.match(html, /Tiếng Việt/)
  assert.doesNotMatch(html, /English/)
})

test("builds styleFrame parent injection for theme stylesheets", () => {
  const html = buildWikidotStyleFrameHtml({
    priority: "2",
    themes: [
      "https://cdn.scpwiki.com/theme/en/basalt/basalt-bedrock-min.css",
      "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1"
    ],
    css: "{$css}",
    origin: "https://scp-wiki.wikijump.localhost"
  })

  assert.match(html, /wikidot-style-theme-count" content="2"/)
  assert.match(html, /window\.parent\.document/)
  assert.match(html, /head\.insertBefore\(element, laterStyle\)/)
  assert.match(html, /restoreStyleFrameOrder/)
  assert.match(html, /head\.appendChild\(node\)/)
  assert.match(html, /cdn\.scpwiki\.com\/theme\/en\/basalt\/basalt-bedrock-min\.css/)
  assert.match(html, /scp-wiki\.wjfiles\.localhost\/local--code\/theme%3Abasalt\/1/)
  assert.doesNotMatch(html, /<style>\{\$css\}<\/style>/)
  assert.doesNotMatch(html, /<style>\$css<\/style>/)
})

test("keeps non-placeholder styleFrame inline CSS safe", () => {
  const html = buildWikidotStyleFrameHtml({
    css: "body::before { content: '</style>'; }"
  })

  assert.equal(isUsableStyleFrameCss("{$css}"), false)
  assert.equal(isUsableStyleFrameCss("$css"), false)
  assert.equal(isUsableStyleFrameCss(" body { color: red } "), true)
  assert.match(html, /<style>body::before \{ content: '<\\\/style>'; \}<\/style>/)
  assert.match(html, /const css = "body::before \{ content: '\\u003c/)
  assert.doesNotMatch(html, /<\/script>.*<\/script>/s)
})

test("localizes Wikidot local file and code theme URLs to the local file host", () => {
  assert.equal(
    localizeWikidotThemeUrl(
      "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://scp-wiki.wjfiles.localhost/local--code/theme%3Abasalt/1"
  )
  assert.equal(
    localizeWikidotThemeUrl(
      "https://scp-wiki.wdfiles.com/local--code/theme:basalt/1",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://scp-wiki.wjfiles.localhost/local--code/theme:basalt/1"
  )
  assert.equal(
    localizeWikidotThemeUrl(
      "https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css",
      "https://scp-wiki.wikijump.localhost"
    ),
    "https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css"
  )
})
