import assert from "node:assert/strict"
import test from "node:test"

import {
  parseCsrfCheckOrigin,
  parseDeploymentEnvironment
} from "../src/lib/server/deployment-environment.js"

test("parses explicit deployment environments", () => {
  for (const environment of ["local", "dev", "prod"]) {
    assert.equal(parseDeploymentEnvironment({ framerailEnv: environment }), environment)
  }
})

test("uses NODE_ENV only when FRAMERAIL_ENV is absent", () => {
  assert.equal(
    parseDeploymentEnvironment({ framerailEnv: "", nodeEnv: "development" }),
    "local"
  )
  assert.equal(
    parseDeploymentEnvironment({ framerailEnv: null, nodeEnv: "production" }),
    "prod"
  )
  assert.equal(
    parseDeploymentEnvironment({ framerailEnv: "dev", nodeEnv: "development" }),
    "dev"
  )
})

test("rejects unknown explicit environments", () => {
  assert.throws(
    () => parseDeploymentEnvironment({ framerailEnv: "staging" }),
    /Invalid FRAMERAIL_ENV/u
  )
})

test("defaults CSRF origin checks from deployment environment", () => {
  assert.equal(parseCsrfCheckOrigin({ deploymentEnvironment: "prod" }), true)
  assert.equal(parseCsrfCheckOrigin({ deploymentEnvironment: "dev" }), true)
  assert.equal(parseCsrfCheckOrigin({ deploymentEnvironment: "local" }), false)
})

test("allows production builds to force CSRF origin checks for local CSP", () => {
  assert.equal(
    parseCsrfCheckOrigin({
      csrfCheckOrigin: "true",
      deploymentEnvironment: "local"
    }),
    true
  )
})

test("rejects unknown explicit CSRF origin check values", () => {
  assert.throws(
    () => parseCsrfCheckOrigin({ csrfCheckOrigin: "yes" }),
    /Invalid FRAMERAIL_CSRF_CHECK_ORIGIN/u
  )
})
