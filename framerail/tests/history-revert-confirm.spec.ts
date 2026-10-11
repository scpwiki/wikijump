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

const pageEntry = (timeline: number, comments: string) => ({
  history_kind: "page",
  history_row_id: `page-${timeline}`,
  timeline_number: timeline,
  page_revision_number: timeline,
  history_action_revision_id: null,
  revision_id: 9_100_000 + timeline,
  revision_type: "regular",
  created_at: "2026-10-01T00:00:00Z",
  revision_number: timeline,
  page_id: 3000340,
  site_id: 6000005,
  user_id: 123,
  author: null,
  changes: [],
  comments,
  wikitext: null,
  compiled_body_html: null,
  compiled_body_styles: null,
  compiled_top_bar_html: null,
  compiled_side_bar_html: null,
  compiled_at: null,
  compiled_generator: null
})

const HISTORY = [pageEntry(2, "latest revision"), pageEntry(1, "older revision")]

/** Open the history pane with crafted entries and a chosen edit permission. */
const openHistory = async (
  page: import("@playwright/test").Page,
  canEdit: boolean,
  rollbackRequests: Route[]
) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.route(
    (url) => url.search === "?/history",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: successBody({ res: HISTORY })
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
    (url) => url.search === "?/rollback",
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
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.locator("#history-button").click()
  await expect(page.getByText("older revision")).toBeVisible()
}

test("anonymous viewers get no Revert control on page history", async ({ page }) => {
  const rollbackRequests: Route[] = []
  await openHistory(page, false, rollbackRequests)

  await expect(page.getByRole("link", { name: "R", exact: true })).toHaveCount(0)
  expect(rollbackRequests).toHaveLength(0)
})

test("Revert asks for confirmation and sends nothing when cancelled", async ({
  page
}) => {
  const rollbackRequests: Route[] = []
  page.on("dialog", (dialog) => void dialog.dismiss())
  await openHistory(page, true, rollbackRequests)

  await page.getByRole("link", { name: "R", exact: true }).first().click()
  await page.waitForTimeout(300)

  expect(rollbackRequests).toHaveLength(0)
})

test("confirmed Revert sends one rollback request", async ({ page }) => {
  const rollbackRequests: Route[] = []
  page.on("dialog", (dialog) => void dialog.accept())
  await openHistory(page, true, rollbackRequests)

  const rollbackSent = page.waitForResponse(
    (response) => new URL(response.url()).search === "?/rollback"
  )
  await page.getByRole("link", { name: "R", exact: true }).first().click()
  await rollbackSent

  expect(rollbackRequests).toHaveLength(1)
})
