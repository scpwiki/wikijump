import assert from "node:assert/strict"
import { describe, it } from "node:test"

import {
  customLicenseSourceForEdit,
  licenseFormValues,
  licenseOptionsFor,
  licenseUpdateValue,
  WIKIDOT_LICENSE_OPTIONS,
  WIKIDOT_STANDARD_LICENSE_OPTIONS
} from "../src/lib/admin/admin-license.js"

describe("Wikidot category license settings", () => {
  it("exposes the thirteen directly typed Wikidot license choices", () => {
    assert.equal(WIKIDOT_STANDARD_LICENSE_OPTIONS.length, 13)
    assert.deepEqual(
      WIKIDOT_STANDARD_LICENSE_OPTIONS.map(({ value }) => value),
      [
        "cc-by-sa-3.0",
        "cc-by-3.0",
        "cc-by-nd-3.0",
        "cc-by-nc-3.0",
        "cc-by-nc-sa-3.0",
        "cc-by-nc-nd-3.0",
        "cc-by-sa-2.5",
        "cc-by-2.5",
        "cc-by-nd-2.5",
        "cc-by-nc-2.5",
        "cc-by-nc-sa-2.5",
        "cc-by-nc-nd-2.5",
        "gnu-fdl-1.2"
      ]
    )
  })

  it("exposes all fifteen live Wikidot choices in source order", () => {
    assert.equal(WIKIDOT_LICENSE_OPTIONS.length, 15)
    assert.deepEqual(
      WIKIDOT_LICENSE_OPTIONS.slice(-2).map(({ value }) => value),
      ["other", "copyright"]
    )
  })

  it("shows the site license while a category inherits", () => {
    assert.deepEqual(
      licenseFormValues(
        { category_id: 12, slug: "article", license: null },
        "cc-by-sa-3.0"
      ),
      { categoryId: 12, inherit: true, license: "cc-by-sa-3.0", licenseOther: "" }
    )
  })

  it("treats _default as the explicit inheritance baseline", () => {
    assert.deepEqual(
      licenseFormValues(
        { category_id: 1, slug: "_default", license: null },
        "cc-by-sa-3.0"
      ),
      { categoryId: 1, inherit: false, license: "cc-by-sa-3.0", licenseOther: "" }
    )
  })

  it("shows the _default category override rather than the legacy site fallback", () => {
    assert.deepEqual(
      licenseFormValues({ category_id: 12, slug: "article", license: null }, "cc-by-3.0"),
      { categoryId: 12, inherit: true, license: "cc-by-3.0", licenseOther: "" }
    )
  })

  it("carries an inherited custom description into the editable form", () => {
    assert.deepEqual(
      licenseFormValues(
        { category_id: 12, slug: "article", license: null, license_other: null },
        "other",
        "Inherited terms"
      ),
      { categoryId: 12, inherit: true, license: "other", licenseOther: "Inherited terms" }
    )
  })

  it("decodes sanitized custom HTML before editing without collapsing literal entities", () => {
    assert.equal(
      customLicenseSourceForEdit(
        'Terms &amp; conditions &gt; defaults <a href="/?a=1&amp;b=2">Details</a> &amp;gt;'
      ),
      'Terms & conditions > defaults <a href="/?a=1&b=2">Details</a> &gt;'
    )
  })

  it("returns explicit and inherited custom licenses as stable sanitizer input", () => {
    const stored = "Terms &amp; conditions &gt; defaults"
    const source = "Terms & conditions > defaults"
    const explicit = licenseFormValues(
      { category_id: 12, slug: "article", license: "other", license_other: stored },
      "cc-by-sa-3.0"
    )
    const inherited = licenseFormValues(
      { category_id: 12, slug: "article", license: null, license_other: null },
      "other",
      stored
    )

    assert.equal(explicit.licenseOther, source)
    assert.equal(inherited.licenseOther, source)
    assert.deepEqual(licenseUpdateValue(explicit), {
      license: "other",
      licenseOther: source
    })
    assert.deepEqual(licenseUpdateValue({ ...inherited, inherit: false }), {
      license: "other",
      licenseOther: source
    })
  })

  it("retains a valid migrated license that Wikidot does not offer for new selection", () => {
    const options = licenseOptionsFor("cc-by-sa-4.0")
    assert.equal(options.at(-1)?.value, "cc-by-sa-4.0")
    assert.equal(licenseOptionsFor("cc-by-sa-3.0"), WIKIDOT_LICENSE_OPTIONS)
  })

  it("maps inheritance to a nullable category override", () => {
    assert.deepEqual(licenseUpdateValue({ inherit: true, license: "cc-by-3.0" }), {
      license: null,
      licenseOther: null
    })
    assert.deepEqual(licenseUpdateValue({ inherit: false, license: "cc-by-3.0" }), {
      license: "cc-by-3.0",
      licenseOther: null
    })
    assert.deepEqual(
      licenseUpdateValue({ inherit: false, license: "other", licenseOther: "Terms" }),
      { license: "other", licenseOther: "Terms" }
    )
  })
})
