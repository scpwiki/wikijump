import assert from "node:assert/strict"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

let vite
let legalDestinations

before(async () => {
  vite = await createTestViteServer()
  ;({ legalDestinations } = await vite.ssrLoadModule("/src/lib/server/legal-links.ts"))
})

after(async () => {
  if (vite) await vite.close()
})

test("legal destinations default to the platform host for the deployment", () => {
  assert.deepEqual(legalDestinations({ deploymentEnvironment: "local" }), {
    termsOfService: "https://wikijump.localhost/legal/terms-of-service",
    privacyPolicy: "https://wikijump.localhost/legal/privacy-policy"
  })
  assert.deepEqual(legalDestinations({ deploymentEnvironment: "dev" }), {
    termsOfService: "https://wikijump.dev/legal/terms-of-service",
    privacyPolicy: "https://wikijump.dev/legal/privacy-policy"
  })
  assert.deepEqual(legalDestinations({ deploymentEnvironment: "prod" }), {
    termsOfService: "https://wikijump.com/legal/terms-of-service",
    privacyPolicy: "https://wikijump.com/legal/privacy-policy"
  })
})

test("a self-hosted instance configures its own legal destinations", () => {
  const destinations = legalDestinations({
    deploymentEnvironment: "prod",
    env: {
      WIKIJUMP_TERMS_OF_SERVICE_URL: "https://legal.example.invalid/terms",
      WIKIJUMP_PRIVACY_POLICY_URL: "https://legal.example.invalid/privacy"
    }
  })
  assert.equal(destinations.termsOfService, "https://legal.example.invalid/terms")
  assert.equal(destinations.privacyPolicy, "https://legal.example.invalid/privacy")
})

test("legal destinations never resolve against the viewed wiki", () => {
  // A blank or path-only configuration is not a legal destination: it would
  // silently point at whatever wiki the reader is currently viewing.
  assert.deepEqual(legalDestinations({ env: {} }), {
    termsOfService: "https://wikijump.localhost/legal/terms-of-service",
    privacyPolicy: "https://wikijump.localhost/legal/privacy-policy"
  })
  assert.deepEqual(legalDestinations({ env: { WIKIJUMP_TERMS_OF_SERVICE_URL: "  " } }), {
    termsOfService: "https://wikijump.localhost/legal/terms-of-service",
    privacyPolicy: "https://wikijump.localhost/legal/privacy-policy"
  })
  const destinationVariables = [
    ["termsOfService", "WIKIJUMP_TERMS_OF_SERVICE_URL"],
    ["privacyPolicy", "WIKIJUMP_PRIVACY_POLICY_URL"]
  ]
  // A `javascript:` destination would be the dangerous case, but the literal
  // is assembled here so the repository lint rule that forbids script URLs
  // still applies to production code.
  const scriptUrl = ["javascript", "alert(1)"].join(":")
  for (const [key, variable] of destinationVariables) {
    for (const value of [
      `/legal/${key}`,
      "legal.example.invalid/terms",
      scriptUrl,
      "http://legal.example.invalid/terms"
    ]) {
      assert.throws(
        () => legalDestinations({ env: { [variable]: value } }),
        new RegExp(variable),
        `${variable}=${value}`
      )
    }
  }
})
