// @ts-nocheck
import { strict as assert } from "node:assert"
import { readdirSync, readFileSync, existsSync } from "node:fs"
import { fileURLToPath } from "node:url"
import test from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const fluentRoot = fileURLToPath(new URL("../../locales/fluent", import.meta.url))

/**
 * Pre-existing user-profile translation gaps, recorded so this check can
 * land without silently widening scope. Each entry is a key the profile
 * route requests that some UI locale does not yet define. The test fails
 * if a key listed here becomes translated (so the list must shrink) and
 * fails for any other gap. Remove entries as the catalogs are filled in.
 */
const KNOWN_PRE_EXISTING_GAPS = {
  ja: [
    "avatar",
    "cancel",
    "close",
    "docs",
    "edit",
    "error",
    "field-required",
    "footer-license-unless",
    "footer-powered-by",
    "hold-to-show-password",
    "message-loading",
    "privacy",
    "save",
    "security",
    "spinny-label.active",
    "spinny-label.error",
    "spinny-label.success",
    "spinny-label.warning",
    "terms-conditions",
    "user-not-exist",
    "user-not-logged-in",
    "user-profile-info.avatar",
    "user-profile-info.biography",
    "user-profile-info.birthday",
    "user-profile-info.email",
    "user-profile-info.gender",
    "user-profile-info.locales",
    "user-profile-info.location",
    "user-profile-info.name",
    "user-profile-info.real-name",
    "user-profile-info.user-page",
    "user-profile-info.website"
  ],
  ko: [
    "cancel",
    "error",
    "user-not-exist",
    "user-not-logged-in",
    "user-profile-info.avatar",
    "user-profile-info.biography",
    "user-profile-info.email",
    "user-profile-info.gender",
    "user-profile-info.locales",
    "user-profile-info.real-name",
    "user-profile-info.user-page",
    "user-profile-info.website"
  ],
  pl: [
    "cancel",
    "error",
    "footer-license-unless",
    "footer-powered-by",
    "spinny-label.active",
    "spinny-label.error",
    "spinny-label.success",
    "spinny-label.warning",
    "user-not-exist",
    "user-not-logged-in",
    "user-profile-info.avatar",
    "user-profile-info.biography",
    "user-profile-info.birthday",
    "user-profile-info.email",
    "user-profile-info.gender",
    "user-profile-info.locales",
    "user-profile-info.location",
    "user-profile-info.name",
    "user-profile-info.real-name",
    "user-profile-info.user-page",
    "user-profile-info.website"
  ],
  vi: [
    "cancel",
    "error",
    "footer-license-unless",
    "footer-powered-by",
    "spinny-label.active",
    "spinny-label.error",
    "spinny-label.success",
    "spinny-label.warning",
    "user-not-exist",
    "user-not-logged-in",
    "user-profile-info.avatar",
    "user-profile-info.biography",
    "user-profile-info.birthday",
    "user-profile-info.email",
    "user-profile-info.gender",
    "user-profile-info.locales",
    "user-profile-info.location",
    "user-profile-info.name",
    "user-profile-info.real-name",
    "user-profile-info.user-page",
    "user-profile-info.website"
  ],
  zh_Hans: ["user-profile-info.website"]
}

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

test("every key the user profile route requests resolves in every UI locale", async () => {
  const previousWorkingDirectory = process.cwd()
  let vite
  try {
    process.chdir(root)
    vite = await createTestViteServer()
    const { client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts")
    const { USER_INTERFACE_LOCALES } = await vite.ssrLoadModule(
      "/src/lib/user-interface-locales.ts"
    )

    const keys = await profileRouteTranslateKeys(vite, client)
    const problems = []

    // The avatar-removal labels are required on this route.
    for (const required of [
      "user-profile-info.remove-avatar",
      "user-profile-info.remove-avatar-pending",
      "user-profile-info.keep-avatar"
    ]) {
      if (!keys.includes(required)) problems.push(`route does not request '${required}'`)
    }

    // Fluent attribute names cannot contain dots, so a key with two dots can
    // never resolve, whatever the catalog says.
    for (const key of keys) {
      if (key.split(".").length > 2) {
        problems.push(`translate key '${key}' is not a valid Fluent selector`)
      }
    }

    for (const { value } of USER_INTERFACE_LOCALES) {
      const stem = value.replace("-", "_")
      const selectors = localeSelectors(stem)
      const allowed = new Set(KNOWN_PRE_EXISTING_GAPS[stem] ?? [])
      for (const key of keys) {
        const present = selectors.has(key)
        if (!present && !allowed.has(key)) {
          problems.push(`${stem}: missing '${key}'`)
        }
        if (present && allowed.has(key)) {
          problems.push(
            `${stem}: '${key}' is now translated; remove it from the gap list`
          )
        }
      }
    }
    assert.deepEqual(problems, [])
  } finally {
    if (vite) await vite.close()
    process.chdir(previousWorkingDirectory)
  }
})
