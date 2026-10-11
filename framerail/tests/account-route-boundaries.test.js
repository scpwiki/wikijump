// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const siteHeaders = {
  "X-Wikijump-Site-Id": "17",
  "X-Wikijump-Site-Slug": "test"
}
const parentData = {
  locales: ["en-US", "en"],
  license_name: "CC BY-SA 4.0",
  license_url: "https://creativecommons.org/licenses/by-sa/4.0/",
  user_session: null
}

let previousWorkingDirectory
let vite
let client
let originalClientRequest
let routes
let render
let userProfilePage

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  ;({ render } = await vite.ssrLoadModule("svelte/server"))
  ;({ default: userProfilePage } = await vite.ssrLoadModule(
    "/src/routes/[x+2d]/user/[slug]/PageView.svelte"
  ))
  originalClientRequest = client.request
  routes = {
    login: await vite.ssrLoadModule("/src/routes/[x+2d]/login/+page.server.ts"),
    logout: await vite.ssrLoadModule("/src/routes/[x+2d]/logout/+page.server.ts"),
    register: await vite.ssrLoadModule("/src/routes/[x+2d]/register/+page.server.ts"),
    settings: await vite.ssrLoadModule("/src/routes/[x+2d]/settings/+page.server.ts"),
    preload: await vite.ssrLoadModule("/src/lib/server/load/preload.ts"),
    user: await vite.ssrLoadModule("/src/routes/[x+2d]/user/+page.server.ts"),
    userSlug: await vite.ssrLoadModule("/src/routes/[x+2d]/user/[slug]/+page.server.ts")
  }
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

const pageRequest = (path) =>
  new Request(`https://wikijump.test${path}`, { headers: siteHeaders })

