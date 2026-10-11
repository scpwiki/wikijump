import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

test("the registration form discloses the account-creation legal contract", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}/-/register`)
  await expect(page.locator("#register")).toBeVisible()

  // The disclosure is part of the signup form, adjacent to the Create Account
  // action, not a footer link the reader has to go looking for.
  const notice = page.locator("#register-legal-notice")
  await expect(notice).toBeVisible()
  await expect(notice).toHaveText(
    "By creating an account you accept our Terms of Service and Privacy Policy."
  )

  await expect(
    page.getByRole("link", { name: "Terms of Service", exact: true })
  ).toHaveAttribute("href", "https://wikijump.localhost/legal/terms-of-service")
  await expect(
    page.getByRole("link", { name: "Privacy Policy", exact: true })
  ).toHaveAttribute("href", "https://wikijump.localhost/legal/privacy-policy")

  // The destinations must be the authoritative platform policies, never the
  // wiki being viewed.
  for (const link of await notice.getByRole("link").all()) {
    const href = await link.getAttribute("href")
    expect(href).toMatch(/^https:\/\//u)
    expect(href).not.toBe("/")
    expect(href).not.toBe(APP_URL)
  }

  // Create Account names the disclosure, so the contract is discoverable
  // from the action boundary itself.
  await expect(page.locator("#register button[type=submit]")).toHaveAttribute(
    "aria-describedby",
    "register-legal-notice"
  )

  // The disclosure is an implicit-acceptance notice, not a consent control:
  // the registration form still submits no extra consent field, and the
  // anti-bot honeypot concept is not turned into a fake terms checkbox.
  await expect(page.locator("#register input[type=checkbox]")).toHaveCount(0)
  await expect(page.locator("#register .register-legal input")).toHaveCount(0)
  await expect(
    page.locator("#register button[type=submit]", { hasText: "create-account" })
  ).toBeVisible()
})
