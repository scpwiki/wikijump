import assert from "node:assert/strict"
import test from "node:test"

import { isPageErrorData } from "../src/lib/page-error-data.ts"

test("generic SvelteKit HTTP errors do not enter wiki-page form rendering", () => {
  assert.equal(isPageErrorData({ status: 404, message: "Not Found" }), false)
  assert.equal(isPageErrorData({ status: 403, message: "Forbidden" }), false)
  assert.equal(isPageErrorData({ status: 400, message: "Bad Request" }), false)
})

test("page-specific errors require validated site, options, and form payloads", () => {
  const pageError = {
    view: "missing",
    forms: {
      pageEditForm: {
        data: {},
        errors: {},
        constraints: {},
        valid: true,
        posted: false
      },
      pageRestoreForm: {
        data: {},
        errors: {},
        constraints: {},
        valid: true,
        posted: false
      }
    },
    site: { site_id: 6000005, default_page: "main" },
    options: {
      edit: false,
      no_redirect: false,
      no_render: false,
      debug: false,
      renderer: false,
      comments: false,
      history: false,
      data: ""
    },
    compiled_body_html: "<p>missing page</p>",
    page_templates: []
  }

  assert.equal(isPageErrorData(pageError), true)
  assert.equal(isPageErrorData({ ...pageError, forms: { pageEditForm: {} } }), false)
  assert.equal(isPageErrorData({ ...pageError, site: null }), false)
  assert.equal(isPageErrorData({ ...pageError, compiled_body_html: null }), false)
})

test("restore availability is an optional server-computed boolean (Refs #2090)", () => {
  const base = {
    view: "missing",
    forms: {
      pageEditForm: { data: {}, errors: {}, constraints: {}, valid: true, posted: false },
      pageRestoreForm: {
        data: {},
        errors: {},
        constraints: {},
        valid: true,
        posted: false
      }
    },
    site: { site_id: 6000005, default_page: "main" },
    options: {
      edit: false,
      no_redirect: false,
      no_render: false,
      debug: false,
      renderer: false,
      comments: false,
      history: false,
      data: ""
    },
    compiled_body_html: "<p>missing page</p>"
  }

  // Absent (for example anonymous or never-existing) must validate: the
  // missing-page shell renders without a Restore control.
  assert.equal(isPageErrorData(base), true)
  assert.equal(isPageErrorData({ ...base, restore_available: true }), true)
  assert.equal(isPageErrorData({ ...base, restore_available: false }), true)
  assert.equal(isPageErrorData({ ...base, restore_available: "yes" }), false)
  assert.equal(isPageErrorData({ ...base, restore_available: 1 }), false)
  assert.equal(isPageErrorData({ ...base, restore_available: null }), false)
})
