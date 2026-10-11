import type { Route } from "@playwright/test"

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

const file = {
  file_id: 26,
  file_created_at: "2026-10-01T00:00:00Z",
  file_updated_at: "2026-10-01T00:00:00Z",
  file_deleted_at: null,
  page_id: 3000340,
  revision_id: 28,
  revision_type: "regular",
  revision_created_at: "2026-10-01T00:00:00Z",
  revision_number: 1,
  revision_user_id: 123,
  name: "file-history-revert-permission.txt",
  data: null,
  mime: "text/plain",
  size: 12,
  s3_hash: "fixture-hash",
  revision_comments: "fixture",
  hidden_fields: []
}

const revision = {
  revision_id: 27,
  revision_type: "regular",
  created_at: "2026-10-01T00:00:00Z",
  revision_number: 1,
  file_id: 26,
  page_id: 3000340,
  site_id: 6000005,
  user_id: 123,
  name: file.name,
  s3_hash: null,
  mime: "text/plain",
  size: 12,
  changes: [],
  comments: "fixture revision",
  hidden: []
}

const openFileHistory = async (
  page: import("@playwright/test").Page,
  canEdit: boolean,
  rollbackRequests: Route[]
) => {
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [file] })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/fileHistory",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [revision] })
      })
    }
  )
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
    (url) => url.search === "?/fileRollback",
    async (route) => {
      rollbackRequests.push(route)
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          type: "failure",
          status: 400,
          data: devalue({ message: "rollback stub" })
        })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  const historyControl = page
    .locator(
      ".file-row .file-attribute.action a, .file-row .file-attribute.action button"
    )
    .first()
  await expect(historyControl).toBeVisible()
  await historyControl.click()
  await expect(page.locator(".revision-row")).toContainText(file.name)
}

test("anonymous file history has no Revert control and emits no rollback request", async ({
  page
}) => {
  const rollbackRequests: Route[] = []
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await openFileHistory(page, false, rollbackRequests)

  const revertControls = page.locator(
    ".revision-row .revision-attribute.action a, .revision-row .revision-attribute.action button"
  )
  await expect(revertControls).toHaveCount(0)
  await expect(page.locator(".revision-row form")).toHaveCount(0)
  await expect(page.getByRole("dialog")).toHaveCount(0)
  const focusableActions = await page
    .locator(".revision-row")
    .evaluate(
      (row) =>
        row.querySelectorAll("a, button, input, select, textarea, [tabindex]").length
    )
  expect(focusableActions).toBe(0)
  expect(rollbackRequests).toHaveLength(0)
})

test("authorized file history retains its existing rollback request", async ({
  page
}) => {
  const rollbackRequests: Route[] = []
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await openFileHistory(page, true, rollbackRequests)

  const revertControl = page.locator(
    ".revision-row .revision-attribute.action a, .revision-row .revision-attribute.action button"
  )
  await expect(revertControl).toHaveCount(1)
  await revertControl.click()

  await expect.poll(() => rollbackRequests.length).toBe(1)
  expect(JSON.parse(rollbackRequests[0].request().postData() ?? "{}")).toMatchObject({
    siteId: 6000005,
    pageId: 3000340,
    fileId: 26,
    revisionNumber: 1,
    lastRevisionId: 28
  })
})
