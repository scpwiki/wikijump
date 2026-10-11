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

const DELETED_FILE_NAMES = [
  "files-restore-race-deleted-a.txt",
  "files-restore-race-deleted-b.txt"
]
const ACTIVE_FILE_NAME = "theme-lab-visual-fixture"

const expectDeletedRestoreView = async (page: import("@playwright/test").Page) => {
  await expect(
    page
      .locator(".file-list .file-row")
      .getByRole("link", { name: "restore", exact: true })
  ).toHaveCount(DELETED_FILE_NAMES.length)
  for (const name of DELETED_FILE_NAMES) {
    await expect(page.locator(".file-list")).toContainText(name)
  }
}

test("a stale active file-list response cannot replace the Restore view", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldActiveLists: Route[] = []
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as { deleted?: boolean }
      if (body.deleted === false && heldActiveLists.length === 0) {
        // Hold the initial active-files request so it resolves after Restore.
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

  const restoreToggle = page
    .locator(".file-panel")
    .getByRole("button", { name: "restore", exact: true })
    .first()
  await restoreToggle.click()

  await expectDeletedRestoreView(page)

  // Release the older active-files response only after the Restore view wins.
  const staleActiveResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).search === "?/fileList" &&
      (response.request().postData() ?? "").includes('"deleted":false')
  )
  await heldActiveLists[0].continue()
  await staleActiveResponse
  // The stale body is read and applied a little after the headers arrive, so
  // give that handler a moment to run before asserting the view is unchanged.
  await page.waitForTimeout(500)

  await expectDeletedRestoreView(page)
  await expect(page.locator(".file-list")).not.toContainText(ACTIVE_FILE_NAME)
})

test("a stale failed file-list response cannot show an error or replace the Restore view", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldActiveLists: Route[] = []
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as { deleted?: boolean }
      if (body.deleted === false && heldActiveLists.length === 0) {
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

  await page
    .locator(".file-panel")
    .getByRole("button", { name: "restore", exact: true })
    .first()
    .click()
  await expectDeletedRestoreView(page)

  // Answer the superseded active request with a failure in SvelteKit's
  // devalue action-result shape; it must neither show a popup nor replace rows.
  const staleFailure = page.waitForResponse(
    (response) =>
      new URL(response.url()).search === "?/fileList" &&
      (response.request().postData() ?? "").includes('"deleted":false')
  )
  await heldActiveLists[0].fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify({
      type: "failure",
      status: 500,
      data: JSON.stringify([{ message: 1 }, "Stale failure"])
    })
  })
  await staleFailure
  await page.waitForTimeout(500)

  await expect(page.getByText("Stale failure")).toHaveCount(0)
  await expectDeletedRestoreView(page)
})

test("an initial file-list 503 shows retry instead of a successful empty inventory", async ({
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

  await expect(page.getByRole("alert")).toContainText("Unable to load files.")
  await expect(page.locator(".file-list:not(.file-list-message)")).toHaveCount(0)
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toBeVisible()
  await expect(page.locator(".file-list .file-row")).toHaveCount(0)

  await page.getByRole("button", { name: "Retry", exact: true }).click()
  await expect(page.locator(".file-list .file-row")).toHaveCount(1)
  await expect(page.locator(".file-list")).toContainText(ACTIVE_FILE_NAME)
  await expect(page.getByRole("alert")).toHaveCount(0)
  expect(fileListRequests).toBe(2)
  expect(pageErrors).toEqual([])
  expect(mutationRequests).toEqual([])
})

test("a failed file-list refresh retains rows and hides actions until retry", async ({
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
  await expect(page.getByRole("alert")).toContainText("Unable to load files.")
  await expect(page.locator(".file-list .file-row")).toContainText(ACTIVE_FILE_NAME)
  await expect(page.getByRole("button", { name: "delete", exact: true })).toHaveCount(0)
  await expect(page.locator(".action-row.file-action")).toBeHidden()
  expect(deletedListFailed).toBe(true)
  expect(mutationRequests).toEqual([])
})
