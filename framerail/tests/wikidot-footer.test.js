import { strict as assert } from "node:assert"
import test from "node:test"

import {
  WIKIDOT_FOOTER_LINKS,
  WIKIDOT_POWERED_BY,
  buildWikidotLicenseHtml,
  formatWikidotLicenseName,
  isImportedWikidotView
} from "../src/lib/wikidot-footer.js"

test("uses Wikidot footer labels in source order", () => {
  assert.deepEqual(
    WIKIDOT_FOOTER_LINKS.map((link) => link.label),
    ["Help", "Terms of Service", "Privacy", "Report a bug", "Flag as objectionable"]
  )
  assert.equal(WIKIDOT_POWERED_BY, "Powered by Wikidot.com")
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

test("selects Wikidot footer branding only for imported Wikidot views", () => {
  assert.equal(isImportedWikidotView(null), false)
  assert.equal(isImportedWikidotView({ site: { from_wikidot: false } }), false)
  assert.equal(isImportedWikidotView({ site: { from_wikidot: true } }), true)
  assert.equal(isImportedWikidotView({ page: { from_wikidot: true } }), true)
  assert.equal(isImportedWikidotView({ page_revision: { from_wikidot: true } }), true)
})
