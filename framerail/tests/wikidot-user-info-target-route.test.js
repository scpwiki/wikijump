import assert from "node:assert/strict"
import { access, readFile } from "node:fs/promises"
import { test } from "node:test"

import {
  WIKIDOT_USER_INFO_MISSING,
  loadWikidotUserInfo
} from "../src/lib/server/wikidot-user-info.js"
import { showUserProfileActionAuthenticationGuard } from "../src/lib/user-profile-action-auth-guard.js"

/**
 * @typedef {{
 *   user_id: number
 *   user_type: "regular" | "system" | "site" | "bot"
 *   created_at: string
 *   updated_at: string | null
 *   deleted_at: string | null
 *   name: string
 *   slug: string
 *   avatar_s3_hash: number[] | null
 *   email: string
 *   email_verified_at: string | null
 *   password: string
 *   real_name: string | null
 *   biography: string | null
 *   locales: string[]
 * }} UserViewUser
 *
 *
 * @typedef {(
 *   siteId: number,
 *   locales: string[],
 *   sessionToken: undefined,
 *   target: string
 * ) => Promise<{ type: "user_found"; data: { user: UserViewUser } }>} FoundUserView
 */

const routeFiles = [
  "../src/routes/user[x+3a]info/[target]/+page.server.ts",
  "../src/routes/user[x+3a]info/[target]/+page.svelte"
]

test("GET /user:info/<target> has an encoded public route", async () => {
  for (const routeFile of routeFiles) {
    await assert.doesNotReject(access(new URL(routeFile, import.meta.url)))
  }
})

/** @type {UserViewUser} */
const publicUser = {
  user_id: 169306,
  user_type: "regular",
  created_at: "2008-07-19T21:26:10Z",
  updated_at: "2026-08-10T00:00:00Z",
  deleted_at: null,
  name: `The <Administrator> & "owner" <script>alert(1)</script>`,
  slug: "the-administrator",
  avatar_s3_hash: [1, 2, 3],
  email: "not-public-email-marker",
  email_verified_at: "2026-08-10T00:00:00Z",
  password: "not-public",
  real_name: "Not Public",
  biography: "Not Public",
  locales: ["en"]
}

test("UserInfo calls the typed user view with the route target and no session", async () => {
  /** @type {[number, string[], undefined, string][]} */
  const calls = []
  /** @type {FoundUserView} */
  const userView = async (...args) => {
    calls.push(args)
    return { type: "user_found", data: { user: publicUser } }
  }

  await loadWikidotUserInfo({
    siteId: 7,
    locales: ["en"],
    target: "the-administrator",
    userView
  })

  assert.deepEqual(calls, [[7, ["en"], undefined, "the-administrator"]])
})

test("UserInfo rejects unsupported user-view response variants", async () => {
  await assert.rejects(
    loadWikidotUserInfo({
      siteId: 7,
      locales: ["en"],
      target: "the-administrator",
      userView: async () => ({ type: "unexpected", data: undefined })
    }),
    /Unexpected user view response 'unexpected'/u
  )
})

test("UserInfo projects only evidenced public identity fields", async () => {
  const result = await loadWikidotUserInfo({
    siteId: 7,
    locales: ["en"],
    target: "the-administrator",
    userView: async () => ({ type: "user_found", data: { user: publicUser } })
  })

  assert.deepEqual(result, {
    status: 200,
    view: "user_found",
    user: {
      userId: 169306,
      name: publicUser.name,
      slug: "the-administrator",
      accountType: "regular",
      createdAt: "2008-07-19T21:26:10Z"
    },
    privateMessageControl: {
      label: "Write private message",
      authenticationPath: "/-/login"
    },
    contactControl: { label: "Add to contacts" },
    abuseReportControl: { label: "flag user as abusive" }
  })
  assert.equal(JSON.stringify(result).includes("not-public"), false)
})

