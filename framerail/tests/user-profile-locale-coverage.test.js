// @ts-nocheck
import { strict as assert } from "node:assert"
import { readdirSync, readFileSync, existsSync } from "node:fs"
import { fileURLToPath } from "node:url"
import test from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const fluentRoot = fileURLToPath(new URL("../../locales/fluent", import.meta.url))

/**
 * Collect the message selectors (`key` or `key.attribute`) defined by one
 * Fluent file. Attribute names cannot contain dots in Fluent, so the
 * parent message and attribute are joined by a single dot.
 */
const fluentSelectors = (source) => {
  const selectors = new Set()
  let message = null
  for (const line of source.split("\n")) {
    const messageMatch = /^([A-Za-z][A-Za-z0-9_-]*)\s*=/u.exec(line)
    if (messageMatch) {
      message = messageMatch[1]
      selectors.add(message)
      continue
    }
    const attributeMatch = /^\s+\.([A-Za-z][A-Za-z0-9_-]*)\s*=/u.exec(line)
    if (attributeMatch && message) {
      selectors.add(`${message}.${attributeMatch[1]}`)
    }
  }
  return selectors
}

/** Every selector a locale resolves, across all components' files for it. */
const localeSelectors = (locale) => {
  const selectors = new Set()
  for (const component of readdirSync(fluentRoot, { withFileTypes: true })) {
    if (!component.isDirectory()) continue
    const file = `${fluentRoot}/${component.name}/${locale}.ftl`
    if (!existsSync(file)) continue
    for (const selector of fluentSelectors(readFileSync(file, "utf8"))) {
      selectors.add(selector)
    }
  }
  return selectors
}

/**
 * Capture the translate keys the profile route requests from Deepwell, by
 * running the real load function with a stubbed Deepwell client.
 */
const profileRouteTranslateKeys = async (vite, client) => {
  const { loadUser } = await vite.ssrLoadModule("/src/lib/server/load/user.ts")
  const originalRequest = client.request
  let requestedKeys = null

  client.request = async (method, params) => {
    if (method === "user_view") {
      return {
        type: "user_found",
        data: {
          user: {
            user_id: 1,
            user_type: "regular",
            slug: "coverage-probe",
            name: "Coverage Probe",
            avatar_s3_hash: null
          }
        }
      }
    }
    if (method === "translate") {
      requestedKeys = Object.keys(params.messages)
      return {}
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  try {
    const request = new Request("https://wikijump.test/-/user", {
      headers: { "X-Wikijump-Site-Id": "1", "X-Wikijump-Site-Slug": "coverage" }
    })
    await loadUser(request, { get: () => undefined }, async () => ({
      locales: ["en"],
      site: { slug: "coverage", name: "Coverage", locale: "en" },
      site_file_domain: "coverage.wjfiles.test",
      license_name: "",
      license_url: "",
      license_kind: "",
      license_html: null,
      user_session: { user: { user_id: 1 } }
    }))
  } finally {
    client.request = originalRequest
  }
  assert.ok(requestedKeys, "profile route did not request translations")
  return requestedKeys
}

const AVATAR_REMOVAL_KEYS = [
  "user-profile-info.remove-avatar",
  "user-profile-info.remove-avatar-pending",
  "user-profile-info.keep-avatar"
]

test("avatar-removal labels requested by the profile route resolve in every UI locale", async () => {
  const previousWorkingDirectory = process.cwd()
  let vite
  try {
    process.chdir(root)
    vite = await createTestViteServer()
    const { client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts")
    const { USER_INTERFACE_LOCALES } = await vite.ssrLoadModule(
      "/src/lib/user-interface-locales.ts"
    )

    const requested = await profileRouteTranslateKeys(vite, client)
    const problems = []

    for (const key of AVATAR_REMOVAL_KEYS) {
      if (!requested.includes(key)) problems.push(`route does not request '${key}'`)
      // Fluent attribute names cannot contain dots, so such a key never resolves.
      if (key.split(".").length > 2) {
        problems.push(`'${key}' is not a valid Fluent selector`)
      }
    }

    for (const { value } of USER_INTERFACE_LOCALES) {
      const stem = value.replace("-", "_")
      const selectors = localeSelectors(stem)
      for (const key of AVATAR_REMOVAL_KEYS) {
        if (!selectors.has(key)) problems.push(`${stem}: missing '${key}'`)
      }
    }
    assert.deepEqual(problems, [])
  } finally {
    if (vite) await vite.close()
    process.chdir(previousWorkingDirectory)
  }
})
