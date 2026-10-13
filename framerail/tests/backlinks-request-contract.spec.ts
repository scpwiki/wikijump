import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"
import type { Request } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

const fixturePort = process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"

test.beforeEach(({ browserName }, testInfo) => {
  test.skip(
    browserName === "webkit" && testInfo.project.name === "webkit",
    "WebKit pane acceptance runs against the real-TLS browser-support fixture"
  )
})

const openBacklinksPane = async (page: import("@playwright/test").Page) => {
  await page.goto("/scp-173", { waitUntil: "domcontentloaded" })
  await page.waitForFunction(() => {
    const content = document.querySelector("#page-content")
    return content !== null && content.firstChild?.nodeType !== Node.COMMENT_NODE
  })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await waitForSvelteDelegatedHandler(page, "#backlinks-button")
  await page.locator("#backlinks-button").click()
}

test("Backlinks lazy read posts an empty body to the route action and renders the server-resolved result", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const requests: {
    contentType: string | undefined
    method: string
    postData: string | null
    url: string
  }[] = []
  const actionStatuses: number[] = []
  await page.route(
    (url) => url.search === "?/backlinks",
    async (route) => {
      const request = route.request()
      requests.push({
        contentType: request.headers()["content-type"],
        method: request.method(),
        postData: request.postData(),
        url: request.url()
      })
      const response = await route.fetch()
      actionStatuses.push(response.status())
      await route.fulfill({ response })
    }
  )

  await openBacklinksPane(page)

  // The real SvelteKit action must accept the pane's request, and the pane must
  // render the backlink the fixture Deepwell resolved for the route slug.
  const backlink = page
    .locator("#page-backlinks-list")
    .getByRole("link", { name: "SCP Foundation" })
  await expect(backlink).toBeVisible()
  await expect(backlink).toHaveAttribute("href", "/scp-173-parent")

  expect(actionStatuses, "the route action must accept the lazy read").toEqual([200])
  expect(requests, "opening the pane must start exactly one lazy read").toHaveLength(1)
  const [request] = requests
  const requestUrl = new URL(request.url)
  expect(request.method).toBe("POST")
  expect(requestUrl.pathname).toBe("/scp-173")
  // The request must not smuggle the page identifier or any other field into
  // the query or body: the action resolves both site and slug server-side.
  expect(requestUrl.search).toBe("?/backlinks")
  expect(request.postData ?? "").toBe("")
  // fetch() with a string body sends this form-compatible content type, which
  // is the shape SvelteKit accepts for a form action.
  expect(request.contentType?.toLowerCase()).toBe("text/plain;charset=utf-8")

  const readRequests = (await (
    await page.request.get(`http://127.0.0.1:${fixturePort}/last-page-read-requests`)
  ).json()) as { backlinksView: Record<string, unknown>[] }
  expect(
    readRequests.backlinksView,
    "the action must resolve site and slug itself"
  ).toEqual([{ site_id: 6000005, page: "scp-173" }])
})

test("Backlinks cancels the previous read and ignores its late response after the page changes", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))
  const requestedPaths: string[] = []
  let heldRequest: Request | undefined
  let releaseHeld: (() => void) | undefined
  await page.route(
    (url) => url.search === "?/backlinks",
    async (route) => {
      const request = route.request()
      requestedPaths.push(new URL(request.url()).pathname)
      // Complete both real reads, but hold the first response until after the
      // route has changed so a late delivery can be observed.
      const response = await route.fetch()
      if (!heldRequest) {
        heldRequest = request
        await new Promise<void>((resolve) => {
          releaseHeld = resolve
        })
      }
      await route.fulfill({ response })
    }
  )

  await page.goto("/page-workflow-probe", { waitUntil: "domcontentloaded" })
  await page.waitForFunction(() => {
    const content = document.querySelector("#page-content")
    return content !== null && content.firstChild?.nodeType !== Node.COMMENT_NODE
  })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await waitForSvelteDelegatedHandler(page, "#backlinks-button")
  await page.locator("#backlinks-button").click()
  // The probe page's read is deliberately held in flight, so the pane is still
  // loading when the route changes.
  await expect(page.locator("#page-backlinks-list")).toContainText("Loading…")
  await expect.poll(() => heldRequest !== undefined).toBe(true)

  // Navigate to another page without closing the pane: the pane stays mounted
  // and starts the read for the new page while the first read is in flight.
  await page.locator("#navigate-history-target").click()
  await expect(page).toHaveURL(/\/scp-173$/u)
  const backlink = page
    .locator("#page-backlinks-list")
    .getByRole("link", { name: "SCP Foundation" })
  await expect(backlink).toBeVisible()
  expect(requestedPaths).toEqual(["/page-workflow-probe", "/scp-173"])

  // Deliver the first page's response after the new page's read has rendered:
  // it must never replace the result that belongs to the current page.
  releaseHeld?.()
  await page.waitForTimeout(250)
  await expect(backlink).toBeVisible()
  await expect(page.locator("#page-backlinks-list")).not.toContainText(
    "No pages link to this page."
  )

  await expect
    .poll(() => heldRequest?.failure()?.errorText ?? null, {
      message: "starting a read for the new page must cancel the previous read"
    })
    .toMatch(/ABORTED/u)

  expect(
    pageErrors,
    "canceling and replacing a backlinks read must not raise unhandled errors"
  ).toEqual([])
})
