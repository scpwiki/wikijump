import assert from "node:assert/strict"
import test from "node:test"

import {
  WIKIDOT_LOGIN_DIALOG_LABELS,
  interpretLoginActionResult,
  isPlainPrimaryActivation
} from "../src/lib/wikidot/wikidot-login-dialog.js"

test("successful native login action closes the in-place surface as success", () => {
  assert.deepEqual(
    interpretLoginActionResult({
      type: "success",
      data: { needsMfa: false, isLoggedIn: true }
    }),
    { kind: "success" }
  )
})

test("MFA-required action never exposes the pending session token to the surface", () => {
  const outcome = interpretLoginActionResult({
    type: "success",
    data: { needsMfa: true, isLoggedIn: false, session_token: "pending-token" }
  })

  assert.deepEqual(outcome, { kind: "mfa" })
  assert.equal(JSON.stringify(outcome).includes("pending-token"), false)
})

test("failed native login action keeps the server message only when it is a string", () => {
  assert.deepEqual(
    interpretLoginActionResult({ type: "failure", data: { message: "Bad credentials" } }),
    { kind: "failure", message: "Bad credentials" }
  )
  assert.deepEqual(
    interpretLoginActionResult({ type: "failure", data: { message: { nested: true } } }),
    { kind: "failure", message: null }
  )
})

test("unexpected action results surface as a generic error", () => {
  assert.deepEqual(interpretLoginActionResult({ type: "error" }), { kind: "error" })
  assert.deepEqual(interpretLoginActionResult({ type: "redirect" }), { kind: "error" })
  assert.deepEqual(interpretLoginActionResult(null), { kind: "error" })
})

test("a success payload without isLoggedIn is not treated as signed in", () => {
  assert.deepEqual(interpretLoginActionResult({ type: "success", data: {} }), {
    kind: "failure",
    message: null
  })
})

test("only plain primary activations are intercepted in place", () => {
  const plain = {
    button: 0,
    metaKey: false,
    ctrlKey: false,
    shiftKey: false,
    altKey: false
  }
  assert.equal(isPlainPrimaryActivation(plain), true)
  assert.equal(isPlainPrimaryActivation({ ...plain, button: 1 }), false)
  assert.equal(isPlainPrimaryActivation({ ...plain, metaKey: true }), false)
  assert.equal(isPlainPrimaryActivation({ ...plain, ctrlKey: true }), false)
  assert.equal(isPlainPrimaryActivation({ ...plain, shiftKey: true }), false)
  assert.equal(isPlainPrimaryActivation({ ...plain, altKey: true }), false)
})

test("in-place sign-in labels are defined for the surface", () => {
  for (const [key, value] of Object.entries(WIKIDOT_LOGIN_DIALOG_LABELS)) {
    assert.equal(typeof value, "string", key)
    assert.notEqual(value.trim(), "", key)
  }
})
