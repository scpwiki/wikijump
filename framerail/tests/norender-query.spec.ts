import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const PAGE = "scp-173"
const RENDERED_TEXT = "SCP-173"
// The hermetic fixture echoes unknown translation keys, so the observed
// no-render placeholder is the message key (the English copy is
// "Content not shown.").
const NO_RENDER_MARKER = "wiki-page-no-render"

async function openPage(page: import("@playwright/test").Page, path: string) {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(path)
  await expect(page.locator("#page-content")).toBeVisible()
}

async function expectNoRenderContent(page: import("@playwright/test").Page) {
  const content = page.locator("#page-content")
  await expect(content).toContainText(NO_RENDER_MARKER)
  await expect(content.locator("textarea.page-source")).toHaveValue("**Item #:** SCP-173")
  // The compiled article body is never shown in no-render mode.
  await expect(content.locator("p")).toHaveCount(0)
  await expect(content.locator("strong")).toHaveCount(0)
}

async function expectRenderedContent(page: import("@playwright/test").Page) {
  const content = page.locator("#page-content")
  await expect(content).toContainText(RENDERED_TEXT)
  await expect(content.locator("textarea.page-source")).toHaveCount(0)
  await expect(content.locator("strong")).toHaveText("Item #:")
}

test("path no-render suffix suppresses the article body", async ({ page }) => {
  await openPage(page, `/${PAGE}/norender/true`)
  await expectNoRenderContent(page)
})

test("query no-render selector matches the path form", async ({ page }) => {
  await openPage(page, `/${PAGE}/norender/true`)
  const pathText = await page.locator("#page-content").innerText()

  await openPage(page, `/${PAGE}?norender=true`)
  await expectNoRenderContent(page)
  await expect(page.locator("#page-content")).toHaveText(pathText)
})

test("query selector accepts the documented truthy aliases", async ({ page }) => {
  for (const value of ["t", "1"]) {
    await openPage(page, `/${PAGE}?norender=${value}`)
    await expectNoRenderContent(page)
  }
})

test("false and zero query values keep the rendered article", async ({ page }) => {
  for (const value of ["false", "0"]) {
    await openPage(page, `/${PAGE}?norender=${value}`)
    await expectRenderedContent(page)
  }
})

test("removing the selector restores the rendered article", async ({ page }) => {
  await openPage(page, `/${PAGE}?norender=true`)
  await expectNoRenderContent(page)

  await openPage(page, `/${PAGE}`)
  await expectRenderedContent(page)
})