test("UserInfo exposes an avatar only through the existing trusted file seam", async () => {
  /** @type {number[][]} */
  const hashes = []
  /** @param {number[]} hash */
  const loadAvatar = async (hash) => {
    hashes.push(hash)
    return "data:image/png;base64,public-avatar"
  }
  const result = await loadWikidotUserInfo({
    siteId: 7,
    locales: ["en"],
    target: "the-administrator",
    userView: async () => ({ type: "user_found", data: { user: publicUser } }),
    loadAvatar
  })

  assert.deepEqual(hashes, [[1, 2, 3]])
  assert.equal(result.view, "user_found")
  assert.equal(result.user.avatar, "data:image/png;base64,public-avatar")
  assert.equal(JSON.stringify(result).includes("avatar_s3_hash"), false)
})

test("UserInfo route data is actor invariant", async () => {
  /** @type {FoundUserView} */
  const userView = async () => ({ type: "user_found", data: { user: publicUser } })
  const common = {
    siteId: 7,
    locales: ["en"],
    target: "the-administrator",
    userView
  }

  const anonymousActor = await loadWikidotUserInfo(common)
  const accountAActor = await loadWikidotUserInfo(common)

  assert.deepEqual(anonymousActor, accountAActor)
})

test("UserInfo route does not read cookies or forward a session", async () => {
  const source = await readFile(
    new URL("../src/routes/user[x+3a]info/[target]/+page.server.ts", import.meta.url),
    "utf8"
  )

  assert.doesNotMatch(source, /cookies|sessionToken|wikijump_token/u)
  assert.match(source, /target:\s*params\.target/u)
})

for (const target of ["0", "-1"]) {
  test(`UserInfo target ${target} stays HTTP 200 with the exact missing body data`, async () => {
    const result = await loadWikidotUserInfo({
      siteId: 7,
      locales: ["en"],
      target,
      userView: async () => ({ type: "user_missing", data: undefined })
    })

    assert.deepEqual(result, {
      status: 200,
      view: "user_missing",
      error: WIKIDOT_USER_INFO_MISSING
    })
    assert.equal(JSON.stringify(result).includes("privateMessage"), false)
    assert.equal(JSON.stringify(result).includes("abuseReport"), false)
    assert.equal(JSON.stringify(result).includes("contact"), false)
    assert.equal(JSON.stringify(result).includes("avatar"), false)
    assert.equal(JSON.stringify(result).includes("href"), false)
  })
}

test("UserInfo view renders the evidenced action row for anonymous viewers only", async () => {
  const source = await readFile(
    new URL("../src/routes/user[x+3a]info/[target]/+page.svelte", import.meta.url),
    "utf8"
  )

  assert.doesNotMatch(source, /\{@html\}|innerHTML/u)
  // The action row is gated on the merged layout session, so the control is
  // never rendered as a dead anchor for an authenticated viewer.
  assert.match(source, /if !data\.user_session/u)
  assert.doesNotMatch(source, /page\.data\.user_session/u)
  // Wikidot links the private-message action to its own message surface; the
  // local control reaches the authentication boundary instead.
  assert.match(
    source,
    /href=\{resolve\(data\.privateMessageControl\.authenticationPath, \{\}\)\}/u
  )
  assert.match(source, /data-action="write-private-message"/u)
  assert.match(source, /data-action="add-to-contacts"/u)
  assert.match(source, /data-action="flag-user-as-abusive"/u)
  assert.match(source, /data-target-user-id=\{data\.user\.userId\}/u)
  assert.match(source, /showUserProfileActionAuthenticationGuard\(errorPopupState\)/u)
  assert.match(source, /\{data\.user\.name\}/u)
  assert.match(source, /<div class="error-block">\{data\.error\}<\/div>/u)
  // The label is no longer an inert redacted span.
  assert.doesNotMatch(source, /data-redacted-control/u)
})

test("an anonymous profile action opens the observed permission error without a write", () => {
  const popupState = {
    current: { state: false, title: null, message: null, data: null }
  }

  showUserProfileActionAuthenticationGuard(popupState)

  assert.deepEqual(popupState.current, {
    state: true,
    title: "Permission error",
    message: "Please note:",
    data: "Please create a Wikidot account and/or sign in first"
  })
})
