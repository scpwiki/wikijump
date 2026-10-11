import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"
import type { Route } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

type Pane = "backlinks" | "watchers"
type ResponseMode =
  "unavailable" | "network" | "malformed" | "action-failure" | "empty" | "populated"

test.beforeEach(({ browserName }, testInfo) => {
  test.skip(
    browserName === "webkit" && testInfo.project.name === "webkit",
    "WebKit pane acceptance runs against the real-TLS browser-support fixture"
  )
})

const devalue = (value: unknown): string => {
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

const actionResult = (type: "success" | "failure", value: unknown) =>
  JSON.stringify({
    type,
    status: type === "success" ? 200 : 500,
    data: devalue(value)
  })

const populatedResult = (pane: Pane) =>
  pane === "backlinks"
    ? { res: [{ slug: "linked-fixture-page", title: "Linked fixture page" }] }
    : {
        res: [
          {
            "user-id": 900001,
            "user-slug": "fixture-watcher",
            "user-name": "Fixture Watcher",
            "user-karma": 1,
            "user-avatar-data": "",
            "user-profile-url": "/profile/fixture-watcher"
          }
        ]
      }

const openPane = async (page: import("@playwright/test").Page, pane: Pane) => {
  await page.goto("/scp-173", { waitUntil: "domcontentloaded" })
  await page.waitForFunction(() => {
    const content = document.querySelector("#page-content")
    return content !== null && content.firstChild?.nodeType !== Node.COMMENT_NODE
  })
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  const button = `#${pane}-button`
  await waitForSvelteDelegatedHandler(page, button)
  await page.locator(button).click()
}

const selectPane = async (page: import("@playwright/test").Page, pane: Pane) => {
  const button = `#${pane}-button`
  if (!(await page.locator(button).isVisible())) {
    await page.locator("#more-options-button").click()
  }
  await waitForSvelteDelegatedHandler(page, button)
  await page.locator(button).click()
}

for (const pane of ["backlinks", "watchers"] as const) {
  test(`${pane} failures stay distinct from empty results and can be retried`, async ({
    page
  }) => {
    await page.setExtraHTTPHeaders(SITE_HEADERS)
    const pageErrors: string[] = []
    page.on("pageerror", (error) => pageErrors.push(error.message))
    const action = `?/${pane}`
    let mode: ResponseMode = "unavailable"
    await page.route(
      (url) => url.search === action,
      async (route) => {
        if (mode === "network") return route.abort("failed")
        if (mode === "unavailable") {
          return route.fulfill({
            status: 503,
            contentType: "text/html",
            body: "INTERNAL SECRET detail"
          })
        }
        if (mode === "malformed") {
          return route.fulfill({
            status: 200,
            contentType: "text/plain",
            body: "MALFORMED SECRET detail"
          })
        }
        if (mode === "action-failure") {
          return route.fulfill({
            status: 200,
            contentType: "application/json",
            body: actionResult("failure", { message: "ACTION SECRET detail" })
          })
        }
        const result = mode === "empty" ? { res: [] } : populatedResult(pane)
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: actionResult("success", result)
        })
      }
    )

    await openPane(page, pane)
    const alert = page.getByRole("alert")
    const retry = page.getByRole("button", { name: `Retry loading ${pane}` })
    for (const failure of [
      "unavailable",
      "network",
      "malformed",
      "action-failure"
    ] as const) {
      if (failure !== "unavailable") {
        mode = failure
        await retry.click()
      }
      await expect(alert).toContainText(`Could not load ${pane}.`)
      await expect(page.getByText(/SECRET detail/u)).toHaveCount(0)
    }

    mode = "empty"
    await retry.click()
    await expect(alert).toHaveCount(0)
    if (pane === "backlinks") {
      await expect(page.getByText("No pages link to this page.")).toBeVisible()
    } else {
      await expect(page.locator(".page-watchers-list li")).toHaveCount(0)
    }

    mode = "populated"
    await page.reload({ waitUntil: "domcontentloaded" })
    await page.waitForFunction(() => {
      const content = document.querySelector("#page-content")
      return content !== null && content.firstChild?.nodeType !== Node.COMMENT_NODE
    })
    await waitForSvelteDelegatedHandler(page, "#more-options-button")
    await page.locator("#more-options-button").click()
    await waitForSvelteDelegatedHandler(page, `#${pane}-button`)
    await page.locator(`#${pane}-button`).click()
    if (pane === "backlinks") {
      await expect(page.getByRole("link", { name: "Linked fixture page" })).toBeVisible()
    } else {
      await expect(page.locator(".page-watchers-list")).toContainText("Fixture Watcher")
    }
    expect(
      pageErrors,
      "pane failures must not escape as unhandled browser errors"
    ).toEqual([])
  })

  test(`${pane} ignores a stale response after close, rapid pane switch, and reopen`, async ({
    page
  }) => {
    await page.setExtraHTTPHeaders(SITE_HEADERS)
    const pageErrors: string[] = []
    page.on("pageerror", (error) => pageErrors.push(error.message))
    let heldRoute: Route | undefined
    let releaseHeldRoute: (() => void) | undefined
    const otherPane = pane === "backlinks" ? "watchers" : "backlinks"
    await page.route(
      (url) => url.search === "?/backlinks" || url.search === "?/watchers",
      async (route) => {
        const requestedPane =
          new URL(route.request().url()).search === "?/backlinks"
            ? "backlinks"
            : "watchers"
        if (requestedPane === pane && !heldRoute) {
          heldRoute = route
          await new Promise<void>((resolve) => {
            releaseHeldRoute = resolve
          })
          try {
            await route.fulfill({
              status: 200,
              contentType: "application/json",
              body: actionResult("success", populatedResult(pane))
            })
          } catch {
            // Closing the pane aborts this request; its late response is ignored.
          }
          return
        }
        const result =
          requestedPane === otherPane
            ? { res: [] }
            : pane === "backlinks"
              ? { res: [{ slug: "fresh-backlink", title: "Fresh backlink" }] }
              : {
                  res: [
                    {
                      "user-id": 900002,
                      "user-slug": "fresh-watcher",
                      "user-name": "Fresh Watcher",
                      "user-karma": 1,
                      "user-avatar-data": "",
                      "user-profile-url": "/profile/fresh-watcher"
                    }
                  ]
                }
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: actionResult("success", result)
        })
      }
    )

    await openPane(page, pane)
    await expect.poll(() => heldRoute !== undefined).toBe(true)
    await page.locator("#action-area .action-area-close").click()
    await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
    await selectPane(page, otherPane)
    if (otherPane === "backlinks") {
      await expect(page.getByText("No pages link to this page.")).toBeVisible()
    } else {
      await expect(page.locator(".page-watchers-list li")).toHaveCount(0)
    }
    await selectPane(page, pane)

    if (pane === "backlinks") {
      await expect(page.getByRole("link", { name: "Fresh backlink" })).toBeVisible()
    } else {
      await expect(page.locator(".page-watchers-list")).toContainText("Fresh Watcher")
    }

    releaseHeldRoute?.()
    await page.waitForTimeout(250)
    if (pane === "backlinks") {
      await expect(page.getByRole("link", { name: "Fresh backlink" })).toBeVisible()
      await expect(page.getByRole("link", { name: "Linked fixture page" })).toHaveCount(0)
    } else {
      await expect(page.locator(".page-watchers-list")).toContainText("Fresh Watcher")
      await expect(page.locator(".page-watchers-list")).not.toContainText(
        "Fixture Watcher"
      )
    }
    expect(
      pageErrors,
      "closing and switching panes must not produce unhandled browser errors"
    ).toEqual([])
  })
}
