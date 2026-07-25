import assert from "node:assert/strict"
import test from "node:test"

import {
  resolveCanonicalViewData,
  resolveCanonicalViewMetadata
} from "../src/lib/view-data-decision.js"

const fixture = (label) => ({
  site: { locale: `${label}-locale` },
  license_name: `${label}-license`,
  license_url: `https://${label}.invalid/license`,
  license_kind: `${label}-kind`,
  license_html: `<span>${label}-license</span>`,
  wikidot_snapshot: { source_site: `${label}-source` }
})

test("error data consistently wins over retained page data", () => {
  const errorData = fixture("error")
  const pageData = fixture("page")
  assert.equal(resolveCanonicalViewData(errorData, pageData), errorData)
  assert.deepEqual(resolveCanonicalViewMetadata(errorData, pageData), {
    viewData: errorData,
    locale: "error-locale",
    licenseName: "error-license",
    licenseUrl: "https://error.invalid/license",
    licenseKind: "error-kind",
    licenseHtml: "<span>error-license</span>",
    sourceSite: "error-source"
  })
})

test("page data is used when no error data exists", () => {
  const pageData = fixture("page")
  assert.equal(resolveCanonicalViewData(null, pageData), pageData)
  assert.equal(resolveCanonicalViewMetadata(undefined, pageData).locale, "page-locale")
})

test("error-only and null states remain coherent", () => {
  const errorData = fixture("error")
  assert.equal(resolveCanonicalViewData(errorData, undefined), errorData)
  assert.deepEqual(resolveCanonicalViewMetadata(null, undefined), {
    viewData: null,
    locale: undefined,
    licenseName: undefined,
    licenseUrl: undefined,
    licenseKind: undefined,
    licenseHtml: undefined,
    sourceSite: undefined
  })
})

test("an empty error payload remains authoritative instead of reviving retained shell data", () => {
  const pageData = fixture("page")
  const errorData = {}

  assert.equal(resolveCanonicalViewData(errorData, pageData), errorData)
  assert.deepEqual(resolveCanonicalViewMetadata(errorData, pageData), {
    viewData: errorData,
    locale: undefined,
    licenseName: undefined,
    licenseUrl: undefined,
    licenseKind: undefined,
    licenseHtml: undefined,
    sourceSite: undefined
  })
})
