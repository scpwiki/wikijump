import type { Route } from "@playwright/test"

import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
// The browser-support matrix declares more than one web server, so it does
// not infer a base URL: navigate against the app origin explicitly, and use
// the HTTPS fixture origin for the WebKit acceptance project.
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const HTTPS_APP_URL = `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token",
  origin: APP_URL
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
      for (const [key, child] of Object.entries(item as Record<string, unknown>)) {
        encoded[key] = add(child)
      }
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

// A wide, realistic revision row: a long unbroken file name, a multibyte
// comment and a full timestamp, which is what pushes the intrinsic width of
// the revision table past a 320px phone.
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
  name: "an-attachment-name-that-is-far-too-long-for-a-320px-phone-viewport.gif",
  data: null,
  mime: "image/gif",
  size: 2431188,
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
  mime: "image/gif",
  size: 2431188,
  changes: [],
  // Long multibyte content must stay in the DOM and in the scroll region,
  // never be truncated away to keep the table narrow.
  comments:
    "添付ファイルの置き換え：スキャン画像の再アップロードと注記の追加 その他の注記 その他の注記",
  hidden: []
}

const openFileHistory = async (
  page: import("@playwright/test").Page,
  projectName: string,
  rollbackRequests: Route[]
) => {
  const appUrl = projectName.startsWith("webkit-https") ? HTTPS_APP_URL : APP_URL
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
        body: successBody({ res: { can_edit: true } })
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

  await page.goto(`${appUrl}/page-workflow-probe`, {
    waitUntil: "domcontentloaded"
  })
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

const regionGeometry = (page: import("@playwright/test").Page) =>
  page.evaluate(() => {
    const region = document.querySelector("#file-history-list")
    if (!(region instanceof HTMLElement)) {
      throw new Error("File history scroll region is missing")
    }
    return {
      documentWidth: document.documentElement.scrollWidth,
      viewportWidth: document.documentElement.clientWidth,
      regionClientWidth: region.clientWidth,
      regionContentWidth: region.scrollWidth,
      regionOverflowX: getComputedStyle(region).overflowX
    }
  })

test("File History stays contained in a local scroll region on phones", async ({
  page
}, testInfo) => {
  test.skip(
    testInfo.project.name === "webkit",
    "WebKit runs this case on the HTTPS fixture"
  )
  const rollbackRequests: Route[] = []
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  for (const width of [280, 320, 375, 390]) {
    await page.setViewportSize({ width, height: 844 })
    await openFileHistory(page, testInfo.project.name, rollbackRequests)

    const region = page.locator("#file-history-list")
    await expect(region).toBeVisible()
    await expect(region).toHaveAttribute("role", "region")
    await expect(region).toHaveAttribute("tabindex", "0")
    await expect(region).not.toHaveAttribute("aria-label", "")
    // Every source column label is still rendered.
    await expect(region.locator(".revision-header .revision-attribute")).toHaveCount(10)

    const geometry = await regionGeometry(page)
    expect(
      geometry.documentWidth,
      `${width}px viewport: ${JSON.stringify(geometry)}`
    ).toBeLessThanOrEqual(geometry.viewportWidth + 1)
    expect(
      geometry.regionClientWidth,
      `${width}px viewport: ${JSON.stringify(geometry)}`
    ).toBeLessThanOrEqual(geometry.viewportWidth)
    // The table really is wider than the pane, so scrolling is the access
    // path rather than a hidden overflow.
    expect(
      geometry.regionContentWidth,
      `${width}px viewport: ${JSON.stringify(geometry)}`
    ).toBeGreaterThan(geometry.regionClientWidth)
    expect(geometry.regionOverflowX).toBe("auto")

    // Metadata is preserved in full, not truncated to fit.
    await expect(page.locator(".revision-row")).toContainText(file.name)
    await expect(page.locator(".revision-row .revision-attribute.comments")).toHaveText(
      revision.comments
    )
    await expect(page.locator(".revision-row .revision-attribute.mime")).toHaveText(
      "image/gif"
    )
    await expect(page.locator(".revision-row .revision-attribute.size")).toHaveText(
      String(revision.size)
    )

    // Reopening for the next width must not compound the layout.
    await page.locator("#files-button").click()
  }
})

test("File History keeps the Files list scroll region and reaches every column", async ({
  page
}, testInfo) => {
  test.skip(
    testInfo.project.name === "webkit",
    "WebKit runs this case on the HTTPS fixture"
  )
  const rollbackRequests: Route[] = []
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.setViewportSize({ width: 320, height: 844 })
  await openFileHistory(page, testInfo.project.name, rollbackRequests)

  // The existing Files list containment is unaffected.
  const fileListScroll = page.locator(".file-list-scroll").first()
  await expect(fileListScroll).toHaveCSS("overflow-x", "auto")

  // Both the first and the last column are brought into the region's
  // visible box by scrolling that the region itself owns.
  const columns = page.locator("#file-history-list .revision-row .revision-attribute")
  await expect(columns).toHaveCount(10)
  const columnCount = await columns.count()
  const boundaryReachable = await page.evaluate((lastIndex: number) => {
    const region = document.querySelector("#file-history-list")
    if (!(region instanceof HTMLElement)) throw new Error("scroll region missing")
    const withinRegion = (index: number) => {
      const cell = document.querySelectorAll(
        "#file-history-list .revision-row .revision-attribute"
      )[index]
      if (!(cell instanceof HTMLElement)) throw new Error(`column ${index} missing`)
      cell.scrollIntoView({ block: "nearest", inline: "start" })
      const cellBox = cell.getBoundingClientRect()
      const regionBox = region.getBoundingClientRect()
      return (
        cellBox.left >= regionBox.left - 1 &&
        cellBox.right <= regionBox.right + 1 &&
        regionBox.right <= document.documentElement.clientWidth + 1
      )
    }
    region.scrollLeft = 0
    return {
      first: withinRegion(0),
      last: withinRegion(lastIndex),
      scrolled: region.scrollLeft > 0
    }
  }, columnCount - 1)
  expect(boundaryReachable.first).toBe(true)
  expect(boundaryReachable.last).toBe(true)
  expect(boundaryReachable.scrolled).toBe(true)

  const revertControl = page.locator(
    ".revision-row .revision-attribute.action a, .revision-row .revision-attribute.action button"
  )
  await expect(revertControl).toHaveCount(1)
  await revertControl.focus()
  await expect(revertControl).toBeFocused()

  // Exploring the contained region must not move the document, and a
  // read-only history visit submits no file mutation.
  const documentWidth = await page.evaluate(() => document.documentElement.scrollWidth)
  expect(documentWidth).toBeLessThanOrEqual(321)
  expect(rollbackRequests).toHaveLength(0)
})
