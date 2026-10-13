import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const PAGE_PATH = "/scp-173"
// Retained Wikidot anonymous edit-denial contract (issue #2336).
const DENIAL_TITLE = "Permission error"
const DENIAL_HEADING = "Please note:"
const DENIAL_POLICY =
  "Sorry, you can not edit this page. Only members of this site, owner (creator) of this page, site administrators and perhaps selected moderators are allowed to do it."

const expectEditDeniedDialog = async (page: import("@playwright/test").Page) => {
  await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
  await expect(page.locator("#owindow-1 .title")).toHaveText(DENIAL_TITLE)
  await expect(page.locator("#modal-title")).toHaveText(DENIAL_HEADING)
  const policy = page.locator(".modal-message-extra")
  await expect(policy).toContainText(DENIAL_POLICY)
  await expect(policy.locator('a[href="#action:login"]')).toHaveText(
    "Sign in as Wikidot user"
  )
  await expect(page.locator("#odialog-container .button-close-message")).toBeVisible()
}

const waitForWikidotHydration = (page: import("@playwright/test").Page) =>
  page.waitForFunction(() => {
    const pageContent = document.querySelector("#page-content")
    return (
      pageContent !== null &&
      pageContent.firstChild?.nodeType !== Node.COMMENT_NODE &&
      pageContent.lastChild?.nodeType !== Node.COMMENT_NODE
    )
  })

const expectNoEditSurface = async (page: import("@playwright/test").Page) => {
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  await expect(page.locator("#page-tags")).toHaveCount(0)
  await expect(page).toHaveURL(/\/scp-173(#.*)?$/u)
}

test("anonymous #_editpage dispatch answers with the permission dialog once", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}${PAGE_PATH}#_editpage`)
  await waitForWikidotHydration(page)

  const dialog = page.locator("#odialog-container .owindow.error")
  await expectEditDeniedDialog(page)
  // The dialog is a single instance, the hash is preserved, and no editor mounts.
  await expect(page.locator("#odialog-container")).toHaveCount(1)
  await expect(page).toHaveURL(/#_editpage$/u)
  await expect(page.locator("#page-tags")).toHaveCount(0)

  await expect(dialog).toBeVisible()
  await expect(page.locator("#odialog-container")).toHaveCount(1)

  await page.locator("#odialog-container .button-close-message").click()
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  await expect(page).toHaveURL(/#_editpage$/u)
  await expectNoEditSurface(page)
})

test("anonymous #_edittags dispatch answers with the same permission dialog", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}${PAGE_PATH}#_edittags`)
  await waitForWikidotHydration(page)

  await expectEditDeniedDialog(page)
  // The tag editor form is not mounted for an unauthorized actor.
  await expect(page.locator("#page-tags")).toHaveCount(0)
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect(page).toHaveURL(/#_edittags$/u)
})

test("authorized #_editpage and #_edittags dispatch reach their surfaces", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto(`${APP_URL}${PAGE_PATH}#_editpage`)
  await expect(page).toHaveURL(/\/scp-173\/edit$/u)
  await expect(page.locator("#editor")).toBeVisible()
  await expect(page.locator("#odialog-container")).toHaveCount(0)

  await page.goto(`${APP_URL}${PAGE_PATH}#_edittags`)
  await waitForWikidotHydration(page)
  await expect(page.locator("#action-area #page-tags")).toBeVisible()
  await expect(page.locator("#page-tags-input")).toBeVisible()
  await expect(page.locator("#odialog-container")).toHaveCount(0)
})

test("history, files, unknown hashes and ordinary anchors keep their behavior", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}${PAGE_PATH}#_history`)
  await expect(page.locator("#action-area .page-revision-header")).toBeVisible()

  // A fresh document load re-runs the initial dispatch; a hash-only change
  // would be a same-document navigation that must stay inert.
  await page.goto(`${APP_URL}${PAGE_PATH}#_files`)
  await page.reload()
  await expect(page.locator("#action-area .page-file-header")).toBeVisible()

  await page.goto(`${APP_URL}${PAGE_PATH}#_nonexistent`)
  await page.reload()
  await waitForWikidotHydration(page)
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expectNoEditSurface(page)

  // A same-document hash change never re-dispatches a Hash Magic command.
  await page.evaluate(() => {
    location.hash = "_editpage"
  })
  await page.waitForTimeout(250)
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  await expect(page).toHaveURL(/#_editpage$/u)
  await expectNoEditSurface(page)
})
