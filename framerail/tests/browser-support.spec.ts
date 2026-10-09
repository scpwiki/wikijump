import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

test("supported browser families share the same core page and authority boundary", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`${APP_URL}/theme:yossistyle`)

  expect(response?.status()).toBe(200)
  await expect(page.locator("#page-content p")).toHaveText("XML-RPC theme body marker.")
  await expect(page.locator("#page-title")).toBeVisible()
  await expect(page.locator("#page-options-container")).toBeVisible()

  expect(response?.headers()["content-security-policy"]).toContain("default-src")
  expect(response?.headers()["x-content-type-options"]).toBe("nosniff")
})

test("About diagnostics and repository links stay usable at narrow widths", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  for (const width of [320, 375, 390, 768]) {
    await page.setViewportSize({ width, height: 812 })
    const response = await page.goto(`${APP_URL}/-/about`)
    expect(response?.status(), `About at ${width}px`).toBe(200)
    await expect(page.locator("table.platform-info")).toBeVisible()

    const dimensions = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      document: document.documentElement.scrollWidth
    }))
    expect(
      dimensions.document,
      `About at ${width}px: ${JSON.stringify(dimensions)}`
    ).toBeLessThanOrEqual(dimensions.viewport + 1)

    const repositoryLinks = page.locator("table.platform-info a[rel='external']")
    await expect(repositoryLinks).toHaveCount(2)
    for (const link of await repositoryLinks.all()) {
      await expect(link).toBeVisible()
      const [href, text] = await Promise.all([
        link.getAttribute("href"),
        link.textContent()
      ])
      expect(text?.trim()).toBe(href)
    }

    if (width === 320) {
      await page.evaluate(() => {
        document.documentElement.style.fontSize = "200%"
      })
      const zoomed = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth,
        document: document.documentElement.scrollWidth
      }))
      expect(
        zoomed.document,
        `About at 320px and 200% text: ${JSON.stringify(zoomed)}`
      ).toBeLessThanOrEqual(zoomed.viewport + 1)
    }
  }
})

test("anonymous public profile HTML omits private and internal profile data", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  const response = await page.goto(`${APP_URL}/-/user/guest`)
  expect(response?.status()).toBe(200)

  const serverHtml = await response!.text()
  const profile = page.locator(".user-info")
  await expect(profile.locator(".name")).toHaveText("Guest")
  await expect(page.locator("textarea.debug")).toHaveCount(0)
  await expect(page.getByText(/UNTRANSLATED:/)).toHaveCount(0)

  const forbiddenValues = [
    "private-email-marker-2139@example.test",
    "private-birthday-marker-2139",
    "private-biography-marker-2139",
    "site_settings",
    "user_session",
    "userEditForm"
  ]
  for (const value of forbiddenValues) {
    expect(serverHtml).not.toContain(value)
    await expect(page.locator("html")).not.toContainText(value)
  }

  const cookies = await page.context().cookies()
  expect(cookies.some(({ name }) => name === "wikijump_token")).toBe(false)
})
