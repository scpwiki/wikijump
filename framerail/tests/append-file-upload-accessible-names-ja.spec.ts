import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const appURL = (projectName: string) =>
  projectName === "webkit-https-edit-meta"
    ? `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`
    : `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

const waitForPageActionsHydration = (page: import("@playwright/test").Page) =>
  page.waitForFunction(() => {
    const pageContent = document.querySelector("#page-content")
    return (
      pageContent !== null &&
      pageContent.firstChild?.nodeType !== Node.COMMENT_NODE &&
      pageContent.lastChild?.nodeType !== Node.COMMENT_NODE
    )
  })

test.use({ locale: "ja-JP" })

test("Japanese Append and Upload labels remain associated at 320px and after reopen", async ({
  page
}, testInfo) => {
  await page.setViewportSize({ width: 320, height: 812 })
  await page.setExtraHTTPHeaders({
    ...SITE_HEADERS,
    "Accept-Language": "ja"
  })
  const response = await page.goto(`${appURL(testInfo.project.name)}/page-workflow-probe`)
  expect(response?.status()).toBe(200)
  await waitForPageActionsHydration(page)

  const options = page.locator("#more-options-button")
  await expect(options).toBeVisible()
  await options.click()
  await page.locator("#edit-append-button").click()
  const appendName = "追記するWikiテキスト"
  const append = page.getByRole("textbox", { name: appendName })
  await expect(append).toHaveCount(1)
  await page.locator('#page-append input[type="button"]').click()
  await expect(page.locator("#edit-append-button")).toBeVisible()
  await page.locator("#edit-append-button").click()
  await expect(page.getByRole("textbox", { name: appendName })).toHaveCount(1)
  await page.locator('#page-append input[type="button"]').click()

  await page.locator("#files-button").click()
  await page.getByRole("button", { name: /upload/u }).click()
  const file = page.getByRole("button", { name: "ファイルを選択:" })
  const filename = page.getByRole("textbox", { name: "ファイル名:" })
  await expect(file).toHaveCount(1)
  await expect(filename).toHaveCount(1)
  await expect(filename).toBeVisible()
  for (const geometry of await Promise.all([
    page.locator("#file-upload-file-input").boundingBox(),
    page.locator("#file-upload-name-input").boundingBox()
  ])) {
    expect(geometry).not.toBeNull()
    expect(geometry!.x + geometry!.width).toBeLessThanOrEqual(320)
  }
})
