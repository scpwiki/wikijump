import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildWikidotInterwikiFrameHtml,
  extractWikidotInterwikiLinks
} from "../src/lib/wikidot-interwiki.js"

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

  assert.match(html, /IN OTHER LANGUAGES/)
  assert.match(html, /中文<\/a><\/div> <div class="menu-item" name="fr"/)
  assert.match(html, /中文/)
  assert.match(html, /Français/)
  assert.match(html, /日本語/)
  assert.match(html, /한국어/)
  assert.match(html, /Русский/)
  assert.match(html, /Tiếng Việt/)
  assert.doesNotMatch(html, /English/)
})
