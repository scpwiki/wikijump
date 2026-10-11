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

const ACTIVE_FILE_NAME = "theme-lab-visual-fixture"
// The page-workflow-probe fixture uses the Wikidot layout.
const FILE_ACTION_ROW = ".buttons"

test("a late first Files result cannot replace History after the viewer switches", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldActiveLists: Route[] = []
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      if (
        JSON.parse(route.request().postData() ?? "{}").deleted === false &&
        heldActiveLists.length === 0
      ) {
        heldActiveLists.push(route)
        return
      }
      await route.continue()
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect.poll(() => heldActiveLists.length).toBe(1)

  // Source Wikidot cannot expose Restore while its first Files list is
  // pending. Switch to another public option instead; a late response must
  // never reopen the abandoned pane.
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect
    .poll(() => page.locator("#action-area").evaluate((area) => area.children.length))
    .toBe(0)
  await expect(page.locator(FILE_ACTION_ROW)).not.toBeVisible()
  await page.locator("#history-button").click()
  await expect(page.locator("#action-area .page-revision-header")).toBeVisible()

  await heldActiveLists[0].continue()
  await page.waitForTimeout(500)
  await expect(page.locator("#action-area .page-revision-header")).toBeVisible()
  await expect(page.locator(".file-list")).toHaveCount(0)
})

test("a late failed Files response cannot replace History or show an error", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldActiveLists: Route[] = []
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      if (
        JSON.parse(route.request().postData() ?? "{}").deleted === false &&
        heldActiveLists.length === 0
      ) {
        heldActiveLists.push(route)
        return
      }
      await route.continue()
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect.poll(() => heldActiveLists.length).toBe(1)

  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await page.locator("#history-button").click()
  await expect(page.locator("#action-area .page-revision-header")).toBeVisible()

  await heldActiveLists[0].fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify({
      type: "failure",
      status: 500,
      data: JSON.stringify([{ message: 1 }, "Stale failure"])
    })
  })
  await page.waitForTimeout(500)

  await expect(page.getByText("Stale failure")).toHaveCount(0)
  await expect(page.locator("#action-area .page-revision-header")).toBeVisible()
  await expect(page.locator(".file-list")).toHaveCount(0)
})

test("an initial file-list 503 shows no result area instead of a false empty inventory", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const pageErrors: string[] = []
  const mutationRequests: string[] = []
  let fileListRequests = 0
  page.on("pageerror", (error) => pageErrors.push(error.message))
  page.on("request", (request) => {
    const search = new URL(request.url()).search
    if (
      [
        "?/fileDelete",
        "?/fileEdit",
        "?/fileMove",
        "?/fileRestore",
        "?/fileUpload"
      ].includes(search)
    ) {
      mutationRequests.push(search)
    }
  })

  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      fileListRequests += 1
      if (fileListRequests === 1) {
        await route.fulfill({
          status: 503,
          contentType: "text/plain",
          body: "file service unavailable"
        })
        return
      }
      await route.continue()
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()

  // Source Wikidot leaves the action area hidden and empty for a failed
  // attachment-list request (issue #2273): no result area, no error dialog,
  // no toast and no retry control. Reproduce that absence rather than invent
  // a recovery affordance, and never read the failure as "no files".
  await expect(page.locator(".file-list .file-row")).toHaveCount(0)
  await expect(page.locator(".file-list .file-list-message")).toHaveCount(0)
  await expect(page.getByRole("alert")).toHaveCount(0)
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(0)
  // Live Wikidot does not open the entire #action-area before an authoritative
  // Files read succeeds; the original shows no Close, Upload or Restore.
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect
    .poll(() => page.locator("#action-area").evaluate((area) => area.children.length))
    .toBe(0)
  await expect(page.locator(FILE_ACTION_ROW)).not.toBeVisible()
  await expect(page.locator("#action-area .action-area-close")).not.toBeVisible()
  expect(pageErrors).toEqual([])
  expect(mutationRequests).toEqual([])

  // Clicking Files again retries without an invented in-pane Retry button.
  await page.locator("#files-button").click()
  await expect(page.locator("#action-area")).not.toHaveClass(/hidden/u)
  await expect(page.locator(".file-list .file-row")).toHaveCount(1)
  await expect(page.locator(".file-list")).toContainText(ACTIVE_FILE_NAME)
  await expect(page.getByRole("alert")).toHaveCount(0)
  expect(fileListRequests).toBe(2)
  expect(pageErrors).toEqual([])
  expect(mutationRequests).toEqual([])
})

test("a pending first Files response keeps the entire action area hidden", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)

  let releaseInitial: (() => void) | undefined
  let markIntercepted: (() => void) | undefined
  const intercepted = new Promise<void>((resolve) => {
    markIntercepted = resolve
  })
  const withheld = new Promise<void>((resolve) => {
    releaseInitial = resolve
  })

  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      markIntercepted?.()
      await withheld
      await route.continue()
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await intercepted

  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect
    .poll(() => page.locator("#action-area").evaluate((area) => area.children.length))
    .toBe(0)
  await expect(page.locator(".file-panel")).not.toBeVisible()
  await expect(page.locator(".action-area-close")).not.toBeVisible()

  releaseInitial?.()
  await expect(page.locator("#action-area")).not.toHaveClass(/hidden/u)
  await expect(page.locator(".file-list .file-row")).toHaveCount(1)
  await expect(page.locator(".file-list")).toContainText(ACTIVE_FILE_NAME)
})

test("a failed file-list refresh withholds the result area without stale rows", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const mutationRequests: string[] = []
  let deletedListFailed = false
  page.on("request", (request) => {
    const search = new URL(request.url()).search
    if (
      [
        "?/fileDelete",
        "?/fileEdit",
        "?/fileMove",
        "?/fileRestore",
        "?/fileUpload"
      ].includes(search)
    ) {
      mutationRequests.push(search)
    }
  })

  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as { deleted?: boolean }
      if (body.deleted && !deletedListFailed) {
        deletedListFailed = true
        await route.fulfill({
          status: 503,
          contentType: "text/plain",
          body: "file service unavailable"
        })
        return
      }
      await route.continue()
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect(page.locator(".file-list .file-row")).toHaveCount(1)

  await page
    .locator(".file-panel")
    .getByRole("button", { name: "restore", exact: true })
    .click()

  // A failed refresh withholds the result area entirely: no invented error or
  // retry surface, and no stale rows from the superseded active list.
  await expect(page.locator(".file-list .file-row")).toHaveCount(0)
  await expect(page.locator(".file-list")).toHaveCount(0)
  await expect(page.getByRole("alert")).toHaveCount(0)
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(0)
  expect(deletedListFailed).toBe(true)
  expect(mutationRequests).toEqual([])
})
