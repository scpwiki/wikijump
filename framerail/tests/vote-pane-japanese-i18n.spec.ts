import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const HTTPS_APP_URL = `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`

const locales = {
  "en-US": {
    heading: "Page rating",
    description: "Simply rate contents of this page.",
    like: "I like it",
    dislike: "I don't like it",
    cancel: "Cancel my vote",
    whoRated: "Look who rated this page",
    user: "User"
  },
  "ja-JP": {
    heading: "ページの評価",
    description: "このページの内容を評価してみましょう。",
    like: "好き",
    dislike: "好きじゃない",
    cancel: "投票を取り消す",
    whoRated: "誰がこのページに評価したかを閲覧",
    user: "ユーザー"
  }
} as const

for (const [locale, expected] of Object.entries(locales) as Array<
  [keyof typeof locales, (typeof locales)[keyof typeof locales]]
>) {
  test.describe(`Vote pane labels (${locale})`, () => {
    test.use({ locale })

    test("keep Japanese/English names and tooltips at narrow width", async ({
      page
    }, testInfo) => {
      test.skip(
        testInfo.project.name === "webkit",
        "WebKit runs this acceptance case against the HTTPS fixture"
      )

      await page.setViewportSize({ width: 320, height: 640 })
      await page.setExtraHTTPHeaders(SITE_HEADERS)
      const baseUrl =
        testInfo.project.name === "webkit-https-edit-meta" ? HTTPS_APP_URL : APP_URL
      const response = await page.goto(`${baseUrl}/scp-173`)
      expect(response?.status()).toBe(200)

      await waitForSvelteDelegatedHandler(page, "#pagerate-button")
      await page.locator("#pagerate-button").click()

      await expect(page.locator(".page-vote-header")).toHaveText(expected.heading)
      await expect(page.locator(".page-vote-header + p")).toHaveText(expected.description)

      const like = page.locator(".rateup a")
      const dislike = page.locator(".ratedown a")
      const cancel = page.locator(".cancel a")
      await expect(like).toHaveAccessibleName(expected.like)
      await expect(like).toHaveAttribute("title", expected.like)
      await expect(dislike).toHaveAccessibleName(expected.dislike)
      await expect(dislike).toHaveAttribute("title", expected.dislike)
      await expect(cancel).toHaveAccessibleName(expected.cancel)
      await expect(cancel).toHaveAttribute("title", expected.cancel)
      await expect(
        page.getByRole("link", { name: expected.whoRated, exact: true })
      ).toBeVisible()

      await like.focus()
      await expect(like).toBeFocused()
      await page.keyboard.press("Tab")
      await expect(dislike).toBeFocused()
      await page.keyboard.press("Tab")
      await expect(cancel).toBeFocused()

      const writes = await page.request.get(
        `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}/last-page-write-requests`
      )
      expect(writes.ok()).toBe(true)
      const recorded = (await writes.json()) as Record<string, unknown[]>
      expect(Object.values(recorded).every((requests) => requests.length === 0)).toBe(
        true
      )
    })

    test("localize the native vote-list entry", async ({ page }, testInfo) => {
      test.skip(
        testInfo.project.name === "webkit",
        "WebKit runs this acceptance case against the HTTPS fixture"
      )

      await page.setExtraHTTPHeaders(SITE_HEADERS)
      const baseUrl =
        testInfo.project.name === "webkit-https-edit-meta" ? HTTPS_APP_URL : APP_URL
      const response = await page.goto(`${baseUrl}/vote-pane-wikijump-probe`)
      expect(response?.status()).toBe(200)

      await waitForSvelteDelegatedHandler(page, ".button-vote")
      await page.locator(".button-vote").click()
      await expect(page.locator(".vote-panel")).toBeVisible()
      await page.locator(".view-vote-list").click()
      await expect(page.locator(".vote-item")).toHaveText(`${expected.user} 456: 1`)
      await expect(page.locator(".vote-item")).not.toContainText("UNTRANSLATED")
    })
  })
}
