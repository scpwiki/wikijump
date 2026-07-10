import { strict as assert } from "node:assert"
import test from "node:test"

import {
  clearLoginPassword,
  clearRegisterPasswords
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
