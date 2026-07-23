import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildPageFileCreatePayload,
  buildPageFileEditPayload,
  buildPageFileRestorePayload,
  buildPageFileRollbackPayload,
  withPageFileClientAddress
} from "../src/lib/server/deepwell/page-file-mutation-payloads.ts"

const CLIENT_IP = "192.0.2.14"

test("file mutation actions forward getClientAddress through the Deepwell transport", async () => {
  const cases = [
    [
      "file_create",
      buildPageFileCreatePayload,
      {
        siteId: 1,
        pageId: 2,
        userId: 3,
        name: "example.txt",
        pendingBlobId: "pending-create",
        revisionComments: "create",
        bypassFilter: false
      }
    ],
    [
      "file_edit",
      buildPageFileEditPayload,
      {
        siteId: 1,
        pageId: 2,
        userId: 3,
        fileId: 4,
        lastRevisionId: 5,
        name: undefined,
        pendingBlobId: "pending-edit",
        revisionComments: "edit",
        bypassFilter: false
      }
    ],
    [
      "file_restore",
      buildPageFileRestorePayload,
      {
        siteId: 1,
        pageId: 2,
        userId: 3,
        fileId: 4,
        newPage: undefined,
        newName: undefined,
        revisionComments: "restore",
        bypassFilter: false
      }
    ],
    [
      "file_rollback",
      buildPageFileRollbackPayload,
      {
        siteId: 1,
        pageId: 2,
        userId: 3,
        fileId: 4,
        lastRevisionId: 5,
        revisionNumber: 6,
        revisionComments: "rollback",
        bypassFilter: false
      }
    ]
  ]
  const requestCalls = []
  const fakeDeepwellRequest = async (method, params) => {
    requestCalls.push({ method, params })
  }

  for (const [method, buildPayload, input] of cases) {
    const actionInput = withPageFileClientAddress(() => CLIENT_IP, input)
    await fakeDeepwellRequest(method, buildPayload(actionInput))
  }

  assert.deepEqual(
    requestCalls.map(({ method, params }) => [method, params.ip_address]),
    [
      ["file_create", CLIENT_IP],
      ["file_edit", CLIENT_IP],
      ["file_restore", CLIENT_IP],
      ["file_rollback", CLIENT_IP]
    ]
  )
})
