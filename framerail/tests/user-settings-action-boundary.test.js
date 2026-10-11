// @ts-nocheck
import { strict as assert } from "node:assert"
import { fileURLToPath } from "node:url"
import test from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))

const requestEvent = ({ locales, sessionToken, submittedUser } = {}) => {
  const data = new FormData()
  if (locales !== undefined) {
    for (const locale of Array.isArray(locales) ? locales : [locales]) {
      data.append("locales", locale)
    }
  }
  if (submittedUser !== undefined) data.set("user", String(submittedUser))

  return {
    request: new Request("https://wikijump.test/-/settings?/display", {
      method: "POST",
      body: data
    }),
    cookies: {
      get(name) {
        assert.equal(name, "wikijump_token")
        return sessionToken
      }
    },
    getClientAddress: () => "192.0.2.63",
    locals: {
      requestContext: {
        sessionToken,
        siteId: 17
      }
    }
  }
}

test("user settings bind persistence to the server session actor", async () => {
  const previousWorkingDirectory = process.cwd()
  let vite
  let client
  let originalRequest

  try {
    process.chdir(root)
    vite = await createTestViteServer()
    ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
    const { loadUserSettings, userDisplaySettingsAction } = await vite.ssrLoadModule(
      "/src/lib/server/load/user-settings.ts"
    )
    originalRequest = client.request

    const calls = []
    let persistedLocales = ["en-US"]
    client.request = async (method, params, context) => {
      calls.push({ method, params, context })
      if (method === "session_get") {
        if (params[0] === "stale-session") {
          throw Object.assign(new Error("Session token is invalid"), { code: 3001 })
        }
        return {
          session_token: "issue-1063-session",
          user_id: 41,
          created_at: "2026-08-09T00:00:00Z",
          expires_at: "2026-08-10T00:00:00Z",
          ip_address: "192.0.2.63",
          user_agent: "issue 1063 test",
          restricted: false
        }
      }
      if (method === "user_edit") {
        persistedLocales = params.locales
        return { user_id: params.user, locales: params.locales }
      }
      if (method === "translate") {
        return {
          settings: "Settings",
          save: "Save",
          cancel: "Cancel",
          "user-profile-info.locales": "Display languages"
        }
      }
      throw new Error(`Unexpected Deepwell method ${method}`)
    }

    await assert.rejects(
      loadUserSettings(async () => ({ user_session: null })),
      (error) => error?.status === 303 && error?.location === "/-/login"
    )

    const invalid = await userDisplaySettingsAction(requestEvent())
    assert.equal(invalid.status, 400)
    assert.equal(calls.length, 0)

    const missingSession = await userDisplaySettingsAction(
      requestEvent({ locales: ["en-US"] })
    )
    assert.equal(missingSession.status, 401)
    assert.equal(calls.length, 0)

    const staleSession = await userDisplaySettingsAction(
      requestEvent({ locales: ["en-US"], sessionToken: "stale-session" })
    )
    assert.equal(staleSession.status, 401)
    assert.equal(staleSession.data.message, "Session token is invalid")
    assert.equal(staleSession.data.code, 3001)
    assert.deepEqual(
      calls.map(({ method }) => method),
      ["session_get"]
    )
    calls.length = 0

    const saved = await userDisplaySettingsAction(
      requestEvent({
        locales: ["ja_JP", "en-US", "ja_JP"],
        sessionToken: "issue-1063-session",
        submittedUser: 999
      })
    )
    assert.equal(saved.form.valid, true)
    assert.deepEqual(saved.form.data.locales, ["ja-JP", "en-US"])
    assert.deepEqual(
      calls.map(({ method }) => method),
      ["session_get", "user_edit"]
    )
    assert.deepEqual(calls[0].params, ["issue-1063-session"])
    assert.deepEqual(calls[1], {
      method: "user_edit",
      params: {
        user: 41,
        ip_address: "192.0.2.63",
        bypass_filter: false,
        forum_signature: null,
        locales: ["ja-JP", "en-US"]
      },
      context: {
        sessionToken: "issue-1063-session",
        siteId: 17
      }
    })

    const reloaded = await loadUserSettings(async () => ({
      locales: ["en-US", "en"],
      user_session: {
        user: {
          user_id: 41,
          locales: persistedLocales
        }
      }
    }))
    assert.deepEqual(reloaded.displaySettingsForm.data.locales, ["ja-JP", "en-US"])

    const duplicatePreferences = await loadUserSettings(async () => ({
      locales: ["en-US", "en"],
      user_session: {
        user: {
          user_id: 41,
          locales: ["ja-JP", "en-US", "ja-JP"]
        }
      }
    }))
    assert.deepEqual(duplicatePreferences.displaySettingsForm.data.locales, [
      "ja-JP",
      "en-US"
    ])

    const emptyPreferences = await loadUserSettings(async () => ({
      locales: ["en-US", "en"],
      user_session: {
        user: {
          user_id: 41,
          locales: []
        }
      }
    }))
    assert.deepEqual(emptyPreferences.displaySettingsForm.data.locales, ["en"])
  } finally {
    if (client && originalRequest) client.request = originalRequest
    if (vite) await vite.close()
    process.chdir(previousWorkingDirectory)
  }
})
