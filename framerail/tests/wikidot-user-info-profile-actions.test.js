// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

import { loadWikidotUserInfo } from "../src/lib/server/wikidot-user-info.js"
import { showUserProfileActionAuthenticationGuard } from "../src/lib/user-profile-action-auth-guard.js"

const root = fileURLToPath(new URL("..", import.meta.url))

let previousWorkingDirectory
let vite
let render
let userInfoPage

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ render } = await vite.ssrLoadModule("svelte/server"))
  ;({ default: userInfoPage } = await vite.ssrLoadModule(
    "/src/routes/user[x+3a]info/[target]/+page.svelte"
  ))
})

after(async () => {
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

/**
 * Retained live case: userinfo-target-routes-live-20260810.json,
 * `userinfo-target-the-administrator-anonymous`.
 *
 * @type {Record<string, unknown>}
 */
const publicUser = {
  user_id: 169306,
  user_type: "regular",
  created_at: "2008-07-19T21:26:10Z",
  updated_at: "2026-08-10T00:00:00Z",
  deleted_at: null,
  name: `The <Administrator> & "owner" <script>alert(1)</script>`,
  slug: "the-administrator",
  avatar_s3_hash: null,
  email: "not-public-email-marker",
  email_verified_at: null,
  password: "not-public",
  real_name: "Not Public",
  biography: "Not Public",
  locales: ["en"]
}

const loadFoundProfile = () =>
  loadWikidotUserInfo({
    siteId: 7,
    locales: ["en"],
    target: "the-administrator",
    userView: async () => ({ type: "user_found", data: { user: publicUser } })
  })

test("UserInfo projects the live action row as three evidenced controls", async () => {
  const data = await loadFoundProfile()

  assert.deepEqual(data.privateMessageControl, {
    label: "Write private message",
    authenticationPath: "/-/login"
  })
  assert.deepEqual(data.contactControl, { label: "Add to contacts" })
  assert.deepEqual(data.abuseReportControl, { label: "flag user as abusive" })
  // Stable target identity, never the mutable display name.
  assert.equal(data.user.userId, 169306)
})

test("UserInfo keeps the public projection free of private account state", async () => {
  const data = await loadFoundProfile()
  const serialized = JSON.stringify(data)

  for (const marker of [
    "not-public",
    "email",
    "password",
    "session",
    "biography",
    "real_name"
  ]) {
    assert.equal(serialized.includes(marker), false, `${marker} leaked`)
  }
})

test("an anonymous profile renders the actionable action row", async () => {
  const data = await loadFoundProfile()
  const body = render(userInfoPage, { props: { data } }).body

  // The private-message label is a real navigation, not inert text.
  assert.match(body, /<a[^>]*data-action="write-private-message"[^>]*>/u)
  assert.match(body, /href="\/-\/login"/u)
  assert.doesNotMatch(body, /data-redacted-control/u)
  // The abuse-report entry point exists and is addressable by stable ID.
  assert.match(body, /<a[^>]*data-action="flag-user-as-abusive"[^>]*>/u)
  assert.match(body, /data-target-user-id="169306"/u)
  assert.match(body, /data-action="add-to-contacts"/u)
  // Escaped identity, never raw authored markup.
  assert.match(body, /The &lt;Administrator> &amp; "owner"/u)
  assert.doesNotMatch(body, /<script>alert\(1\)/u)
  assert.doesNotMatch(body, /<Administrator>/u)
})

test("an authenticated viewer gets no dead profile action", async () => {
  const data = await loadFoundProfile()
  const body = render(userInfoPage, {
    props: { data: { ...data, user_session: { user: { user_id: 42 } } } }
  }).body

  for (const action of [
    "write-private-message",
    "add-to-contacts",
    "flag-user-as-abusive"
  ]) {
    assert.equal(body.includes(`data-action="${action}"`), false, `${action} rendered`)
  }
  // The profile itself still renders, so the negative check is not vacuous.
  assert.match(body, /profile-title/u)
  assert.match(body, /The &lt;Administrator> &amp; "owner"/u)
})

test("a missing target renders no action row and no destination", async () => {
  for (const target of ["0", "-1"]) {
    const data = await loadWikidotUserInfo({
      siteId: 7,
      locales: ["en"],
      target,
      userView: async () => ({ type: "user_missing", data: undefined })
    })
    const body = render(userInfoPage, { props: { data } }).body

    assert.match(body, /User does not exist\./u)
    assert.doesNotMatch(body, /data-action=/u)
    assert.doesNotMatch(body, /\/-\/login/u)
  }
})

test("the profile action guard only opens the frozen dialog", () => {
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
  // No request, relation, report, or message state is reachable from the guard.
  assert.equal(Object.keys(popupState.current).length, 4)
})
