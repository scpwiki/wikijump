import assert from "node:assert/strict"
import test from "node:test"

import { buildPublicPreloadData } from "../src/lib/server/load/preload-data.js"

const assertBrowserSerializationIsPublic = (preloadData) => {
  const serialized = JSON.stringify(preloadData)

  assert.equal(serialized.includes("session_token"), false)
  assert.equal(serialized.includes("secret-session"), false)
  assert.equal(serialized.includes("article_page_cache_key"), false)
  assert.equal(serialized.includes("public_content_cache_fence"), false)
  assert.equal(serialized.includes("anonymous_permission_cache_fence"), false)
}

test("public preload data is an allowlisted DTO", () => {
  const response = {
    site: { site_id: 1, name: "Example" },
    site_file_domain: "files.example",
    license_name: "CC BY-SA 3.0",
    license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
    license_kind: "standard",
    license_html: null,
    user_session: {
      session: { session_token: "secret-session" },
      user: { user_id: 7, email: "private@example.test" }
    },
    article_page_cache_key: "private-cache-key",
    public_content_cache_fence: "private-public-fence",
    anonymous_permission_cache_fence: "private-permission-fence",
    future_internal_field: "must not become public"
  }
  const publicUserSession = { user: { user_id: 7, name: "Public name" } }
  const locales = ["ja", "en"]

  const result = buildPublicPreloadData(response, publicUserSession, locales)

  assert.deepEqual(result, {
    site: response.site,
    site_file_domain: response.site_file_domain,
    license_name: response.license_name,
    license_url: response.license_url,
    license_kind: response.license_kind,
    license_html: response.license_html,
    user_session: publicUserSession,
    locales
  })
  assert.equal("session" in result.user_session, false)
  assert.equal("article_page_cache_key" in result, false)
  assert.equal("public_content_cache_fence" in result, false)
  assert.equal("anonymous_permission_cache_fence" in result, false)
  assert.equal("future_internal_field" in result, false)
  assertBrowserSerializationIsPublic(result)
})

test("anonymous browser serialization excludes session and cache internals", () => {
  const response = {
    site: { site_id: 1, name: "Example" },
    site_file_domain: "files.example",
    license_name: "CC BY-SA 3.0",
    license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
    license_kind: "standard",
    license_html: null,
    user_session: null,
    session_token: "secret-session",
    article_page_cache_key: "private-cache-key",
    public_content_cache_fence: "private-public-fence",
    anonymous_permission_cache_fence: "private-permission-fence"
  }

  const result = buildPublicPreloadData(response, null, ["en"])

  assert.equal(result.user_session, null)
  assertBrowserSerializationIsPublic(result)
})
