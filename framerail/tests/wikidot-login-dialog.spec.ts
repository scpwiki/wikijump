import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const WIKI_PAGE = `${APP_URL}/wikidot-login-probe`

const openAnonymousWikiPage = async (page: import("@playwright/test").Page) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(WIKI_PAGE)
  expect(response?.status()).toBe(200)
  await expect(page.locator("#page-content")).toBeVisible()
  await waitForSvelteDelegatedHandler(page, ".login-status-sign-in", "click")
}

test("imported anonymous chrome exposes native hrefs on the auth controls", async ({
  page
}) => {
  await openAnonymousWikiPage(page)

  const createAccount = page.locator("a.login-status-create-account")
  const signIn = page.locator("a.login-status-sign-in")
  await expect(createAccount).toHaveAttribute("href", "/-/register")
  await expect(signIn).toHaveAttribute("href", "/-/login")
})

test("Sign in opens the surface in place and Cancel restores the page without history", async ({
  page
}) => {
  await openAnonymousWikiPage(page)
  const historyLength = await page.evaluate(() => history.length)

  await page.locator("a.login-status-sign-in").click()
  const dialog = page.locator("dialog.wikidot-login-dialog")
  await expect(dialog).toBeVisible()
  await expect(page).toHaveURL(WIKI_PAGE)
  await expect(page.locator("#page-content")).toBeVisible()

  await page.getByRole("button", { name: "Cancel" }).click()
  await expect(dialog).toBeHidden()
  await expect(page).toHaveURL(WIKI_PAGE)
  expect(await page.evaluate(() => history.length)).toBe(historyLength)
})

test("Escape closes the in-place surface and clears any typed password", async ({
  page
}) => {
  await openAnonymousWikiPage(page)
  await page.locator("a.login-status-sign-in").click()
  await page.locator("#wikidot-login-password").fill("dummy-password")
  await page.keyboard.press("Escape")

  await expect(page.locator("dialog.wikidot-login-dialog")).toBeHidden()
  await expect(page).toHaveURL(WIKI_PAGE)

  await page.locator("a.login-status-sign-in").click()
  await expect(page.locator("#wikidot-login-password")).toHaveValue("")
})

test("keyboard activation of Sign in opens the surface like a link", async ({ page }) => {
  await openAnonymousWikiPage(page)
  const signIn = page.locator("a.login-status-sign-in")
  await signIn.focus()
  await signIn.press("Enter")

  await expect(page.locator("dialog.wikidot-login-dialog")).toBeVisible()
  await expect(page).toHaveURL(WIKI_PAGE)
})

test("successful sign-in closes the surface in place and sets the session without a navigation", async ({
  page
}) => {
  await openAnonymousWikiPage(page)
  await page.locator("a.login-status-sign-in").click()
  await page.locator("#wikidot-login-name-or-email").fill("fixture-member")
  await page.locator("#wikidot-login-password").fill("fixture-member-password")
  await page.getByRole("button", { name: "Login" }).click()

  await expect(page.locator("dialog.wikidot-login-dialog")).toBeHidden()
  await expect(page).toHaveURL(WIKI_PAGE)
  expect(page.url()).not.toContain("fixture-member-password")
  // The fixture article view hardcodes user_session to null, so the
  // reconciled authenticated chrome is not provable here; the session cookie
  // set by the native action is the hermetic signal.
  const cookies = await page.context().cookies()
  expect(cookies.find((cookie) => cookie.name === "wikijump_token")?.value).toBe(
    "fixture-authenticated-session-token"
  )
})

test("failed sign-in keeps the page in place and clears the password field", async ({
  page
}) => {
  await openAnonymousWikiPage(page)
  await page.locator("a.login-status-sign-in").click()
  await page.locator("#wikidot-login-name-or-email").fill("fixture-member")
  await page.locator("#wikidot-login-password").fill("wrong-password")
  await page.getByRole("button", { name: "Login" }).click()

  await expect(page.locator("dialog.wikidot-login-dialog")).toBeVisible()
  await expect(page.locator("#wikidot-login-password")).toHaveValue("")
  await expect(page).toHaveURL(WIKI_PAGE)
})

test("MFA-required sign-in points to the native sign-in page without carrying a token", async ({
  page
}) => {
  await openAnonymousWikiPage(page)
  await page.locator("a.login-status-sign-in").click()
  await page.locator("#wikidot-login-name-or-email").fill("fixture-mfa-member")
  await page.locator("#wikidot-login-password").fill("fixture-mfa-password")
  await page.getByRole("button", { name: "Login" }).click()

  await expect(page.getByRole("status")).toContainText("Two-step verification")
  const nativeLink = page.getByRole("link", { name: "Open the sign-in page" })
  await expect(nativeLink).toHaveAttribute("href", "/-/login")
  await expect(page).toHaveURL(WIKI_PAGE)
})

test.describe("without JavaScript", () => {
  test.use({ javaScriptEnabled: false })

  test("Sign in is a plain native link to /-/login", async ({ page }) => {
    await page.setExtraHTTPHeaders(SITE_HEADERS)
    await page.goto(WIKI_PAGE)
    await page.locator("a.login-status-sign-in").click()
    await expect(page).toHaveURL(`${APP_URL}/-/login`)
  })
})
