import {
  expect,
  installNativeEventListenerProbe,
  test,
  waitForNativeEventListener
} from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

test.use({ locale: "ja-JP" })

test("registration discloses the account-creation legal contract in Japanese", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}/-/register`)
  await expect(page.locator("#register")).toBeVisible()

  await expect(page.locator("#register-legal-notice")).toHaveText(
    "アカウントを作成すると、利用規約およびプライバシーポリシーに同意したものとみなされます。"
  )
  await expect(page.getByRole("link", { name: "利用規約", exact: true })).toHaveAttribute(
    "href",
    "https://wikijump.localhost/legal/terms-of-service"
  )
  await expect(
    page.getByRole("link", { name: "プライバシーポリシー", exact: true })
  ).toHaveAttribute("href", "https://wikijump.localhost/legal/privacy-policy")

  // The action boundary still names the disclosure under the Japanese locale.
  await expect(page.locator("#register button[type=submit]")).toHaveAttribute(
    "aria-describedby",
    "register-legal-notice"
  )
})

test("registration password errors render Japanese inline text", async ({ page }) => {
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}/-/register`)

  await page.locator("#username").fill("fixture-japanese-short-password")
  await page.locator("#register .email").fill("japanese-short@example.invalid")
  await page.locator("#register .auth-password").fill("too-short")
  await page.locator(".confirm-password").fill("too-short")
  await page.locator("#locale").selectOption(["ja"])
  await waitForNativeEventListener(page, "#register", "submit")
  await page.locator("#register button[type=submit]").click()

  await expect(page.locator("#register-password-error")).toHaveText(
    "パスワードは15文字以上で入力してください。"
  )
  await expect(page.locator("#modal-message")).toHaveCount(0)

  await page.locator("#register .auth-password").fill("Good-fixture-password-2026")
  await page.locator(".confirm-password").fill("Good-fixture-password-2027")
  await waitForNativeEventListener(page, "#register", "submit")
  await page.locator("#register button[type=submit]").click()

  await expect(page.locator("#register-confirm-password-error")).toHaveText(
    "パスワードが一致しません。"
  )
  await expect(page.locator("#modal-message")).toHaveCount(0)
})
