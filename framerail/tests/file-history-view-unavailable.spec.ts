import { expect, test } from "./hermetic-playwright"

const appURL = (projectName: string) =>
  projectName === "webkit-https-edit-meta"
    ? `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`
    : `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

const encodeDevalue = (value: unknown): string => {
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
  JSON.stringify({ type: "success", status: 200, data: encodeDevalue(value) })

test("file history without a page snapshot exposes no inert View Version action", async ({
  page
}, testInfo) => {
  test.skip(
    testInfo.project.name === "webkit",
    "WebKit history coverage runs against the HTTPS fixture project"
  )

  const baseURL = appURL(testInfo.project.name)
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki",
    cookie: "wikijump_token=fixture-session-token",
    origin: new URL(baseURL).origin
  })
  await page.route(
    (url) => url.search === "?/history",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({
          res: [
            {
              history_kind: "file",
              history_row_id: "file-2151",
              timeline_number: 31,
              page_revision_number: null,
              history_action_revision_id: null,
              revision_id: 9002151,
              revision_type: "regular",
              created_at: "2026-04-21T00:00:00Z",
              revision_number: 0,
              page_id: 3000345,
              site_id: 6000005,
              user_id: 123,
              author: null,
              changes: ["files"],
              comments: 'Uploaded file "ESTAB_COMMS.png"',
              wikitext: null,
              compiled_body_html: null,
              compiled_body_styles: null,
              compiled_top_bar_html: null,
              compiled_side_bar_html: null,
              compiled_at: null,
              compiled_generator: null
            }
          ]
        })
      })
    }
  )

  await page.goto(`${baseURL}/authoring-history-probe#_history`)
  await page.waitForLoadState("networkidle")

  const row = page.locator(
    '#action-area table.page-history tr[id="revision-row-file-2151"]'
  )
  await expect(row).toContainText('Uploaded file "ESTAB_COMMS.png"')
  expect(await row.innerHTML()).toContain("Historical page snapshot unavailable")
  await expect(row.locator("a[title='View page revision']")).toHaveCount(0)
  await expect(page.locator("#history-subarea .page-revision-header")).toHaveCount(0)
})
