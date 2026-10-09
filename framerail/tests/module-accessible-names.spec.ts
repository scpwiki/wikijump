import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const HTTPS_APP_URL = `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`

test("NewPage page-name inputs keep accessible names in SSR and hydrated DOM", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto("/newpage-helper")

  expect(response?.status()).toBe(200)
  const ssr = await response?.text()
  expect(ssr?.match(/aria-label="Name of the new page"/gu)).toHaveLength(3)

  const inputs = page.locator('input[name="pageName"]')
  await expect(inputs).toHaveCount(3)
  await expect(page.getByRole("textbox", { name: "Name of the new page" })).toHaveCount(3)
  expect(
    await inputs.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute("aria-label"))
    )
  ).toEqual(Array(3).fill("Name of the new page"))
  await expect(page.locator("#default-newpage input[type=submit]")).toHaveValue(
    "Default create"
  )
})

test("SearchAll query input has a distinct accessible name and keeps radio labels", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto("/search:all")

  expect(response?.status()).toBe(200)
  expect((await response?.text()) ?? "").toContain('aria-label="Search all Wikis"')

  const query = page.locator("#search-form-all-input")
  await expect(page.getByRole("textbox", { name: "Search all Wikis" })).toHaveCount(1)
  await expect(query).toHaveAttribute("aria-label", "Search all Wikis")
  await expect(page.getByRole("radio", { name: "pages and forums" })).toBeChecked()
  await expect(page.getByRole("radio", { name: "pages only" })).toBeAttached()
  await expect(page.getByRole("radio", { name: "forums only" })).toBeAttached()
})

for (const localeCase of [
  {
    locale: "en-US",
    layout: "Page layout",
    name: "Meta tag name",
    content: "Meta tag content"
  },
  {
    locale: "ja-JP",
    layout: "ページレイアウト",
    name: "メタタグ名",
    content: "メタタグの内容"
  }
]) {
  test.describe(`page option controls (${localeCase.locale})`, () => {
    test.use({ locale: localeCase.locale })

    test("expose localized accessible names", async ({ page }, testInfo) => {
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

      await waitForSvelteDelegatedHandler(page, "#more-options-button")
      await page.locator("#more-options-button").click()
      await waitForSvelteDelegatedHandler(page, "#layout-button")
      await page.locator("#layout-button").click()

      const layout = page.getByRole("combobox", { name: localeCase.layout })
      await expect(layout).toHaveCount(1)
      await expect(layout).toHaveAttribute("name", "layout")
      await layout.focus()
      await page.keyboard.press("ArrowDown")
      await expect(layout).toBeFocused()
      await expect(layout).toHaveAccessibleName(localeCase.layout)

      await page.locator("#action-area .action-area-close").click()
      await waitForSvelteDelegatedHandler(page, "#edit-meta-button")
      await page.locator("#edit-meta-button").click()
      await expect(page.locator("#edit-meta-addbutton button")).toBeVisible()
      await page.locator("#edit-meta-addbutton button").click()

      const metaName = page.getByRole("textbox", { name: localeCase.name })
      const metaContent = page.getByRole("textbox", {
        name: localeCase.content
      })
      await expect(metaName).toHaveCount(1)
      await expect(metaContent).toHaveCount(1)
      await expect(metaName).toHaveAttribute("name", "metaName")
      await expect(metaContent).toHaveAttribute("name", "metaContent")
      await expect(metaName).toHaveAccessibleName(localeCase.name)
      await expect(metaContent).toHaveAccessibleName(localeCase.content)
      await metaName.focus()
      await page.keyboard.type("accessible-name-probe")
      await expect(metaName).toHaveValue("accessible-name-probe")

      await page.locator("#action-area .action-area-close").click()
      await waitForSvelteDelegatedHandler(page, "#edit-meta-button")
      await page.locator("#edit-meta-button").click()
      await page.locator("#edit-meta-addbutton button").click()
      await expect(page.getByRole("textbox", { name: localeCase.name })).toHaveCount(1)
      await expect(page.getByRole("textbox", { name: localeCase.content })).toHaveCount(1)

      const writes = await page.request.get(
        `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}/last-page-write-requests`
      )
      expect(writes.ok()).toBe(true)
      expect(await writes.json()).toEqual({
        login: [],
        pageCreate: [],
        pageEdit: [],
        pageRollback: [],
        pageMove: [],
        parentGetAll: [],
        parentUpdate: [],
        sessionGet: [],
        userGet: [],
        voteSet: [],
        voteRemove: []
      })
    })
  })
}
