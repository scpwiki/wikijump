import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const PAGE_PATH = "/authoring-history-probe"

// Retained Wikidot anonymous edit-denial contract (issue #2336): the EN
// Permission error window with a Please note: heading, one policy
// paragraph, and an inline #action:login sign-in anchor.
const DENIAL_TITLE = "Permission error"
const DENIAL_HEADING = "Please note:"
const DENIAL_POLICY =
  "Sorry, you can not edit this page. Only members of this site, owner (creator) of this page, site administrators and perhaps selected moderators are allowed to do it."

test("anonymous Edit option shows the Wikidot denial without an editor", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(PAGE_PATH, { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await expect(page.locator("#edit-button")).toBeVisible()
  await page.locator("#edit-button").click()

  const dialog = page.locator("#odialog-container .owindow.error")
  await expect(dialog).toBeVisible()
  await expect(page.locator("#owindow-1 .title")).toHaveText(DENIAL_TITLE)
  await expect(page.locator("#modal-title")).toHaveText(DENIAL_HEADING)
  const policy = page.locator(".modal-message-extra")
  await expect(policy).toContainText(DENIAL_POLICY)
  await expect(policy.locator('a[href="#action:login"]')).toHaveText(
    "Sign in as Wikidot user"
  )
  await expect(page.locator("#odialog-container .button-close-message")).toBeVisible()

  // No editor mounts, the action area stays hidden, and the page URL is kept.
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect(page.locator("#editor")).toHaveCount(0)
  await expect(page).toHaveURL(new RegExp(`${PAGE_PATH}$`, "u"))

  await page.locator("#odialog-container .button-close-message").click()
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  await expect(page).toHaveURL(new RegExp(`${PAGE_PATH}$`, "u"))
})
