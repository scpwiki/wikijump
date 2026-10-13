// @ts-nocheck
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

// Refs #2059. The native account/profile/error routes must never expose
// literal `UNTRANSLATED:` developer markers: headings, form labels, inline
// validation, and error titles all come from the Fluent catalogs, and this
// file fails if a served account template reintroduces the sentinel.

const root = fileURLToPath(new URL("..", import.meta.url))
const siteHeaders = {
  "X-Wikijump-Site-Id": "17",
  "X-Wikijump-Site-Slug": "test"
}

// Every Svelte template that serves the T28 account lifecycle: registration,
// login, settings, own/public profiles, and the user/admin error boundaries.
const accountTemplates = [
  "src/routes/[x+2d]/register/+page.svelte",
  "src/routes/[x+2d]/login/+page.svelte",
  "src/routes/[x+2d]/settings/+page.svelte",
  "src/routes/[x+2d]/user/+page.svelte",
  "src/routes/[x+2d]/user/[slug]/PageView.svelte",
  "src/routes/[x+2d]/user/+error.svelte",
  "src/routes/[x+2d]/admin/+error.svelte"
]

let previousWorkingDirectory
let vite
let client
let originalClientRequest
let routes

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  routes = {
    register: await vite.ssrLoadModule("/src/routes/[x+2d]/register/+page.server.ts"),
    user: await vite.ssrLoadModule("/src/routes/[x+2d]/user/+page.server.ts"),
    admin: await vite.ssrLoadModule("/src/routes/[x+2d]/admin/+page.server.ts")
  }
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

const pageRequest = (path) =>
  new Request(`https://wikijump.test${path}`, { headers: siteHeaders })

test("served account templates contain no UNTRANSLATED sentinel copy", async () => {
  for (const template of accountTemplates) {
    const source = await readFile(new URL(`../${template}`, import.meta.url), "utf8")
    assert.doesNotMatch(source, /UNTRANSLATED:/u, template)
  }
})

test("the inline email-validation message is localized in English and Japanese", async () => {
  const baseEn = await readFile(
    new URL("../../locales/fluent/base/en.ftl", import.meta.url),
    "utf8"
  )
  const baseJa = await readFile(
    new URL("../../locales/fluent/base/ja.ftl", import.meta.url),
    "utf8"
  )

  assert.ok(
    baseEn.includes("\n  .email-invalid = The email address is invalid.\n"),
    "en: error-form.email-invalid"
  )
  assert.ok(
    baseJa.includes("\n  .email-invalid = メールアドレスの形式が正しくありません。\n"),
    "ja: error-form.email-invalid"
  )
})

test("registration requests its inline validation and locale-label messages", async () => {
  const translateCalls = []
  client.request = async (method, params) => {
    if (method === "translate") {
      translateCalls.push(params)
      return {}
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  await routes.register.load({
    request: pageRequest("/-/register"),
    parent: async () => ({ user_session: undefined, locales: ["en"] })
  })

  const messages = translateCalls.at(0)?.messages ?? {}
  assert.ok(Object.hasOwn(messages, "error-form.email-invalid"))
  // The selector label reuses the advertised-language wording shared with
  // the account display settings, never a hard-coded English-only string.
  assert.ok(Object.hasOwn(messages, "dashboard-account.language"))
})

test("user error bodies carry localized generic-failure copy", async () => {
  const translateCalls = []
  client.request = async (method, params) => {
    if (method === "translate") {
      translateCalls.push(params)
      // Echo every requested key so the error body carries the same
      // message set the real catalog would resolve through fallback.
      return Object.fromEntries(
        Object.keys(params.messages ?? {}).map((key) => [key, key])
      )
    }
    if (method === "user_view") return { type: "user_missing" }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const thrown = await routes.user
    .load({
      request: pageRequest("/-/user"),
      cookies: { get: () => undefined },
      parent: async () => ({
        locales: ["en"],
        license_name: "CC BY-SA 4.0",
        license_url: "https://creativecommons.org/licenses/by-sa/4.0/"
      })
    })
    .then(
      () => assert.fail("missing-profile load must raise the error boundary"),
      (error) => error
    )

  assert.equal(thrown.status, 404)
  const messages = translateCalls.at(0)?.messages ?? {}
  assert.ok(Object.hasOwn(messages, "error"), "defaults carry the error title")
  assert.ok(
    Object.hasOwn(messages, "error-api.GENERIC"),
    "unexpected views stay localized"
  )
  assert.ok(Object.hasOwn(messages, "user-not-exist"))
  // The boundary template reads these keys, so the served error page can
  // never fall back to a developer sentinel.
  for (const key of ["error", "error-api.GENERIC", "user-not-exist"]) {
    assert.equal(thrown.body.internationalization?.[key], key, key)
  }
})

test("admin permission errors carry the localized denial copy", async () => {
  const translateCalls = []
  client.request = async (method, params) => {
    if (method === "translate") {
      translateCalls.push(params)
      return Object.fromEntries(
        Object.keys(params.messages ?? {}).map((key) => [key, key])
      )
    }
    if (method === "admin_view") {
      return { type: "admin_permissions", data: { html: "<p>denied fixture</p>" } }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const thrown = await routes.admin
    .load({
      request: pageRequest("/-/admin"),
      cookies: { get: () => undefined },
      parent: async () => ({
        locales: ["en"],
        license_name: "CC BY-SA 4.0",
        license_url: "https://creativecommons.org/licenses/by-sa/4.0/",
        site: {
          site_id: 17,
          slug: "test",
          name: "Admin Fixture Site",
          tagline: "",
          description: "",
          default_page: "start",
          welcome_page: "start",
          locale: "en",
          settings_revision: 1
        },
        site_settings: {
          membership: { application_enabled: false, password_enabled: false }
        }
      })
    })
    .then(
      () => assert.fail("denied admin load must raise the error boundary"),
      (error) => error
    )

  assert.equal(thrown.status, 401)
  assert.equal(thrown.body.view, "admin_permissions")
  assert.equal(
    Object.hasOwn(thrown.body.internationalization ?? {}, "admin-unauthorized"),
    true
  )
  const messages = translateCalls.at(0)?.messages ?? {}
  assert.ok(Object.hasOwn(messages, "admin-unauthorized"))
  assert.ok(
    Object.hasOwn(messages, "error-api.GENERIC"),
    "unexpected views stay localized"
  )
})
