import assert from "node:assert/strict"
import fs from "node:fs/promises"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"
import vm from "node:vm"

import {
  WIKIDOT_REQUEST_INFO_MARKER,
  buildWikidotRequestInfo,
  injectWikidotRequestInfo,
  requestHostFromRequest,
  serializeWikidotRequestInfo
} from "../src/lib/server/wikidot-request-info.js"

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")

const input = {
  domain: "scp-wiki.wikijump.localhost",
  site: {
    site_id: 6000006,
    slug: "scp-wiki",
    locale: "en",
    from_wikidot: true
  },
  page: {
    page_id: 3000011662,
    page_category_id: 100000053,
    slug: "scp-173"
  }
}

test("builds source-compatible Wikidot request metadata from local identities", () => {
  assert.deepEqual(buildWikidotRequestInfo(input), {
    domain: "scp-wiki.wikijump.localhost",
    siteId: 6000006,
    siteUnixName: "scp-wiki",
    categoryId: 100000053,
    requestPageName: "scp-173",
    lang: "en",
    pageUnixName: "scp-173",
    pageId: 3000011662
  })
})

test("binds a nonstandard local port into the client-visible request host", () => {
  assert.equal(
    buildWikidotRequestInfo({
      ...input,
      domain: "127.0.0.1:3405"
    }).domain,
    "127.0.0.1:3405"
  )
})

test("prefers the HTTP Host header over the adapter origin", () => {
  assert.equal(
    requestHostFromRequest(
      new Request("http://127.0.0.1:34091/scp-173", {
        headers: { host: "SCP-WIKI.WIKIDOT.COM" }
      })
    ),
    "scp-wiki.wikidot.com"
  )
  assert.equal(
    requestHostFromRequest(new Request("http://127.0.0.1:34091/scp-173")),
    "127.0.0.1:34091"
  )
})

test("serializes the exact raw statements consumed by wikidot.py", () => {
  const script = serializeWikidotRequestInfo(buildWikidotRequestInfo(input))
  assert.match(script, /^var WIKIREQUEST = \{\};/u)
  assert.match(script, /WIKIREQUEST\.info\.domain = "scp-wiki\.wikijump\.localhost";/u)
  assert.match(script, /WIKIREQUEST\.info\.siteId = 6000006;/u)
  assert.match(script, /WIKIREQUEST\.info\.siteUnixName = "scp-wiki";/u)
  assert.match(script, /WIKIREQUEST\.info\.pageId = 3000011662;/u)

  const context = {}
  vm.runInNewContext(script, context)
  assert.deepEqual({ ...context.WIKIREQUEST.info }, buildWikidotRequestInfo(input))
})

test("escapes inline script terminators and rejects malformed identities", () => {
  const safe = buildWikidotRequestInfo({
    ...input,
    page: { ...input.page, slug: "category:</script><script>alert(1)</script>" }
  })
  const script = serializeWikidotRequestInfo(safe)
  assert.doesNotMatch(script, /<\/script/iu)
  assert.match(script, /\\u003c\/script/u)

  assert.throws(
    () => buildWikidotRequestInfo({ ...input, domain: "example.com/path" }),
    /host grammar/u
  )
  assert.throws(
    () =>
      buildWikidotRequestInfo({ ...input, site: { ...input.site, slug: "scp.wiki" } }),
    /unix-name grammar/u
  )
  assert.throws(
    () => buildWikidotRequestInfo({ ...input, page: { ...input.page, page_id: 1.5 } }),
    /safe integer/u
  )
})

test("injects one marker and leaves non-HTML chunks unchanged", () => {
  const info = buildWikidotRequestInfo(input)
  const html = `<html lang="en"><head><script>${WIKIDOT_REQUEST_INFO_MARKER}</script></head></html>`
  const injected = injectWikidotRequestInfo(html, info, input.site.locale)
  assert.doesNotMatch(injected, /__WIKIDOT_REQUEST_INFO__/u)
  assert.match(injected, /WIKIREQUEST\.info\.pageId/u)
  assert.match(injected, /<html lang="en">/u)
  assert.equal(injectWikidotRequestInfo("plain text", info), "plain text")
  assert.throws(
    () =>
      injectWikidotRequestInfo(
        `${WIKIDOT_REQUEST_INFO_MARKER}${WIKIDOT_REQUEST_INFO_MARKER}`,
        info
      ),
    /at most once/u
  )
})

test("injects the raw Wikidot site locale as the document language independently of page metadata", () => {
  const info = buildWikidotRequestInfo({
    ...input,
    site: { ...input.site, locale: "ja-corrections" }
  })
  const html = `<html lang="en"><head><script>${WIKIDOT_REQUEST_INFO_MARKER}</script></head></html>`

  assert.match(
    injectWikidotRequestInfo(html, info, "ja-corrections"),
    /<html lang="ja-corrections">/u
  )
  assert.match(
    injectWikidotRequestInfo(html, undefined, "ja-corrections"),
    /<html lang="ja-corrections">/u
  )
  assert.doesNotMatch(
    injectWikidotRequestInfo(html, undefined, "ja-corrections"),
    /WIKIREQUEST\.info/u
  )
})

test("the app template binds the compatibility script to SvelteKit's CSP nonce", async () => {
  const template = await fs.readFile(path.join(ROOT, "src/app.html"), "utf8")
  assert.match(
    template,
    /<script nonce="%sveltekit\.nonce%">\s*\/\*__WIKIDOT_REQUEST_INFO__\*\/\s*<\/script>/u
  )
})

test("the app template preserves Wikidot's limited-quirks document mode", async () => {
  const template = await fs.readFile(path.join(ROOT, "src/app.html"), "utf8")
  assert.match(
    template,
    /^<!DOCTYPE html PUBLIC "-\/\/W3C\/\/DTD XHTML 1\.0 Transitional\/\/EN" "http:\/\/www\.w3\.org\/TR\/xhtml1\/DTD\/xhtml1-transitional\.dtd">\n/u
  )
})

test("the app template preserves Wikidot's body shell without an extra wrapper", async () => {
  const template = await fs.readFile(path.join(ROOT, "src/app.html"), "utf8")
  assert.match(template, /<body id="html-body">\s*%sveltekit\.body%\s*<\/body>/u)
  assert.doesNotMatch(template, /<body[^>]*>\s*<div>%sveltekit\.body%<\/div>/u)
})
