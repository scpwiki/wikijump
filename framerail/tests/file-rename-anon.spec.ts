import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  // Only the owner fixture token resolves to a non-null user_session in the
  // article view fixture; the rename gate keys on that session presence.
  cookie: "wikijump_token=fixture-owner-session-token",
  origin: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
}

const openFilesPane = async (
  page: import("@playwright/test").Page,
  headers: Record<string, string>
) => {
  await page.setExtraHTTPHeaders(headers)
  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect(page.locator(".file-panel .file-row").first()).toBeVisible()
}

const renameAction = (page: import("@playwright/test").Page) =>
  page
    .locator(".file-panel .file-row")
    .first()
    .locator(".file-attribute.action a", { hasText: "Edit" })

test("anonymous viewers see the source rename denial without an editor or POST", async ({
  page
}) => {
  const editPosts: string[] = []
  page.on("request", (request) => {
    const url = new URL(request.url())
    if (request.method() === "POST" && url.search === "?/fileEdit") {
      editPosts.push(request.url())
    }
  })

  await openFilesPane(page, SITE_HEADERS)
  await renameAction(page).click()

  // Source scp-9506 permission text for Files Options Rename (issue #2324).
  const dialog = page.locator("#odialog-container .owindow.error")
  await expect(dialog.locator(".title")).toHaveText("PERMISSION ERROR")
  await expect(dialog.locator(".content")).toContainText(
    "Please note: Sorry, you can not rename file attachment in this page."
  )
  await expect(dialog.locator(".content")).toContainText(
    "Only owner (creator) of this page, site administrators and perhaps selected moderators are allowed to do it."
  )
  const signIn = dialog.getByRole("link", { name: "Sign in as Wikidot user" })
  await expect(signIn).toHaveAttribute("href", "#action:login")
  await expect(signIn).toBeFocused()

  await expect(page.locator("form#file-edit")).toHaveCount(0)
  await expect(page.locator(".file-panel .file-row").first()).toBeVisible()

  await page.keyboard.press("Escape")
  await expect(dialog).toHaveCount(0)
  await expect(renameAction(page)).toBeFocused()
  expect(editPosts).toEqual([])
})

test("signed-in viewers keep the file rename form", async ({ page }) => {
  await openFilesPane(page, AUTHENTICATED_HEADERS)

  await renameAction(page).click()

  await expect(page.locator("form#file-edit")).toBeVisible()
  await expect(page.locator("#odialog-container .owindow.error")).toHaveCount(0)
})
