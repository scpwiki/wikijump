import { strict as assert } from "node:assert"
import test from "node:test"

import { parseAcceptLangHeader, withFallbackLocale } from "../src/lib/locales.js"
import { buildPageLoadData } from "../src/lib/server/load/page-data.js"

const requestWithAcceptLanguage = (value) => {
  return new Request("https://scp-wiki.example/", {
    headers: value === null ? {} : { "Accept-Language": value }
  })
}

test("Accept-Language parsing ignores wildcards and preserves ordered locales", () => {
  assert.deepEqual(parseAcceptLangHeader(requestWithAcceptLanguage(null)), [])
  assert.deepEqual(
    parseAcceptLangHeader(requestWithAcceptLanguage("fr-CA, fr;q=0.8, *, en;q=0.7")),
    ["fr-CA", "fr", "en"]
  )
  assert.deepEqual(parseAcceptLangHeader(requestWithAcceptLanguage("*")), [])
  assert.deepEqual(
    parseAcceptLangHeader(
      requestWithAcceptLanguage("en-US, en;q=0.9, *;q=0.1, en-US;q=0.5")
    ),
    ["en-US", "en"]
  )
})

test("fallback locale is appended only when missing", () => {
  assert.deepEqual(withFallbackLocale([]), ["en"])
  assert.deepEqual(withFallbackLocale(["ja-JP"]), ["ja-JP", "en"])
  assert.deepEqual(withFallbackLocale(["en"]), ["en"])
  assert.deepEqual(withFallbackLocale(["ja-JP", "en"]), ["ja-JP", "en"])
})

test("article route data includes preload data and page-specific data", () => {
  const parentData = {
    site: { name: "SCP Wiki", locale: "en" },
    site_file_domain: "files.example",
    license_name: "CC BY-SA 3.0",
    license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
    user_session: null,
    locales: ["en-US", "en"]
  }
  const viewData = {
    view: "found",
    compiled_body_html: "<p>body</p>",
    wikidot_page_watch: null
  }
  const forms = { pageEditForm: { valid: true } }

  assert.deepEqual(buildPageLoadData(parentData, viewData, forms), {
    ...parentData,
    ...viewData,
    forms
  })
})
