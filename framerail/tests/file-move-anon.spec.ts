import type { Page, Route } from "@playwright/test"

import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token",
  origin: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
}

/**
 * Encode a value in SvelteKit's flat devalue shape, as `deserialize`
 * expects.
 */
const devalue = (value: unknown): string => {
  const flat: unknown[] = []
  const add = (item: unknown): number => {
    const index = flat.length
    flat.push(null)
    if (Array.isArray(item)) {
      flat[index] = item.map(add)
    } else if (item !== null && typeof item === "object") {
      const encoded: Record<string, number> = {}
      for (const [key, child] of Object.entries(item)) encoded[key] = add(child)
      flat[index] = encoded
    } else {
      flat[index] = item
    }
    return index
  }
  add(value)
  return JSON.stringify(flat)
}

const successBody = (value: unknown) =>
  JSON.stringify({ type: "success", status: 200, data: devalue(value) })

/**
 * Open the Files pane with an edit permission answer and record any move
 * submissions. Returns the permission response promise so callers can wait
 * for the server answer before asserting on the form.
 */
const openFilesPane = async (
  page: Page,
  headers: Record<string, string>,
  canEdit: boolean,
  moveRequests: Route[]
) => {
  await page.setExtraHTTPHeaders(headers)
  await page.route(
    (url) => url.search === "?/editPermission",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: { can_edit: canEdit } })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/fileMove",
    async (route) => {
      moveRequests.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          type: "failure",
          status: 400,
          data: devalue({ message: "file move stub" })
        })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect(page.locator(".file-panel .file-row").first()).toBeVisible()
}

const moveAction = (page: Page) =>
  page
    .locator(".file-panel .file-row")
    .first()
    .getByRole("button", { name: /^move$/i })
    .or(
      page
        .locator(".file-panel .file-row")
        .first()
        .getByRole("link", { name: /^move$/i })
    )

test("anonymous viewers are not offered the File Move form", async ({ page }) => {
  const moveRequests: Route[] = []
  await openFilesPane(page, SITE_HEADERS, false, moveRequests)

  // Wait for the server's answer on the permission check before asserting on
  // the form. The check is expected to answer; on a superseded or unanswered
  // check this resolves to null and the assertions below still run.
  const permissionAnswered = page
    .waitForResponse(
      (response) => new URL(response.url()).search === "?/editPermission",
      {
        timeout: 5000
      }
    )
    .catch(() => null)
  await moveAction(page).click()
  await permissionAnswered

  await expect(page.locator("#file-move")).toHaveCount(0)
  await expect(page.locator(".file-panel .file-move-destination-page")).toHaveCount(0)
  expect(moveRequests).toHaveLength(0)
})

test("authorized editors keep the File Move form", async ({ page }) => {
  const moveRequests: Route[] = []
  await openFilesPane(page, AUTHENTICATED_HEADERS, true, moveRequests)

  await moveAction(page).click()

  await expect(page.locator("#file-move")).toBeVisible()
  await expect(page.locator("#file-move .file-move-destination-page")).toBeVisible()
  expect(moveRequests).toHaveLength(0)
})
