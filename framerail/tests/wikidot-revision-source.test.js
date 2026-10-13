import assert from "node:assert/strict"
import crypto from "node:crypto"
import fs from "node:fs"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"
import {
  wikidotPageSourceHtml,
  wikidotRevisionSourceHtml
} from "../src/lib/wikidot-history-contract.js"

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..")
const evidence = "framerail/tests/fixtures/native-revision-source"

test("frozen revision source proves the div and explicit escaped line-boundary contract", () => {
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, evidence + "/observation.json"))
  )
  assert.equal(receipt.public_writes, 0)
  for (const site of receipt.sites) {
    const binding = site.requests[0]
    const bytes = fs.readFileSync(path.join(root, evidence, binding.response_file))
    assert.equal(
      crypto.createHash("sha256").update(bytes).digest("hex"),
      binding.response_sha256
    )
    const response = JSON.parse(bytes)
    assert.equal(response.status, "ok")
    assert.match(response.body, /<div class="page-source">/u)
    assert.match(response.body, /<br \/>/u)
    assert.doesNotMatch(response.body, /<textarea/u)
  }
})

test("historical source remains inert and preserves Japanese lines, blank lines and CRLF", () => {
  assert.equal(
    wikidotRevisionSourceHtml("日本語\r\n\r\n<img src=x onerror=\"alert(1)\"> & 'text'"),
    '<div class="page-source">日本語<br />\n<br />\n&lt;img src=x onerror=&quot;alert(1)&quot;&gt; &amp; &#039;text&#039;</div>'
  )
  assert.equal(wikidotRevisionSourceHtml(""), '<div class="page-source"></div>')
})

test("current page source links only a standalone same-site component include", () => {
  const source = [
    "before & <safe>",
    "[[include component:image-block]]",
    "[[include component:parameterized",
    "|title=Safe",
    "]]",
    "[[include component:unterminated",
    "[[include component:bad-parameter",
    "|not a parameter",
    "]]",
    "[[include :scp-wiki:component:license-box]]",
    "[[include component:bad%2fpath]]",
    " [[include component:spaced-literal]]",
    "@@[[include component:literal]]@@",
    "[!--",
    "[[include component:commented]]",
    "--]",
    "[[code]]",
    "[[include component:example]]",
    "[[/code]]",
    '<img src=x onerror="alert(1)">'
  ].join("\n")

  const html = wikidotPageSourceHtml(source)
  assert.equal(
    html,
    [
      "before &amp; &lt;safe&gt;",
      '[[include <a href="/component%3Aimage-block">component:image-block</a>]]',
      '[[include <a href="/component%3Aparameterized">component:parameterized</a>',
      "|title=Safe",
      "]]",
      "[[include component:unterminated",
      "[[include component:bad-parameter",
      "|not a parameter",
      "]]",
      '[[include <a href="https://scp-wiki.wikidot.com/component:license-box">:scp-wiki:component:license-box</a>]]',
      "[[include component:bad%2fpath]]",
      " [[include component:spaced-literal]]",
      "@@[[include component:literal]]@@",
      "[!--",
      "[[include component:commented]]",
      "--]",
      "[[code]]",
      "[[include component:example]]",
      "[[/code]]",
      "&lt;img src=x onerror=&quot;alert(1)&quot;&gt;"
    ].join("\n")
  )

  const textContent = html
    .replace(/<a href="[^"]+">/gu, "")
    .replace(/<\/a>/gu, "")
    .replaceAll("&amp;", "&")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#039;", "'")
  assert.equal(textContent, source)
})

test("current page source links cross-site and parameterized include targets", () => {
  // Forms observed in the shared corpus sources for scp-3352 (11 includes)
  // and scp-5382 (21 includes); live Wikidot linkifies each target in its
  // read-only source view, including the absolute-URL form
  // [[include <a href="https://scp-wiki.wikidot.com/info:start">:scp-wiki:info:start</a>]].
  const source = [
    "[[include :scp-wiki:info:start]]",
    "[[include :scpsandbox2:hippo]]",
    "[[include :scp-wiki:component:classified-decoration",
    "|lv=2",
    "|item=3352",
    "]]",
    "[[include component:image-block",
    "    name=Eye.jpg|",
    '    caption=Stage 1. [[span class="fnnum"]]. |',
    "    width=150px|]]",
    "[[include :scp-wiki:component:customizable-acs",
    "|item-number= SCP-5382",
    "|inc-custom-text= --]]]",
    "[[include component:image-block name=steel.png|align=center|caption=Fig 1.1]]",
    "[[include :scp-wiki:theme:basalt:]]",
    "[[include :custom.domain:page]]",
    "[[include :scp-wiki:]]",
    "[[include]]",
    "[[include |title=No target]]",
    '[[include component:x]]<img src=x onerror="alert(1)">',
    "[[include javascript:alert(1)]]"
  ].join("\n")

  const html = wikidotPageSourceHtml(source)
  assert.equal(
    html,
    [
      '[[include <a href="https://scp-wiki.wikidot.com/info:start">:scp-wiki:info:start</a>]]',
      '[[include <a href="https://scpsandbox2.wikidot.com/hippo">:scpsandbox2:hippo</a>]]',
      '[[include <a href="https://scp-wiki.wikidot.com/component:classified-decoration">:scp-wiki:component:classified-decoration</a>',
      "|lv=2",
      "|item=3352",
      "]]",
      '[[include <a href="/component%3Aimage-block">component:image-block</a>',
      "    name=Eye.jpg|",
      "    caption=Stage 1. [[span class=&quot;fnnum&quot;]]. |",
      "    width=150px|]]",
      '[[include <a href="https://scp-wiki.wikidot.com/component:customizable-acs">:scp-wiki:component:customizable-acs</a>',
      "|item-number= SCP-5382",
      "|inc-custom-text= --]]]",
      '[[include <a href="/component%3Aimage-block">component:image-block</a> name=steel.png|align=center|caption=Fig 1.1]]',
      "[[include :scp-wiki:theme:basalt:]]",
      "[[include :custom.domain:page]]",
      "[[include :scp-wiki:]]",
      "[[include]]",
      "[[include |title=No target]]",
      '[[include <a href="/component%3Ax">component:x</a>]]&lt;img src=x onerror=&quot;alert(1)&quot;&gt;',
      "[[include javascript:alert(1)]]"
    ].join("\n")
  )

  assert.doesNotMatch(html, /href="javascript/u)
  assert.equal(
    html
      .replace(/<a href="[^"]+">/gu, "")
      .replace(/<\/a>/gu, "")
      .replaceAll("&amp;", "&")
      .replaceAll("&lt;", "<")
      .replaceAll("&gt;", ">")
      .replaceAll("&quot;", '"')
      .replaceAll("&#039;", "'"),
    source
  )
})
