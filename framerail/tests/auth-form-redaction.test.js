import { strict as assert } from "node:assert"
import test from "node:test"

import {
  clearLoginPassword,
  clearRegisterPasswords,
  redactAuthActionPayload
} from "../src/lib/server/load/auth-form-redaction.js"

test("login action forms do not serialize the submitted password", () => {
  const form = {
    valid: false,
    data: { nameOrEmail: "alice@example.com", password: "submitted" },
    errors: { password: ["Invalid password"] }
  }

  assert.equal(clearLoginPassword(form), form)
  assert.equal(form.data.password, "")
  assert.equal(form.data.nameOrEmail, "alice@example.com")
  assert.doesNotMatch(JSON.stringify(form), /submitted/)
})

test("auth action payload serialization removes nested credentials on every path", () => {
  const password = "correct horse battery staple"
  const confirmPassword = "confirm secret"
  const mfaSessionToken = "mfa-session-token-must-remain"
  const paths = {
    invalid: {
      form: { data: { password, confirmPassword }, errors: { password: [password] } }
    },
    backend: {
      form: { data: { password } },
      message: `backend echoed ${password}`,
      data: { nested: [{ submitted: password }] }
    },
    mfa: {
      form: { data: { password } },
      session_token: mfaSessionToken,
      needsMfa: true
    },
    success: {
      form: { data: { password, confirmPassword } },
      res: { audit: { accidentalEcho: confirmPassword } },
      isRegistered: true
    }
  }

  for (const [path, payload] of Object.entries(paths)) {
    const redacted = redactAuthActionPayload(payload, [password, confirmPassword])
    const serialized = JSON.stringify(redacted)
    assert.doesNotMatch(serialized, new RegExp(password), path)
    assert.doesNotMatch(serialized, new RegExp(confirmPassword), path)
  }

  assert.equal(paths.mfa.session_token, mfaSessionToken)
  assert.match(JSON.stringify(paths.mfa), new RegExp(mfaSessionToken))
})

test("registration action forms clear both password fields", () => {
  const form = {
    valid: false,
    data: {
      username: "alice",
      password: "submitted",
      confirmPassword: "submitted"
    }
  }

  assert.equal(clearRegisterPasswords(form), form)
  assert.equal(form.data.password, "")
  assert.equal(form.data.confirmPassword, "")
  assert.equal(form.data.username, "alice")
  assert.doesNotMatch(JSON.stringify(form), /submitted/)
})
