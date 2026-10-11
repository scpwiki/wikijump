import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`
const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("canceling a selected profile avatar preserves the saved avatar without a write", async ({
  page,
  request
}) => {
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

  await page.route("https://scp-wiki.wjfiles.localhost/-/avatar/6000008", (route) =>
    route.fulfill({
      contentType: "image/png",
      body: Buffer.from(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=",
        "base64"
      )
    })
  )

  await page.goto("/-/user")
  const savedAvatar = page.locator(".user-info img.user-attribute-value")
  const savedAvatarUrl = "https://scp-wiki.wjfiles.localhost/-/avatar/6000008"
  await expect(savedAvatar).toHaveAttribute("src", savedAvatarUrl)
  await expect
    .poll(() => savedAvatar.evaluate((image: HTMLImageElement) => image.naturalWidth))
    .toBe(1)

  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  const avatarInput = page.locator("#avatar")
  await expect(avatarInput).toBeVisible()
  await avatarInput.setInputFiles({
    name: "replacement.png",
    mimeType: "image/png",
    buffer: Buffer.from("synthetic replacement image")
  })
  await expect(
    avatarInput.evaluate((input: HTMLInputElement) => input.files?.length)
  ).resolves.toBe(1)

  await waitForSvelteDelegatedHandler(page, "form#editor .button-cancel")
  await page.locator("form#editor .button-cancel").click()
  await expect(savedAvatar).toHaveAttribute("src", savedAvatarUrl)
  await expect
    .poll(() => savedAvatar.evaluate((image: HTMLImageElement) => image.naturalWidth))
    .toBe(1)

  await waitForSvelteDelegatedHandler(page, ".button-edit")
  await page.locator(".button-edit").click()
  await expect(page.locator("#avatar")).toBeVisible()
  await expect(
    page.locator("#avatar").evaluate((input: HTMLInputElement) => input.files?.length)
  ).resolves.toBe(0)
  await page.locator("form#editor .button-cancel").click()
  await expect(savedAvatar).toHaveAttribute("src", savedAvatarUrl)

  const writes = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  expect(writes.userEdit).toEqual([])
})
