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
 * Open the page `+ Options` menu on the probe page with stubbed permission
 * answers, and record every permission check. The move and delete routes
 * are stubbed alongside edit so Move/Delete clicks can be attributed to
 * their own server gate (issue #2177).
 */
const openOptionsMenu = async (
  page: Page,
  canEdit: boolean,
  permissionRequests: Route[],
  headers: Record<string, string>,
  pagePath = "/page-workflow-probe",
  moveDeleteGrants: { canMove?: boolean; canDelete?: boolean } = {},
  moveDeleteRequests: { move?: Route[]; delete?: Route[] } = {}
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
  await page.route(
    (url) => url.search === "?/movePermission",
    async (route) => {
      moveDeleteRequests.move?.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: { can_move: moveDeleteGrants.canMove ?? false } })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/deletePermission",
    async (route) => {
      moveDeleteRequests.delete?.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({
          res: { can_delete: moveDeleteGrants.canDelete ?? false }
        })
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

  await page.goto(pagePath, { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await expect(page.locator("#edit-append-button")).toBeVisible()
}

const expectNoWritableSurface = async (page: Page, formSelector: string) => {
  await expect(page.locator(formSelector)).toHaveCount(0)
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
}

test("hidden action area does not reserve space against a theme ID selector", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })

  const actionArea = page.locator("#action-area")
  await expect(actionArea).toHaveClass(/hidden/u)
  const footerTop = await page
    .locator("#footer")
    .evaluate((element) => element.getBoundingClientRect().top)

  // Imported Wikidot themes may target #action-area directly. That selector
  // outranks the shell's `.hidden` class unless the hidden state is inline.
  await page.addStyleTag({
    content: "#action-area { display: grid; margin-top: 16px; padding: 0; }"
  })

  await expect(actionArea).toHaveCSS("display", "none")
  await expect
    .poll(() =>
      page.locator("#footer").evaluate((element) => element.getBoundingClientRect().top)
    )
    .toBe(footerTop)
})

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

// Move and Delete must obey their own Wikidot permission boundary (issue
// #2177): an anonymous click answers with the move/delete denial dialog
// and mounts no writable form, and each click is attributed to exactly one
// permission request on its own route.
test.describe("anonymous existing-page Move and Delete options", () => {
  test("Move answers with the move-permission dialog and opens no form", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    const moveRequests: Route[] = []
    await openOptionsMenu(
      page,
      false,
      permissionRequests,
      SITE_HEADERS,
      "/page-workflow-probe",
      {},
      { move: moveRequests }
    )

    await page.locator("#rename-move-button").click()

    await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
    await expect(page.locator("#modal-title")).toHaveText(
      "You don't have permission to move this page."
    )
    await expectNoWritableSurface(page, "#page-move")
    expect(moveRequests).toHaveLength(1)
    expect(permissionRequests).toHaveLength(0)
  })

  test("Delete answers with the delete-permission dialog and opens no form", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    const deleteRequests: Route[] = []
    await openOptionsMenu(
      page,
      false,
      permissionRequests,
      SITE_HEADERS,
      "/page-workflow-probe",
      {},
      { delete: deleteRequests }
    )

    await page.locator("#delete-button").click()

    await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
    await expect(page.locator("#modal-title")).toHaveText(
      "You don't have permission to delete this page."
    )
    await expectNoWritableSurface(page, "#page-delete")
    expect(deleteRequests).toHaveLength(1)
    expect(permissionRequests).toHaveLength(0)
  })
})

test.describe("authorized existing-page Move and Delete options", () => {
  test("Move opens the move form without a permission dialog", async ({ page }) => {
    const permissionRequests: Route[] = []
    const moveRequests: Route[] = []
    await openOptionsMenu(
      page,
      false,
      permissionRequests,
      AUTHENTICATED_HEADERS,
      "/page-workflow-probe",
      { canMove: true },
      { move: moveRequests }
    )

    await page.locator("#rename-move-button").click()

    await expect(page.locator("#page-move")).toBeVisible()
    await expect(page.locator("#odialog-container")).toHaveCount(0)
    expect(moveRequests).toHaveLength(1)
    expect(permissionRequests).toHaveLength(0)
  })

  test("Delete opens the delete form without a permission dialog", async ({ page }) => {
    const permissionRequests: Route[] = []
    const deleteRequests: Route[] = []
    await openOptionsMenu(
      page,
      false,
      permissionRequests,
      AUTHENTICATED_HEADERS,
      "/page-workflow-probe",
      { canDelete: true },
      { delete: deleteRequests }
    )

    await page.locator("#delete-button").click()

    await expect(page.locator("#page-delete")).toBeVisible()
    await expect(page.locator("#odialog-container")).toHaveCount(0)
    expect(deleteRequests).toHaveLength(1)
    expect(permissionRequests).toHaveLength(0)
  })
})

// The visible Tags option must obey the same Wikidot edit-permission boundary
// as the #_edittags route. The real anonymous SCP-9506 Tags click shows a
// permission modal without mounting any writable tag form (issue #2329).
test.describe("Wikidot Tags option permission parity", () => {
  test("anonymous Tags opens the edit denial dialog, not an editable form", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(
      page,
      false,
      permissionRequests,
      SITE_HEADERS,
      "/wikidot-login-probe"
    )

    await page.locator("#tags-button").click()

    await expect(page.locator("#odialog-container .owindow.error")).toBeVisible()
    await expect(page.locator("#modal-title")).toHaveText(PERMISSION_MESSAGE)
    await expectNoWritableSurface(page, "#page-tags")
    expect(permissionRequests).toHaveLength(1)
  })

  test("authorized Tags opens its source-style edit form after the server gate", async ({
    page
  }) => {
    const permissionRequests: Route[] = []
    await openOptionsMenu(
      page,
      true,
      permissionRequests,
      AUTHENTICATED_HEADERS,
      "/wikidot-login-probe"
    )

    await page.locator("#tags-button").click()

    await expect(page.locator("#page-tags")).toBeVisible()
    await expect(page.locator("#page-tags-input")).toBeVisible()
    await expect(page.locator("#odialog-container")).toHaveCount(0)
    expect(permissionRequests).toHaveLength(1)
  })
})
