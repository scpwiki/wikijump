import assert from "node:assert/strict"
import test from "node:test"

import { handleAjaxModuleConnectorRequest } from "../src/lib/server/ajax-module-connector.js"

const request = (form, options = {}) =>
  new Request("http://scp-wiki.local/ajax-module-connector.php", {
    method: options.method ?? "POST",
    headers: {
      "content-type": "application/x-www-form-urlencoded",
      ...(options.headers ?? {})
    },
    body: options.method === "GET" ? undefined : new URLSearchParams(form)
  })

test("dispatches ListPages forms and returns the Wikidot JSON envelope", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "list/ListPagesModule",
      module_body: '[[div class="page"]]%%fullname%%[[/div]]',
      wikidot_token7: "client-token",
      category: "_default",
      name: "scp-173",
      perPage: "250",
      separate: "no",
      wrapper: "no"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        received = input
        return { body: '<div class="page">scp-173</div>' }
      }
    }
  )

  assert.equal(response.status, 200)
  assert.deepEqual(await response.json(), {
    status: "ok",
    body: '<div class="page">scp-173</div>'
  })
  assert.deepEqual(received, {
    siteId: 6000006,
    moduleBody: '[[div class="page"]]%%fullname%%[[/div]]',
    parameters: {
      category: "_default",
      name: "scp-173",
      perPage: "250",
      separate: "no",
      wrapper: "no"
    }
  })
  assert.equal(response.headers.get("cache-control"), "no-store")
})

test("fails closed for unsupported modules and duplicate fields", async () => {
  const unsupported = await handleAjaxModuleConnectorRequest(
    request({ moduleName: "forum/ForumStartModule", module_body: "" }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )
  assert.deepEqual(await unsupported.json(), {
    status: "not_ok",
    message: "Unsupported AJAX module: forum/ForumStartModule"
  })

  const duplicate = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=list%2FListPagesModule&moduleName=list%2FListPagesModule&module_body=x"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )
  assert.equal(duplicate.status, 400)
  assert.equal((await duplicate.json()).status, "not_ok")
})

test("converts Deepwell failures to a stable Wikidot error envelope", async () => {
  const originalConsoleError = console.error
  console.error = () => {}
  try {
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "list/ListPagesModule",
        module_body: "%%fullname%%",
        name: "="
      }),
      {
        siteId: 6000006,
        renderListPages: async () => {
          throw new Error("current-page selectors are unsupported")
        }
      }
    )
    assert.deepEqual(await response.json(), {
      status: "not_ok",
      message: "Unable to render ListPages module"
    })
  } finally {
    console.error = originalConsoleError
  }
})
