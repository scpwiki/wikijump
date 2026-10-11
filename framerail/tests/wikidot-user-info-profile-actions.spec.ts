import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"
import type { APIRequestContext } from "./hermetic-playwright"

const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`
const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-authenticated-session-token"
}
const PROFILE_PATH = "/user:info/profile-action-probe"
const TARGET_USER_ID = "987658"

const privateMessageAction = '[data-action="write-private-message"]'
const contactAction = '[data-action="add-to-contacts"]'
const flagAction = '[data-action="flag-user-as-abusive"]'

const resetWriteRequests = async (request: APIRequestContext) => {
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
}

/** Reads (and clears) the shared fixture write ledger. */
const recordedWriteRequests = async (request: APIRequestContext) => {
  const response = await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  const groups = (await response.json()) as Record<string, unknown[]>
  return Object.entries(groups).flatMap(([group, requests]) =>
    requests.length > 0 ? [`${group}: ${JSON.stringify(requests)}`] : []
  )
}

test("an anonymous profile exposes the evidenced action row", async ({ page }) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(PROFILE_PATH)

  // The private-message label is actionable rather than inert text.
  const privateMessage = page.locator(privateMessageAction)
  await expect(privateMessage).toBeVisible()
  await expect(privateMessage).toHaveText("Write private message")
  await expect(privateMessage).toHaveAttribute("href", "/-/login")

  // Both guarded actions exist and address the profile by stable user ID.
  const flag = page.locator(flagAction)
  await expect(flag).toBeVisible()
  await expect(flag).toHaveText("flag user as abusive")
  await expect(flag).toHaveAttribute("data-target-user-id", TARGET_USER_ID)
  const contact = page.locator(contactAction)
  await expect(contact).toBeVisible()
  await expect(contact).toHaveAttribute("data-target-user-id", TARGET_USER_ID)

  // The profile itself still renders, so the checks above are not vacuous.
  await expect(page.locator("h1.profile-title")).toHaveText("Profile Action Probe")
})

test("activating flag user as abusive anonymously stays on the profile and writes nothing", async ({
  page,
  request
}) => {
  await resetWriteRequests(request)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const actionRequests: string[] = []
  page.on("request", (request) => {
    const url = new URL(request.url())
    if (url.search.startsWith("?/")) actionRequests.push(url.search)
  })

  await page.goto(PROFILE_PATH)
  await waitForSvelteDelegatedHandler(page, flagAction)
  await page.locator(flagAction).click()

  // The frozen live guard: same page, no navigation, no report.
  await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
  await expect(page.locator("#owindow-1 .title")).toHaveText("Permission error")
  await expect(page.locator("#modal-title")).toHaveText("Please note:")
  await expect(page.locator(".modal-message-extra")).toHaveText(
    "Please create a Wikidot account and/or sign in first"
  )
  await expect(page).toHaveURL(new RegExp(`${PROFILE_PATH}$`))
  expect(actionRequests).toEqual([])

  // Closing the guard leaves no report, contact, or message behind.
  await page.keyboard.press("Escape")
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  expect(await recordedWriteRequests(request)).toEqual([])
})

test("activating Add to contacts anonymously stays on the profile and writes nothing", async ({
  page,
  request
}) => {
  await resetWriteRequests(request)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const actionRequests: string[] = []
  page.on("request", (request) => {
    const url = new URL(request.url())
    if (url.search.startsWith("?/")) actionRequests.push(url.search)
  })

  await page.goto(PROFILE_PATH)
  await waitForSvelteDelegatedHandler(page, contactAction)
  await page.locator(contactAction).click()

  await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
  await expect(page.locator("#owindow-1 .title")).toHaveText("Permission error")
  await expect(page.locator("#modal-title")).toHaveText("Please note:")
  await expect(page).toHaveURL(new RegExp(`${PROFILE_PATH}$`))
  expect(actionRequests).toEqual([])
  expect(await recordedWriteRequests(request)).toEqual([])
})

test("activating the private-message action reaches the sign-in boundary", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const posts: string[] = []
  page.on("request", (request) => {
    if (request.method() === "POST") posts.push(request.url())
  })

  await page.goto(PROFILE_PATH)
  await page.locator(privateMessageAction).click()

  // Live Wikidot asks an anonymous visitor to sign in before composing.
  await expect(page).toHaveURL(/\/-\/login$/u)
  await expect(page.locator("#login-name-or-email")).toBeVisible()
  expect(posts).toEqual([])
})

test("an authenticated viewer gets no dead profile action", async ({ page }) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto(PROFILE_PATH)

  await expect(page.locator("h1.profile-title")).toHaveText("Profile Action Probe")
  await expect(page.locator(privateMessageAction)).toHaveCount(0)
  await expect(page.locator(contactAction)).toHaveCount(0)
  await expect(page.locator(flagAction)).toHaveCount(0)
  await expect(page.locator("#odialog-container")).toHaveCount(0)
})

test("a missing profile target exposes no action row", async ({ page }) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/user:info/missing-profile-action-probe")

  await expect(page.locator(".error-block")).toHaveText("User does not exist.")
  await expect(page.locator(privateMessageAction)).toHaveCount(0)
  await expect(page.locator(contactAction)).toHaveCount(0)
  await expect(page.locator(flagAction)).toHaveCount(0)
  await expect(page.locator("h1.profile-title")).toHaveCount(0)
})
