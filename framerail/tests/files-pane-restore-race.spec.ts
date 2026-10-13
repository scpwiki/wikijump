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

test("a late deleted-files response cannot replace the active list after reopening", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldDeletedLists: Route[] = []
  let deletedListRequests = 0
  const activeFile = (fileId: number, name: string) => ({
    file_id: fileId,
    file_created_at: "2026-10-01T00:00:00Z",
    file_updated_at: null,
    file_deleted_at: null,
    page_id: 3000340,
    revision_id: fileId + 1000,
    revision_type: "create",
    revision_created_at: "2026-10-01T00:00:00Z",
    revision_number: 0,
    revision_user_id: 123,
    name,
    data: null,
    mime: "text/plain",
    size: 12,
    s3_hash: "fixture-hash",
    revision_comments: "",
    hidden_fields: []
  })
  const deletedFile = (fileId: number, name: string) => ({
    ...activeFile(fileId, name),
    file_deleted_at: "2026-10-02T00:00:00Z",
    revision_type: "delete"
  })
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as {
        deleted?: boolean
      }
      const payload = body.deleted
        ? [
            deletedFile(91301, "restore-race-deleted-a.txt"),
            deletedFile(91302, "restore-race-deleted-b.txt")
          ]
        : [
            activeFile(91201, "restore-race-active-a.txt"),
            activeFile(91202, "restore-race-active-b.txt")
          ]
      if (body.deleted) {
        deletedListRequests += 1
        if (deletedListRequests === 2) {
          heldDeletedLists.push(route)
          return
        }
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: payload })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await expect(page.locator(".file-list")).toContainText("restore-race-active-a.txt")

  const restoreToggle = page
    .locator(".file-panel")
    .getByRole("button", { name: "restore", exact: true })
  await restoreToggle.click()
  await expect(page.locator(".file-list")).toContainText("restore-race-deleted-a.txt")

  await restoreToggle.click()
  await expect.poll(() => heldDeletedLists.length).toBe(1)

  await page.locator("#action-area .action-area-close").click()
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await expect
    .poll(() => page.locator("#action-area").evaluate((area) => area.children.length))
    .toBe(0)

  await page.locator("#files-button").click()
  await expect(page.locator(".file-list")).toContainText("restore-race-active-a.txt")

  await heldDeletedLists[0].continue()
  await page.waitForTimeout(500)
  // Only the last user-selected mode wins: the stale deleted-files reply
  // cannot flip the reopened pane back to Restore rows.
  await expect(page.locator(".file-list")).toContainText("restore-race-active-a.txt")
  await expect(page.locator(".file-list")).not.toContainText("restore-race-deleted-a.txt")
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

// SvelteKit form-action responses carry devalued payloads. This is the same
// minimal serializer the Backlinks/Watchers failure spec uses; it lets the
// tests below fulfill an authoritative empty file list without touching file
// data or the fixture server.
const devalueFileList = (value: unknown): string => {
  const flat: unknown[] = []
  const add = (item: unknown): number => {
    const index = flat.length
    flat.push(null)
    if (Array.isArray(item)) flat[index] = item.map(add)
    else if (item !== null && typeof item === "object") {
      const encoded: Record<string, number> = {}
      for (const [key, child] of Object.entries(item)) encoded[key] = add(child)
      flat[index] = encoded
    } else flat[index] = item
    return index
  }
  add(value)
  return JSON.stringify(flat)
}

const fileListActionResult = (type: "success" | "failure", value: unknown) =>
  JSON.stringify({
    type,
    status: type === "success" ? 200 : 500,
    data: devalueFileList(value)
  })

test("failed file-list requests stay distinct from an empty inventory across failure modes", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const pageErrors: string[] = []
  const mutationRequests: string[] = []
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

  // Every failure mode must reproduce the same frozen source absence (issue
  // #2273): the Wikidot action area stays hidden and empty, with no result
  // area, no error dialog, no toast and no retry control, and the failure is
  // never read as "no files".
  type FailureMode = "network" | "malformed" | "action-failure"
  let mode: FailureMode = "network"
  // The same predicate reference must be reused for unroute below:
  // Playwright matches function predicates by reference.
  const fileListUrl = (url: URL) => url.search === "?/fileList"
  await page.route(fileListUrl, async (route) => {
    if (mode === "network") return route.abort("failed")
    if (mode === "malformed") {
      return route.fulfill({
        status: 200,
        contentType: "text/plain",
        body: "MALFORMED file list detail"
      })
    }
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: fileListActionResult("failure", {
        message: "ACTION file list detail"
      })
    })
  })

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")

  const modes: FailureMode[] = ["network", "malformed", "action-failure"]
  for (const failure of modes) {
    mode = failure
    // Clicking Files retries through the page action; there is no invented
    // in-pane Retry button to click.
    await page.locator("#files-button").click()
    await expect(page.locator(".file-list .file-row")).toHaveCount(0)
    await expect(page.locator(".file-list .file-list-message")).toHaveCount(0)
    await expect(page.getByRole("alert")).toHaveCount(0)
    await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(0)
    await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
    await expect(page.locator("#action-area .action-area-close")).not.toBeVisible()
    await expect(
      page.getByText(/MALFORMED file list detail|ACTION file list detail/u)
    ).toHaveCount(0)
    expect(
      pageErrors,
      `file-list ${failure} must not escape as an unhandled browser error`
    ).toEqual([])
    expect(mutationRequests, `file-list ${failure} must stay read-only`).toEqual([])
  }

  // A later successful load still presents the authoritative file data.
  await page.unroute(fileListUrl)
  await page.locator("#files-button").click()
  await expect(page.locator("#action-area")).not.toHaveClass(/hidden/u)
  await expect(page.locator(".file-list .file-row")).toHaveCount(1)
  await expect(page.locator(".file-list")).toContainText(ACTIVE_FILE_NAME)
  expect(pageErrors).toEqual([])
  expect(mutationRequests).toEqual([])
})

test("a successfully parsed empty file list shows the empty inventory wording", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))

  // Only an authoritative empty response may render the source-compatible
  // empty wording (issue #2273). No file data is touched; the fixture server
  // still holds its file, only this response claims the empty list.
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: fileListActionResult("success", { res: [] })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()

  await expect(page.locator("#action-area")).not.toHaveClass(/hidden/u)
  await expect(page.locator(".file-list .file-row")).toHaveCount(0)
  // The hermetic fixture server echoes untranslated i18n keys, so the
  // fixture renders the key while a real locale bundle resolves it to the
  // source-compatible "No files for this page." (locales/fluent/wiki-page/en.ftl).
  // Either way, this wording is reachable only after an authoritative empty
  // success, never from a failed request.
  await expect(page.locator(".file-list .file-list-message")).toContainText(
    /No files for this page\.|wiki-page-file-no-files/u
  )
  await expect(page.getByRole("alert")).toHaveCount(0)
  expect(pageErrors).toEqual([])
})
