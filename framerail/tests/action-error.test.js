import { strict as assert } from "node:assert"
import test from "node:test"

import {
  failForActionError,
  failForMissingSession,
  normalizeActionError,
  readActionJson
} from "../src/lib/server/load/action-error.ts"

test("action errors preserve validated public Deepwell details", () => {
  assert.deepEqual(
    normalizeActionError({
      message: "Permission denied",
      code: 3106,
      data: { resource: "page", allowed: false }
    }),
    {
      message: "Permission denied",
      code: 3106,
      data: { resource: "page", allowed: false }
    }
  )
})

test("action errors safely normalize arbitrary thrown values", () => {
  assert.deepEqual(normalizeActionError("network failed"), {
    message: "network failed"
  })
  assert.deepEqual(normalizeActionError({ code: "not-numeric", data: undefined }), {
    message: "An unexpected server error occurred."
  })
})

test("action failures classify permission errors and honor fallback statuses", () => {
  const denied = failForActionError({ message: "Permission denied", code: 3106 })
  assert.equal(denied.status, 403)
  assert.deepEqual(denied.data, {
    message: "Permission denied",
    code: 3106
  })

  const unavailable = failForActionError(new Error("Backend unavailable"), {}, 502)
  assert.equal(unavailable.status, 502)
  assert.deepEqual(unavailable.data, {
    message: "Backend unavailable"
  })
})

test("missing sessions remain an explicit authentication failure", () => {
  const failure = failForMissingSession({ form: "preserved" })
  assert.equal(failure.status, 401)
  assert.deepEqual(failure.data, {
    form: "preserved",
    message: "Authentication required."
  })
})

test("malformed action JSON is a client error", async () => {
  const request = new Request("https://example.test", {
    method: "POST",
    body: "{"
  })

  await assert.rejects(readActionJson(request), (error) => {
    const failure = failForActionError(error)
    assert.equal(failure.status, 400)
    assert.deepEqual(failure.data, {
      message: "Invalid JSON request body."
    })
    return true
  })
})