test("account route loads expose their public SvelteKit page data", async () => {
  assert.equal(routes.preload.pageRouteProvidesPreload("/[x+2d]/user/[slug]"), true)
  const userViewNames = []
  const translateCalls = []
  client.request = async (method, params) => {
    if (method === "translate") {
      translateCalls.push(params)
      return {}
    }
    if (method === "user_view") {
      userViewNames.push(params.user)
      return {
        type: "user_found",
        data: {
          user: {
            user_id: 41,
            user_type: "regular",
            created_at: "2026-08-10T00:00:00Z",
            updated_at: null,
            deleted_at: null,
            name: "Account Fixture",
            slug: "account-fixture",
            avatar_s3_hash: null,
            email: "target-private@example.test",
            birthday: "2000-01-01",
            biography: "private target biography",
            website: null,
            user_page: null
          }
        }
      }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const publicProfileParent = {
    site: {
      site_id: 17,
      slug: "test",
      name: "Profile Fixture Site",
      locale: "en",
      membership_by_password: true,
      password_configured: true
    },
    site_settings: {
      membership: {
        password_enabled: true,
        password_configured: true,
        private_marker: "do-not-serialize"
      },
      private_marker: "private-site-settings"
    },
    promoted_sites: [{ slug: "private-promotion", name: "Private promotion" }],
    site_file_domain: "files.wikijump.test",
    license_name: "CC BY-SA 4.0",
    license_url: "https://creativecommons.org/licenses/by-sa/4.0/",
    license_kind: "standard",
    license_html: null,
    locales: ["en"],
    user_session: {
      user: {
        user_id: 41,
        name: "Private viewer",
        slug: "private-viewer",
        email: "private-viewer@example.test"
      }
    }
  }

  const login = await routes.login.load({
    request: pageRequest("/-/login"),
    parent: async () => parentData
  })
  const logout = await routes.logout.load({
    request: pageRequest("/-/logout"),
    parent: async () => parentData
  })
  const register = await routes.register.load({
    request: pageRequest("/-/register"),
    parent: async () => parentData
  })
  const settings = await routes.settings.load({
    parent: async () => ({
      ...parentData,
      user_session: {
        user: {
          user_id: 41,
          locales: ["en-US", "ja-JP"],
          forum_signature: "**Stored signature**"
        }
      }
    })
  })
  const user = await routes.user.load({
    request: pageRequest("/-/user"),
    cookies: { get: () => undefined },
    parent: async () => parentData
  })
  const userSlug = await routes.userSlug.load({
    params: { slug: "account-fixture" },
    request: pageRequest("/-/user/account-fixture"),
    cookies: { get: () => undefined },
    parent: async () => publicProfileParent
  })

  assert.equal(login.isLoggedIn, false)
  assert.equal(login.loginForm.valid, false)
  assert.equal(logout.isLoggedIn, false)
  assert.equal(register.isLoggedIn, false)
  assert.equal(register.registerForm.valid, false)
  assert.deepEqual(settings.displaySettingsForm.data.locales, ["en-US", "ja-JP"])
  assert.equal(settings.displaySettingsForm.data.signature, "**Stored signature**")
  const settingsTranslate = translateCalls.find(
    (params) =>
      params.messages?.settings && params.messages?.["user-profile-info.locales"]
  )
  assert.deepEqual(settingsTranslate?.strip_message_keys, [])
  assert.equal(user.view, "user_found")
  assert.equal(user.user.slug, "account-fixture")
  const userTranslate = translateCalls.find(
    (params) => params.messages?.["user-profile-info"]
  )
  assert.ok(userTranslate)
  assert.equal(userSlug.view, "user_found")
  assert.equal(Object.hasOwn(userSlug.user, "email"), false)
  assert.equal(Object.hasOwn(userSlug.user, "birthday"), false)
  assert.equal(Object.hasOwn(userSlug.user, "biography"), false)
  assert.deepEqual(userSlug.site, {
    slug: "test",
    name: "Profile Fixture Site",
    locale: "en"
  })
  for (const key of [
    "site_settings",
    "promoted_sites",
    "user_session",
    "locales",
    "userEditForm"
  ]) {
    assert.equal(Object.hasOwn(userSlug, key), false, key)
  }
  const renderedProfile = render(userProfilePage, {
    props: {
      data: userSlug,
      userData: {
        name: "Account Fixture",
        website: null,
        userPage: null,
        locales: ""
      }
    }
  }).body
  assert.doesNotMatch(renderedProfile, /textarea|UNTRANSLATED:/u)
  assert.doesNotMatch(
    renderedProfile,
    /private-site-settings|do-not-serialize|private-viewer@example\.test/u
  )
  assert.deepEqual(userViewNames, [undefined, "account-fixture"])
})

test("public profile websites render safe actionable links and leave user pages as text", () => {
  const data = {
    site: { name: "Profile fixture" },
    site_file_domain: "files.wikijump.test",
    user: { user_id: 41, name: "Profile fixture", slug: "profile-fixture" },
    internationalization: { "user-profile-info.website": "Website:" }
  }
  const userData = (website) => ({
    name: "Profile fixture",
    website,
    userPage: "other-site:profile-fixture"
  })

  const normalized = render(userProfilePage, {
    props: { data, userData: userData("scp-wiki.wikidot.com/seekgull") }
  }).body
  assert.match(
    normalized,
    /<a class="[^"]*website-link[^"]*" href="https:\/\/scp-wiki\.wikidot\.com\/seekgull">scp-wiki\.wikidot\.com\/seekgull<\/a>/u
  )
  assert.match(
    normalized,
    /<span class="user-attribute-value[^"]*">other-site:profile-fixture<\/span>/u
  )

  const unsafe = render(userProfilePage, {
    props: { data, userData: userData(["javascript", "alert(1)"].join(":")) }
  }).body
  assert.match(
    unsafe,
    /<span class="user-attribute-value[^"]*">javascript:alert\(1\)<\/span>/u
  )
  assert.doesNotMatch(unsafe, /href="javascript:alert\(1\)"/u)
})

