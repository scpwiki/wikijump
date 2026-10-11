import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`
const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test.afterEach(async ({ request }) => {
  await request.post(`${FIXTURE_URL}/reset-authenticated-profile`)
})

test("Cancel discards a rejected profile draft and a corrected field is submitted", async ({
  page,
  request
}) => {
  await request.post(`${FIXTURE_URL}/reset-authenticated-profile`)
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  await page.goto("/-/login")
  await page.locator("#login-name-or-email").fill("fixture-member")
  await page.locator("#login-password").fill("fixture-member-password")
  const loginResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/-/login" &&
      response.request().method() === "POST"
  )
  await page.locator("#login button[type='submit']").click()
  await loginResponse
  await expect
    .poll(async () =>
      (await page.context().cookies()).some((cookie) => cookie.name === "wikijump_token")
    )
    .toBe(true)

  await page.goto("/-/user")
  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()

  await expect(
    page.getByRole("heading", { name: "User profile information" })
  ).toBeVisible()
  await expect(page.locator("textarea.debug")).toHaveCount(0)

  const email = page.locator("#email")
  await expect(email).toHaveValue("fixture-member@example.test")
  await email.fill("not-a-valid-email")
  const rejectedSave = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/-/user" &&
      response.request().method() === "POST"
  )
  await page.locator("form#editor .button-save").click()
  await rejectedSave
  await expect(page.locator("#modal-message")).toHaveText("The user's email is invalid")
  await expect(email).toHaveValue("not-a-valid-email")

  await page.keyboard.press("Escape")
  await expect(page.locator("#modal-message")).toBeHidden()
  await expect(page.locator("form#editor")).toBeVisible()
  await page.locator("form#editor .button-cancel").click()
  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(email).toHaveValue("fixture-member@example.test")

  const correctedEmail = "updated-fixture-member@example.test"
  await email.fill(correctedEmail)
  const acceptedSave = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/-/user" &&
      response.request().method() === "POST"
  )
  await page.locator("form#editor .button-save").click()
  await acceptedSave
  await expect(page.locator("form#editor")).toBeHidden()

  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(email).toHaveValue(correctedEmail)
  await page.reload()
  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(page.locator("#email")).toHaveValue(correctedEmail)

  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(writes.userEdit).toHaveLength(2)
  for (const write of writes.userEdit) {
    expect(write.paramKeys).toContain("email")
    expect(write.paramKeys).not.toContain("name")
  }
})
