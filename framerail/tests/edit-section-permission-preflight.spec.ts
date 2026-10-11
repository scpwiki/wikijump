import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const PAGE_PATH = "/edit-section-permission-probe"
const DENIAL_MESSAGE = "You don't have permission to edit this page."

const openSectionEditor = async (page: import("@playwright/test").Page) => {
  await page.goto(PAGE_PATH, { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await page.locator("#edit-sections-button").click()
  await expect(page.locator(".edit-section-button")).toBeVisible()
  await page.locator(".edit-section-button").click()
}

test("anonymous Edit Sections viewers see denial without an editor or mutation", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const actionRequests: string[] = []
  page.on("request", (request) => {
    const url = new URL(request.url())
    if (url.search.startsWith("?/")) actionRequests.push(url.search)
  })

  await openSectionEditor(page)

  await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
  await expect(page.locator("#modal-title")).toHaveText(DENIAL_MESSAGE)
  await expect(page.locator("#edit-section-content form#editor")).toHaveCount(0)
  await expect(page.getByRole("status")).toHaveCount(0)
  expect(actionRequests.filter((action) => action === "?/editPermission")).toHaveLength(1)
  expect(actionRequests.filter((action) => action === "?/edit")).toHaveLength(0)
})

test("authorized Edit Sections viewers get the editor after permission succeeds", async ({
  page
}) => {
  await page.setExtraHTTPHeaders({
    ...SITE_HEADERS,
    cookie: "wikijump_token=fixture-session-token"
  })
  const actionRequests: string[] = []
  page.on("request", (request) => {
    const url = new URL(request.url())
    if (url.search.startsWith("?/")) actionRequests.push(url.search)
  })

  await openSectionEditor(page)

  await expect(page.locator("#edit-section-content form#editor")).toBeVisible()
  await expect(page.locator("#edit-section-content .editor-wikitext")).toHaveValue(
    "++ Permission probe section\nSection body."
  )
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  expect(actionRequests.filter((action) => action === "?/editPermission")).toHaveLength(1)
  expect(actionRequests.filter((action) => action === "?/edit")).toHaveLength(0)
})
