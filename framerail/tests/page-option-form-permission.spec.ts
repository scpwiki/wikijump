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
const PERMISSION_MESSAGE = "You don't have permission to edit this page."

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
 * Open the page `+ Options` menu on the probe page with a stubbed
 * `?/editPermission` answer, and record every permission check.
 */
const openOptionsMenu = async (
  page: Page,
  canEdit: boolean,
  permissionRequests: Route[],
  headers: Record<string, string>
) => {
  await page.setExtraHTTPHeaders(headers)
  await page.route(
    (url) => url.search === "?/editPermission",
    async (route) => {
      permissionRequests.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: { can_edit: canEdit } })
      })
    }
  )

  // The Parent pane reads its current parents on mount. The shared Deepwell
  // fixture only answers slug lookups, so this spec answers the numeric-id
  // lookup itself; the parent list is not part of the permission gate.
  await page.route(
    (url) => url.search === "?/parentGet",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [] })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await expect(page.locator("#edit-append-button")).toBeVisible()
}

const expectNoWritableSurface = async (page: Page, formSelector: string) => {
  await expect(page.locator(formSelector)).toHaveCount(0)
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
}

test.describe("anonymous existing-page Append and Parent options", () => {
  test("Append answers with the edit-permission dialog and opens no form", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(page, false, permissionRequests, SITE_HEADERS)

    await page.locator("#edit-append-button").click()

    await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
    await expect(page.locator("#modal-title")).toHaveText(PERMISSION_MESSAGE)
    await expectNoWritableSurface(page, "#page-append")
    expect(permissionRequests).toHaveLength(1)
  })

  test("Parent answers with the edit-permission dialog and opens no form", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(page, false, permissionRequests, SITE_HEADERS)

    await page.locator("#parent-page-button").click()

    await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
    await expect(page.locator("#modal-title")).toHaveText(PERMISSION_MESSAGE)
    await expectNoWritableSurface(page, "#page-parent")
    expect(permissionRequests).toHaveLength(1)
  })
})

test.describe("authorized existing-page Append and Parent options", () => {
  test("Append opens the append form without a permission dialog", async ({ page }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(page, true, permissionRequests, AUTHENTICATED_HEADERS)

    await page.locator("#edit-append-button").click()

    await expect(page.locator("#page-append")).toBeVisible()
    await expect(page.locator("#page-append-input")).toBeVisible()
    await expect(page.locator("#odialog-container")).toHaveCount(0)
    expect(permissionRequests).toHaveLength(1)
  })

  test("Parent opens the parent form without a permission dialog", async ({ page }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(page, true, permissionRequests, AUTHENTICATED_HEADERS)

    await page.locator("#parent-page-button").click()

    await expect(page.locator("#page-parent")).toBeVisible()
    await expect(page.locator("#odialog-container")).toHaveCount(0)
    expect(permissionRequests).toHaveLength(1)
  })
})
