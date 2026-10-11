// @ts-nocheck
import assert from "node:assert/strict"
import { after, before, test } from "node:test"

import { pageActionEvent, startPageActionHarness } from "./page-action-test-harness.js"

const SITE_ID = 17
const ROLLBACK = {
  siteId: SITE_ID,
  pageId: 23,
  fileId: 5,
  revisionNumber: 6,
  lastRevisionId: 8,
  comments: "issue 2195 test"
}

let harness
let originalClientRequest

before(async () => {
  harness = await startPageActionHarness()
  originalClientRequest = harness.client.request
})

after(async () => {
  if (harness) {
    harness.client.request = originalClientRequest
    await harness.close()
  }
})

test("forged anonymous file rollback is rejected before the mutation RPC", async () => {
  const calls = []
  harness.client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const result = await harness.actions.fileRollback(
    pageActionEvent({
      action: "fileRollback",
      body: ROLLBACK,
      siteId: SITE_ID,
      requestContext: { siteId: SITE_ID, page: "main" }
    })
  )

  assert.equal(result.status, 403)
  assert.equal(result.data.message, "Permission denied.")
  assert.deepEqual(calls, [])
})

test("an authenticated file rollback retains its existing scoped Deepwell request", async () => {
  const calls = []
  harness.client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    if (method === "session_get") return { user_id: 91 }
    if (method === "file_rollback") return { file_id: 5, revision_id: 9 }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const result = await harness.actions.fileRollback(
    pageActionEvent({
      action: "fileRollback",
      body: ROLLBACK,
      siteId: SITE_ID,
      sessionToken: "rollback-session",
      requestContext: {
        siteId: SITE_ID,
        page: "main",
        sessionToken: "rollback-session"
      },
      clientAddress: "192.0.2.14"
    })
  )

  assert.deepEqual(result.res, { file_id: 5, revision_id: 9 })
  assert.deepEqual(calls, [
    {
      method: "session_get",
      params: ["rollback-session"],
      context: undefined
    },
    {
      method: "file_rollback",
      params: {
        site_id: SITE_ID,
        page_id: 23,
        user_id: 91,
        file: 5,
        last_revision_id: 8,
        revision_number: 6,
        revision_comments: "issue 2195 test",
        ip_address: "192.0.2.14",
        bypass_filter: false
      },
      context: {
        siteId: SITE_ID,
        page: "main",
        sessionToken: "rollback-session"
      }
    }
  ])
})
