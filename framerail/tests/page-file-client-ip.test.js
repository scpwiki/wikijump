import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildPageFileCreatePayload,
  buildPageFileEditPayload,
  buildPageFileRestorePayload,
  buildPageFileRollbackPayload
} from "../src/lib/server/deepwell/page-file-mutation-payloads.ts"

const CLIENT_IP = "192.0.2.14"

test("file mutation payloads serialize the server supplied client IP", () => {
  const createPayload = buildPageFileCreatePayload({
    siteId: 1,
    pageId: 2,
    userId: 3,
    name: "example.txt",
    pendingBlobId: "pending-create",
    revisionComments: "create",
    ipAddress: CLIENT_IP,
    bypassFilter: false
  })
  const editPayload = buildPageFileEditPayload({
    siteId: 1,
    pageId: 2,
    userId: 3,
    fileId: 4,
    lastRevisionId: 5,
    name: undefined,
    pendingBlobId: "pending-edit",
    revisionComments: "edit",
    ipAddress: CLIENT_IP,
    bypassFilter: false
  })
  const restorePayload = buildPageFileRestorePayload({
    siteId: 1,
    pageId: 2,
    userId: 3,
    fileId: 4,
    newPage: undefined,
    newName: undefined,
    revisionComments: "restore",
    ipAddress: CLIENT_IP,
    bypassFilter: false
  })
  const rollbackPayload = buildPageFileRollbackPayload({
    siteId: 1,
    pageId: 2,
    userId: 3,
    fileId: 4,
    lastRevisionId: 5,
    revisionNumber: 6,
    revisionComments: "rollback",
    ipAddress: CLIENT_IP,
    bypassFilter: false
  })

  assert.equal(createPayload.ip_address, CLIENT_IP)
  assert.equal(editPayload.ip_address, CLIENT_IP)
  assert.equal(restorePayload.ip_address, CLIENT_IP)
  assert.equal(rollbackPayload.ip_address, CLIENT_IP)
})
