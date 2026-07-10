import assert from "node:assert/strict"
import test from "node:test"

import { stripPrivateDeepwellErrorData } from "../src/lib/server/deepwell/public-error.js"

test("structured Deepwell diagnostics are removed without mutating the response", () => {
  const response = {
    jsonrpc: "2.0",
    id: 1,
    error: {
      code: 2101,
      message: "Password is empty",
      data: {
        call_trace: "private service trace",
        code_trace: [1004, 2101],
        extra: [{ submitted_password: "secret" }]
      }
    }
  }

  const sanitized = stripPrivateDeepwellErrorData(response)

  assert.deepEqual(sanitized, {
    jsonrpc: "2.0",
    id: 1,
    error: { code: 2101, message: "Password is empty" }
  })
  assert.notEqual(sanitized, response)
  assert.equal(response.error.data.call_trace, "private service trace")
})

test("plain string error data and successful responses are preserved", () => {
  const publicError = {
    jsonrpc: "2.0",
    id: 2,
    error: { code: -32602, message: "Invalid params", data: "public detail" }
  }
  const success = { jsonrpc: "2.0", id: 3, result: { ok: true } }

  assert.equal(stripPrivateDeepwellErrorData(publicError), publicError)
  assert.equal(stripPrivateDeepwellErrorData(success), success)
})

test("batched responses sanitize only structured error data", () => {
  const responses = [
    { jsonrpc: "2.0", id: 1, error: { code: 1, message: "one", data: null } },
    { jsonrpc: "2.0", id: 2, error: { code: 2, message: "two", data: "safe" } },
    { jsonrpc: "2.0", id: 3, result: "ok" }
  ]

  assert.deepEqual(stripPrivateDeepwellErrorData(responses), [
    { jsonrpc: "2.0", id: 1, error: { code: 1, message: "one" } },
    responses[1],
    responses[2]
  ])
})