test("display settings persist the forum signature through the existing account mutation", async () => {
  const calls = []
  client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    if (method === "session_get") {
      return {
        session_token: "account-session",
        user_id: 41,
        created_at: "2026-08-10T00:00:00Z",
        expires_at: "2026-08-11T00:00:00Z",
        ip_address: "192.0.2.41",
        user_agent: "account route test",
        restricted: false
      }
    }
    if (method === "user_edit") return { user_id: 41 }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const formData = new FormData()
  formData.set("locales", "en-US en")
  formData.set("signature", "**Forum signature**\nSecond line")
  const result = await routes.settings.actions.display({
    request: new Request("https://wikijump.test/-/settings?/display", {
      method: "POST",
      headers: siteHeaders,
      body: formData
    }),
    cookies: { get: () => "account-session" },
    getClientAddress: () => "192.0.2.41",
    locals: {
      requestContext: {
        siteId: 17,
        page: "-/settings",
        sessionToken: "account-session"
      }
    }
  })

  assert.equal(result.form.valid, true)
  assert.deepEqual(calls, [
    { method: "session_get", params: ["account-session"], context: undefined },
    {
      method: "user_edit",
      params: {
        user: 41,
        ip_address: "192.0.2.41",
        bypass_filter: false,
        locales: ["en-US", "en"],
        forum_signature: "**Forum signature**\nSecond line"
      },
      context: { siteId: 17, page: "-/settings", sessionToken: "account-session" }
    }
  ])

  calls.length = 0
  const tooManyLines = new FormData()
  tooManyLines.set("locales", "en-US en")
  tooManyLines.set("signature", "one\ntwo\nthree\nfour\nfive")
  const rejected = await routes.settings.actions.display({
    request: new Request("https://wikijump.test/-/settings?/display", {
      method: "POST",
      headers: siteHeaders,
      body: tooManyLines
    }),
    cookies: { get: () => "account-session" },
    getClientAddress: () => "192.0.2.41",
    locals: { requestContext: { siteId: 17, page: "-/settings" } }
  })
  assert.equal(rejected.status, 400)
  assert.equal(calls.length, 0)
})

test("login shares a session across native Wikijump wiki hosts but not custom domains", async () => {
  const previousFramerailEnv = process.env.FRAMERAIL_ENV
  process.env.FRAMERAIL_ENV = "local"

  try {
    client.request = async (method, params) => {
      if (method === "login") {
        assert.equal(params.name_or_email, "account-fixture")
        assert.equal(params.password, "fixture-password")
        return { session_token: "shared-account-session", needs_mfa: false }
      }
      if (method === "session_get") {
        assert.deepEqual(params, ["shared-account-session"])
        return {
          session_token: "shared-account-session",
          user_id: 41,
          created_at: "2026-08-10T00:00:00Z",
          expires_at: "2026-08-11T00:00:00Z",
          ip_address: "192.0.2.41",
          user_agent: "account route test",
          restricted: false
        }
      }
      throw new Error(`Unexpected Deepwell method ${method}`)
    }

    const login = async (origin) => {
      const formData = new FormData()
      formData.set("nameOrEmail", "account-fixture")
      formData.set("password", "fixture-password")
      const setCookies = []
      const result = await routes.login.actions.default({
        request: new Request(`${origin}/-/login`, {
          method: "POST",
          headers: siteHeaders,
          body: formData
        }),
        getClientAddress: () => "192.0.2.41",
        cookies: {
          set: (name, value, options) => setCookies.push({ name, value, options })
        }
      })
      assert.equal(result.isLoggedIn, true)
      assert.equal(result.needsMfa, false)
      assert.equal(result.session_token, undefined)
      assert.equal(setCookies.length, 1)
      return setCookies[0]
    }

    const nativeCookie = await login("https://scp-wiki.wikijump.localhost")
    assert.equal(nativeCookie.name, "wikijump_token")
    assert.equal(nativeCookie.value, "shared-account-session")
    assert.equal(nativeCookie.options.domain, "wikijump.localhost")
    assert.equal(nativeCookie.options.path, "/")
    assert.equal(nativeCookie.options.httpOnly, true)
    assert.equal(nativeCookie.options.secure, true)
    assert.equal(nativeCookie.options.sameSite, "lax")

    const customDomainCookie = await login("https://wiki.example.test")
    assert.equal(customDomainCookie.options.domain, undefined)
  } finally {
    if (previousFramerailEnv === undefined) delete process.env.FRAMERAIL_ENV
    else process.env.FRAMERAIL_ENV = previousFramerailEnv
  }
})

