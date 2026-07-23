import { strict as assert } from "node:assert"
import test from "node:test"

import { normalizeActionError } from "../src/lib/server/load/action-error.ts"

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
