import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
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

test("File History shows a page title link only for the current site page", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.route(
    (url) => url.search === "?/fileList",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as {
        pageId: number
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({
          res: [
            {
              file_id: 77,
              file_created_at: "2026-10-01T00:00:00Z",
              file_updated_at: null,
              file_deleted_at: null,
              page_id: body.pageId,
              revision_id: 88,
              revision_type: "create",
              revision_created_at: "2026-10-01T00:00:00Z",
              revision_number: 0,
              revision_user_id: 123,
              name: "history-probe.txt",
              data: null,
              mime: "text/plain",
              size: 12,
              s3_hash: "fixture-hash",
              revision_comments: "",
              hidden_fields: []
            }
          ]
        })
      })
    }
  )
  await page.route(
    (url) => url.search === "?/fileHistory",
    async (route) => {
      const body = JSON.parse(route.request().postData() ?? "{}") as {
        pageId: number
      }
      const revision = (
        revisionId: number,
        revisionNumber: number,
        pageId: number,
        siteId = 6000005
      ) => ({
        revision_id: revisionId,
        revision_type: "create",
        created_at: "2026-10-01T00:00:00Z",
        revision_number: revisionNumber,
        file_id: 77,
        page_id: pageId,
        site_id: siteId,
        user_id: -1,
        name: "history-probe.txt",
        s3_hash: null,
        mime: "text/plain",
        size: 12,
        changes: [],
        comments: null,
        hidden: []
      })
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({
          res: [
            revision(89, 0, body.pageId),
            revision(90, 1, body.pageId + 1),
            revision(91, 2, body.pageId, 6000006)
          ]
        })
      })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()
  await page
    .locator(".file-row")
    .getByRole("link", { name: "history", exact: true })
    .click()

  const currentPage = page.locator('.revision-row [class~="page"] a')
  await expect(currentPage).toHaveText("page-workflow-probe")
  await expect(currentPage).toHaveAttribute("href", "/page-workflow-probe")

  const unavailablePage = page.locator('.revision-row [class~="page"] span')
  await expect(unavailablePage).toHaveCount(2)
  await expect(unavailablePage).toHaveText(["—", "—"])
})
