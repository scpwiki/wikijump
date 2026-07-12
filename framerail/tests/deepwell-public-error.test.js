import assert from "node:assert/strict"
import test from "node:test"

import { stripPrivateDeepwellErrorData } from "../src/lib/server/deepwell/public-error.js"
import { publicErrorExtraMessage } from "../src/lib/popup/public-error-data.js"

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

test("batch sanitization handles every structured data shape without mutating input", () => {
  const responses = [
    {
      jsonrpc: "2.0",
      id: 1,
      error: { code: 1, message: "object", data: { call_trace: "secret" } }
    },
    { jsonrpc: "2.0", id: 2, error: { code: 2, message: "array", data: ["secret"] } },
    { jsonrpc: "2.0", id: 3, error: { code: 3, message: "number", data: 42 } },
    { jsonrpc: "2.0", id: 4, error: { code: 4, message: "boolean", data: false } },
    {
      jsonrpc: "2.0",
      id: 5,
      error: { code: 5, message: "string", data: "public detail" }
    },
    {
      jsonrpc: "2.0",
      id: 6,
      result: { data: { call_trace: "result data is not error data" } }
    }
  ]
  const before = structuredClone(responses)

  const sanitized = stripPrivateDeepwellErrorData(responses)

  assert.deepEqual(responses, before)
  for (const index of [0, 1, 2, 3]) assert.equal("data" in sanitized[index].error, false)
  assert.equal(sanitized[4], responses[4])
  assert.equal(sanitized[5], responses[5])
  assert.equal(sanitized[4].error.data, "public detail")
})

test("non-object responses and popup details expose only public strings", () => {
  for (const response of [null, "raw", 7, true]) {
    assert.equal(stripPrivateDeepwellErrorData(response), response)
  }

  assert.equal(publicErrorExtraMessage("public detail"), "public detail")
  assert.equal(publicErrorExtraMessage(""), null)
  for (const privateData of [null, { call_trace: "secret" }, ["secret"], 7, false]) {
    assert.equal(publicErrorExtraMessage(privateData), null)
  }
})