test("register binds account creation to the request address and redacts submitted passwords", async () => {
  const calls = []
  client.request = async (method, params) => {
    calls.push({ method, params })
    if (method === "user_create") {
      return { user_id: 42, name: "registration-fixture", slug: "registration-fixture" }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const password = "registration-password-fixture"
  const formData = new FormData()
  formData.set("username", "registration-fixture")
  formData.set("email", "registration-fixture@example.invalid")
  formData.set("password", password)
  formData.set("confirmPassword", password)
  formData.append("locale", "en")

  const result = await routes.register.actions.default({
    request: new Request("https://wikijump.test/-/register", {
      method: "POST",
      headers: siteHeaders,
      body: formData
    }),
    getClientAddress: () => "192.0.2.42"
  })

  assert.equal(result.isRegistered, true)
  assert.equal(result.form.valid, true)
  assert.equal(result.form.data.password, "")
  assert.equal(result.form.data.confirmPassword, "")
  assert.equal(JSON.stringify(result).includes(password), false)
  assert.deepEqual(calls, [
    {
      method: "user_create",
      params: {
        user_type: "regular",
        name: "registration-fixture",
        email: "registration-fixture@example.invalid",
        locales: ["en"],
        password,
        ip_address: "192.0.2.42",
        bypass_filter: false,
        bypass_email_verification: false
      }
    }
  ])
})

test("register offers only deployment-advertised locales and rejects others before account creation", async () => {
  const calls = []
  client.request = async (method, params) => {
    calls.push({ method, params })
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const password = "registration-password-fixture"
  const formData = new FormData()
  formData.set("username", "registration-fixture")
  formData.set("email", "registration-fixture@example.invalid")
  formData.set("password", password)
  formData.set("confirmPassword", password)
  formData.append("locale", "ja")
  formData.append("locale", "xx-unadvertised")

  const result = await routes.register.actions.default({
    request: new Request("https://wikijump.test/-/register", {
      method: "POST",
      headers: siteHeaders,
      body: formData
    }),
    getClientAddress: () => "192.0.2.42"
  })

  assert.equal(result.data.form.valid, false)
  assert.ok(result.data.form.errors.locale)
  assert.deepEqual(calls, [])
})

test("register load advertises Japanese alongside the supported UI locales", async () => {
  client.request = async (method) => {
    if (method === "translate") return {}
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const register = await routes.register.load({
    request: pageRequest("/-/register"),
    parent: async () => ({ user_session: undefined, locales: ["en"] })
  })

  assert.deepEqual(
    register.userInterfaceLocales.map(({ value }) => value),
    ["en", "ja", "ko", "pl", "vi", "zh-Hans"]
  )
})

test("legacy user slug route fails closed for imported profiles", async () => {
  client.request = async (method) => {
    if (method === "translate") return {}
    if (method === "user_view") {
      return {
        type: "user_found",
        data: {
          user: {
            user_id: 169306,
            user_type: "wikidot",
            created_at: "2008-07-19T21:26:10Z",
            fetched_at: "2026-08-13T00:00:00Z",
            is_deleted: false,
            name: "The Administrator",
            slug: "the-administrator",
            avatar_s3_hash: null,
            real_name: null,
            gender: null,
            birthday: null,
            location: null,
            biography: null,
            website: null,
            karma: 3,
            is_pro: false
          }
        }
      }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  await assert.rejects(
    routes.userSlug.load({
      params: { slug: "the-administrator" },
      request: pageRequest("/-/user/the-administrator"),
      cookies: { get: () => undefined },
      parent: async () => ({
        ...parentData,
        site: { slug: "test", name: "Test site", locale: "en" },
        site_file_domain: "files.wikijump.test"
      })
    }),
    (error) => {
      assert.equal(error.status, 404)
      assert.equal(error.body.view, "user_missing")
      assert.equal("user" in error.body, false)
      return true
    }
  )
})

test("logout and user-edit route actions bind mutations to the server session", async () => {
  const calls = []
  client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    if (method === "logout") return null
    if (method === "session_get") {
      return {
        session_token: "account-session",
        user_id: 41,
        created_at: "2026-08-10T00:00:00Z",
        expires_at: "2026-08-11T00:00:00Z",
        ip_address: "192.0.2.41",
        user_agent: "account route test",
        restricted: false
      }
    }
    if (method === "user_edit") {
      return {
        user_id: 987654321,
        slug: "rpc-user-edit-result-must-remain-private"
      }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const deletedCookies = []
  assert.deepEqual(
    await routes.logout.actions.logout({
      request: pageRequest("/-/logout?/logout"),
      cookies: {
        get: () => "account-session",
        delete: (name, options) => deletedCookies.push({ name, options })
      }
    }),
    { success: true }
  )

  const editData = new FormData()
  editData.set("name", "Updated Account")
  const edited = await routes.user.actions.userEdit({
    request: new Request("https://wikijump.test/-/user?/userEdit", {
      method: "POST",
      headers: siteHeaders,
      body: editData
    }),
    cookies: { get: () => "account-session" },
    getClientAddress: () => "192.0.2.41",
    locals: {
      requestContext: { siteId: 17, page: "-/user", sessionToken: "account-session" }
    }
  })

  assert.equal(edited.form.valid, true)
  assert.deepEqual(Object.keys(edited), ["form"])
  assert.equal("res" in edited, false)
  assert.equal(
    JSON.stringify(edited).includes("rpc-user-edit-result-must-remain-private"),
    false
  )
  assert.deepEqual(deletedCookies, [
    {
      name: "wikijump_token",
      options: { path: "/", httpOnly: true, secure: true, sameSite: "lax" }
    }
  ])
  assert.deepEqual(calls, [
    { method: "logout", params: ["account-session"], context: undefined },
    { method: "session_get", params: ["account-session"], context: undefined },
    {
      method: "user_edit",
      params: {
        user: 41,
        ip_address: "192.0.2.41",
        bypass_filter: false,
        name: "Updated Account"
      },
      context: { siteId: 17, page: "-/user", sessionToken: "account-session" }
    }
  ])
})

test("logout clears a stale browser session when Deepwell reports it invalid", async () => {
  client.request = async (method) => {
    assert.equal(method, "logout")
    throw Object.assign(new Error("Session token is invalid"), { code: 3001 })
  }

  const deletedCookies = []
  const result = await routes.logout.actions.logout({
    request: pageRequest("/-/logout?/logout"),
    cookies: {
      get: () => "stale-account-session",
      delete: (name, options) => deletedCookies.push({ name, options })
    }
  })

  assert.deepEqual(result, { success: true })
  assert.deepEqual(deletedCookies, [
    {
      name: "wikijump_token",
      options: { path: "/", httpOnly: true, secure: true, sameSite: "lax" }
    }
  ])
})

test("logout retains the browser session when invalidation otherwise fails", async () => {
  const errors = [
    new Error("Deepwell transport unavailable"),
    Object.assign(new Error("Database query failed"), { code: 1200 }),
    Object.assign(new Error("Unexpected logout failure"), { code: 3999 })
  ]
  const statuses = []
  const deletedCookies = []

  for (const error of errors) {
    client.request = async (method) => {
      assert.equal(method, "logout")
      throw error
    }

    const result = await routes.logout.actions.logout({
      request: pageRequest("/-/logout?/logout"),
      cookies: {
        get: () => "active-account-session",
        delete: (name, options) => deletedCookies.push({ name, options })
      }
    })
    statuses.push(result.status)
  }

  assert.deepEqual(statuses, [500, 500, 500])
  assert.deepEqual(deletedCookies, [])
})

test("logout without a browser session remains a bad request", async () => {
  client.request = async (method) => {
    assert.equal(method, "translate")
    return { "error-api.NOT_LOGGED_IN": "Not logged in" }
  }

  const result = await routes.logout.actions.logout({
    request: pageRequest("/-/logout?/logout"),
    cookies: {
      get: () => undefined,
      delete: () => assert.fail("missing session must not delete a cookie")
    }
  })

  assert.equal(result.status, 400)
  assert.deepEqual(result.data, { message: "Not logged in" })
})
