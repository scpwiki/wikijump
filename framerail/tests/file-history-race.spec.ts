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

const FILE_A = {
  file_id: 7_100_001,
  name: "history-race-a.txt",
  comments: "file A revision"
}
const FILE_B = {
  file_id: 7_100_002,
  name: "history-race-b.txt",
  comments: "file B revision"
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

const fileRecord = (file: typeof FILE_A) => ({
  file_id: file.file_id,
  file_created_at: "2026-10-01T00:00:00Z",
  file_updated_at: null,
  revision_id: file.file_id + 1000,
  revision_created_at: "2026-10-01T00:00:00Z",
  revision_user_id: 123,
  revision_type: "create",
  name: file.name,
  mime: "text/plain",
  size: 17,
  revision_comments: file.comments
})

const revisionRecord = (file: typeof FILE_A) => ({
  revision_id: file.file_id + 2000,
  revision_type: "create",
  created_at: "2026-10-01T00:00:00Z",
  revision_number: 0,
  file_id: file.file_id,
  page_id: 3000340,
  site_id: 6000005,
  user_id: 123,
  name: file.name,
  s3_hash: null,
  mime: "text/plain",
  size: 17,
  comments: file.comments
})

const successBody = (value: unknown) =>
  JSON.stringify({ type: "success", status: 200, data: devalue(value) })

test("a superseded file-history response cannot replace the selected file's revisions", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  const heldHistoryForA: Route[] = []

  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [fileRecord(FILE_A), fileRecord(FILE_B)] })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/fileHistory",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as { fileId?: number }
      if (body.fileId === FILE_A.file_id) {
        // Hold the earlier history request for file A until B has been selected.
        heldHistoryForA.push(route)
        return
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: [revisionRecord(FILE_B)] })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  const rowFor = (file: typeof FILE_A) =>
    page.locator(`.file-row[data-id="${file.file_id}"]`)
  await expect(rowFor(FILE_A)).toBeVisible()
  await expect(rowFor(FILE_B)).toBeVisible()

  await rowFor(FILE_A).getByRole("link", { name: "history", exact: true }).click()
  await expect.poll(() => heldHistoryForA.length).toBe(1)

  await rowFor(FILE_B).getByRole("link", { name: "history", exact: true }).click()
  await expect(page.locator(".revision-row")).toContainText(FILE_B.comments)

  // Release the earlier file-A response only after file B's revisions are shown.
  const staleHistoryResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).search === "?/fileHistory" &&
      (response.request().postData() ?? "").includes(`"fileId":${FILE_A.file_id}`)
  )
  await heldHistoryForA[0].fulfill({
    status: 200,
    contentType: "application/json",
    body: successBody({ res: [revisionRecord(FILE_A)] })
  })
  await staleHistoryResponse
  // The stale body is applied shortly after its headers arrive.
  await page.waitForTimeout(500)

  await expect(page.locator(".revision-row")).toContainText(FILE_B.comments)
  await expect(page.locator(".revision-row")).not.toContainText(FILE_A.comments)
})
