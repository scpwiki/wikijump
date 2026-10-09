import {
  expect,
  installNativeEventListenerProbe,
  test,
  waitForNativeEventListener,
  waitForSvelteDelegatedHandler
} from "./hermetic-playwright"

import type { APIResponse } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const AUTHENTICATED_HEADERS = {
  ...SITE_HEADERS,
  cookie: "wikijump_token=fixture-session-token",
  origin: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
}
const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`

test("canonical page URL aliases issue a 301 and preserve route state", async ({
  page,
  request
}) => {
  const redirect = await request.get("/PAGE-WORKFLOW-PROBE?view=history", {
    headers: SITE_HEADERS,
    maxRedirects: 0
  })
  expect(redirect.status()).toBe(301)
  expect(redirect.headers().location).toBe("/page-workflow-probe?view=history")

  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/PAGE-WORKFLOW-PROBE?view=history#history-target")
  await expect(page).toHaveURL(/\/page-workflow-probe\?view=history#history-target$/u)
  await expect(page.locator("#page-content")).toContainText("Page workflow probe")
})

async function expectSuccessfulAction(response: APIResponse) {
  const body = await response.text()
  expect(response.ok(), body).toBe(true)
  expect(body).toContain('"type":"success"')
}

test("article routes carry load and mutation context through Deepwell", async ({
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-article-read-requests`)
  const article = await request.get("/page-workflow-probe", {
    headers: SITE_HEADERS
  })
  const cachedArticle = await request.get("/page-workflow-probe", {
    headers: SITE_HEADERS
  })
  expect(article.ok()).toBe(true)
  expect(article.headers()["x-content-type-options"]).toBe("nosniff")
  expect(cachedArticle.ok()).toBe(true)
  expect(await cachedArticle.text()).toBe(await article.text())
  expect(await article.text()).toContain("Page workflow probe")

  const loadRequests = await request
    .get(`${FIXTURE_URL}/last-article-read-requests`)
    .then((response) => response.json())
  const requestsForProbe = (requests: { route: { slug: string } }[]) =>
    requests.filter(({ route }) => route.slug === "page-workflow-probe")
  expect(requestsForProbe(loadRequests.articleView)).toHaveLength(1)
  expect(requestsForProbe(loadRequests.articleViewCacheMetadata)).toHaveLength(2)

  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/voteGet", {
      headers: {
        ...AUTHENTICATED_HEADERS,
        "content-type": "text/plain;charset=UTF-8"
      },
      data: JSON.stringify({
        siteId: 6000005,
        pageId: 3000340,
        slug: "page-workflow-probe"
      })
    })
  )

  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/edit", {
      headers: AUTHENTICATED_HEADERS,
      multipart: {
        siteId: "6000005",
        pageId: "3000340",
        lastRevisionId: "9000340",
        title: "Page Workflow Probe",
        altTitle: "",
        wikitext: "Cross-layer edit",
        tags: "fixture",
        comments: "route edit"
      }
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/fileRestore", {
      headers: AUTHENTICATED_HEADERS,
      multipart: {
        siteId: "6000005",
        pageId: "3000340",
        lastRevisionId: "41",
        fileId: "42",
        newPage: "",
        newName: "",
        comments: "route file restore"
      }
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/voteCast", {
      headers: {
        ...AUTHENTICATED_HEADERS,
        "content-type": "text/plain;charset=UTF-8"
      },
      data: JSON.stringify({
        siteId: 6000005,
        pageId: 3000340,
        value: 1
      })
    })
  )
  await expectSuccessfulAction(
    await request.post("/page-workflow-probe?/rollback", {
      headers: {
        ...AUTHENTICATED_HEADERS,
        "content-type": "text/plain;charset=UTF-8"
      },
      data: JSON.stringify({
        siteId: 6000005,
        pageId: 3000340,
        revisionNumber: 1,
        lastRevisionId: 9100000,
        comments: "route rollback"
      })
    })
  )

  const pageRequests = await request
    .get(`${FIXTURE_URL}/last-page-write-requests`)
    .then((response) => response.json())
  const fileRequests = await request
    .get(`${FIXTURE_URL}/last-file-requests`)
    .then((response) => response.json())
  const pageReadRequests = await request
    .get(`${FIXTURE_URL}/last-page-read-requests`)
    .then((response) => response.json())

  expect(fileRequests.pageGetFiles).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: {
        deleted: false,
        page_id: 3000340,
        site_id: 6000005
      }
    })
  )
  expect(pageReadRequests.voteList).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        id: 3000340,
        type: "Page"
      })
    })
  )

  expect(pageRequests.pageEdit).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        page: 3000340,
        wikitext: "Cross-layer edit"
      })
    })
  )
  expect(pageRequests.voteSet).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: {
        page_id: 3000340,
        value: 1
      }
    })
  )
  expect(pageRequests.pageRollback).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        page: 3000340,
        revision_number: 1
      })
    })
  )
  expect(fileRequests.fileRestore).toContainEqual(
    expect.objectContaining({
      headers: {
        page: "page-workflow-probe",
        sessionToken: "fixture-session-token",
        siteId: "6000005"
      },
      params: expect.objectContaining({
        file_id: 42,
        page_id: 3000340
      })
    })
  )
})

test("rating pane loads its score and refreshes after plus, minus, and cancel", async ({
  page
}) => {
  const scoreStatuses: number[] = []
  page.on("response", (response) => {
    if (response.url().includes("?/score")) scoreStatuses.push(response.status())
  })
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/page-workflow-rating-probe")
  await waitForSvelteDelegatedHandler(page, "#pagerate-button")
  await page.locator("#pagerate-button").click()

  const rating = page.locator(".page-rate-widget-box")
  const score = rating.locator(".number.prw54353")
  await expect(rating).toBeVisible()
  await expect(score).toHaveText("0")
  expect(scoreStatuses).toEqual([200])

  const refreshAfter = async (
    action: () => Promise<unknown>,
    actionName: string,
    expectedScoreRequests: number,
    expectedScore: string
  ) => {
    const actionResponse = page.waitForResponse((response) =>
      response.url().includes(`?/${actionName}`)
    )
    await action()
    expect((await actionResponse).status()).toBe(200)
    await expect(score).toHaveText(expectedScore)
    await expect.poll(() => scoreStatuses.length).toBe(expectedScoreRequests)
  }

  await refreshAfter(() => rating.locator(".rateup a").click(), "voteCast", 2, "1")
  await page.getByRole("link", { name: "Look who rated this page" }).click()
  await expect(page.locator("#who-rated-page-area")).toContainText("Guest")
  await refreshAfter(() => rating.locator(".ratedown a").click(), "voteCast", 3, "-1")
  await refreshAfter(() => rating.locator(".cancel a").click(), "voteCancel", 4, "0")
  await page.getByRole("link", { name: "Look who rated this page" }).click()
  await expect(page.locator("#who-rated-page-area")).not.toContainText("Guest")
  expect(scoreStatuses).toEqual([200, 200, 200, 200])

  scoreStatuses.length = 0
  await page.goto("/page-workflow-star-probe")
  await waitForSvelteDelegatedHandler(page, "#pagerate-button")
  await page.locator("#pagerate-button").click()
  const starWidget = page.locator(".page-rate-widget")
  await expect(starWidget).toBeVisible()
  await expect(starWidget.locator(".page-rate-widget-start")).toHaveAttribute(
    "data-rating",
    "3"
  )
  await expect.poll(() => scoreStatuses.length).toBe(1)
  const starVoteResponse = page.waitForResponse((response) =>
    response.url().includes("?/voteCast")
  )
  await starWidget.locator('img[alt="4"]').click()
  expect((await starVoteResponse).status()).toBe(200)
  await expect.poll(() => scoreStatuses.length).toBe(2)
  await expect(starWidget.locator(".page-rate-widget-start")).toHaveAttribute(
    "data-rating",
    "4"
  )
  expect(scoreStatuses).toEqual([200, 200])
})

test("autonumbered page creation follows the assigned slug", async ({ page }) => {
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/autonumber-requested/edit/true")
  await waitForNativeEventListener(page, "#editor", "submit")

  await page.locator("input[name='title']").fill("Autonumber browser test")
  await page.locator("textarea[name='wikitext']").fill("Assigned page body")
  await page.locator("textarea[name='comments']").fill("create")
  await page.getByRole("button", { name: "save", exact: true }).click()

  await expect(page).toHaveURL(/\/104$/u)
  await expect(page.locator("#page-content")).toContainText("Assigned page body")
})

test("history ignores a stale revision diff response", async ({ page, request }) => {
  await request.get(`${FIXTURE_URL}/last-page-read-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.getByRole("link", { name: "history", exact: true }).click()
  await expect(page.locator(".revision-diff-controls")).toBeVisible()

  const fromRevision = page.locator("#revision-diff-from")
  const toRevision = page.locator("#revision-diff-to")
  await page.locator(".revision-diff-controls button").nth(1).click()
  await expect
    .poll(async () => {
      const response = await request.get(`${FIXTURE_URL}/page-revision-diff-requests`)
      return (await response.json()).length
    })
    .toBe(1)

  await fromRevision.selectOption("2")
  await toRevision.selectOption("1")
  await page.locator(".revision-diff-controls button").nth(1).click()

  await expect
    .poll(async () => {
      const response = await request.get(`${FIXTURE_URL}/page-revision-diff-requests`)
      return (await response.json()).length
    })
    .toBe(2)
  const requests = await request
    .get(`${FIXTURE_URL}/page-revision-diff-requests`)
    .then((response) => response.json())
  expect(requests).toEqual([
    {
      site_id: 6000005,
      page_id: 3000345,
      from_revision_number: 1,
      to_revision_number: 2
    },
    {
      site_id: 6000005,
      page_id: 3000345,
      from_revision_number: 2,
      to_revision_number: 1
    }
  ])

  const diff = page.locator(".revision-diff")
  await expect(diff).toContainText("NEW CURRENT DIFF")

  const release = await request.get(
    `${FIXTURE_URL}/release-page-revision-diff?outcome=success`
  )
  expect(await release.json()).toEqual({ released: true })
  await expect(page.locator("#odialog-container")).toHaveCount(0)
  await expect(diff).toContainText("NEW CURRENT DIFF")
  await expect(diff).not.toContainText("OLD STALE DIFF")
})

test("WIKIDOT History View Version and View Source actions still load selected revisions", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-read-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.getByRole("link", { name: "history", exact: true }).click()

  const oldRevision = page.locator(
    '#action-area table.page-history tr[id="revision-row-9000341"]'
  )
  await oldRevision.locator("a[title='View source of the revision']").click()
  await expect(page.locator("#history-subarea div.page-source")).toHaveText(
    "Source for revision 1"
  )

  await oldRevision.locator("a[title='View page revision']").click()
  await expect(page.locator("#page-content")).toContainText("Historical version 1")
  await expect(page.locator("#history-subarea div.page-source")).toHaveCount(0)

  const requests = await request
    .get(`${FIXTURE_URL}/last-page-read-requests`)
    .then((response) => response.json())
  expect(requests.pageRevisionGet).toContainEqual(
    expect.objectContaining({
      page_id: 3000345,
      revision_number: 1,
      details: { compiled_html: false, wikitext: true }
    })
  )
  expect(requests.pageRevisionGet).toContainEqual(
    expect.objectContaining({
      page_id: 3000345,
      revision_number: 1,
      details: { compiled_html: true, wikitext: false }
    })
  )
})

test("WIKIDOT Page Source links only the same-site component include target", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/page-source-include-probe")
  await page.waitForLoadState("networkidle")
  await page.locator("#more-options-button").click()
  await page.locator("#view-source-button").click()

  const source = page.locator("div.page-source").first()
  expect(await source.textContent()).toBe(
    '[[include component:image-block\n|caption=fixture\n]]\n[[include :scp-wiki:component:license-box]]\n@@[[include component:literal]]@@\n<img src=x onerror="alert(1)">'
  )
  await expect(source.locator("a")).toHaveCount(1)
  const include = source.getByRole("link", { name: "component:image-block" })
  await expect(include).toHaveAttribute("href", "/component%3Aimage-block")
  await expect(source.locator("img")).toHaveCount(0)
  await expect(source.locator("a").nth(0)).toHaveText("component:image-block")

  await include.click()
  await expect(page).toHaveURL(/\/component%3Aimage-block$/u)
  await expect(page.locator("#page-content")).toContainText("Component target.")
})

test("history diff keeps added and removed source readable against its semantic colors", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-read-requests`)
  await page.setViewportSize({ width: 390, height: 844 })
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.getByRole("link", { name: "history", exact: true }).click()
  await page.locator("#revision-diff-from").selectOption("2")
  await page.locator("#revision-diff-to").selectOption("1")
  await page.locator(".revision-diff-controls button").nth(1).click()

  const added = page.locator("#action-area .revision-diff-line.added").first()
  const removed = page.locator("#action-area .revision-diff-line.removed").first()
  const unchanged = page.locator("#action-area .revision-diff-line.unchanged").first()
  await expect
    .poll(async () => {
      const response = await request.get(`${FIXTURE_URL}/page-revision-diff-requests`)
      return (await response.json()).length
    })
    .toBe(1)
  await expect(added).toBeVisible()
  await expect(removed).toBeVisible()
  await expect(unchanged).toBeVisible()
  await page.addStyleTag({
    content: `#action-area .revision-diff-line { white-space: pre !important; }`
  })
  await expect
    .poll(async () => added.evaluate((element) => getComputedStyle(element).whiteSpace))
    .toBe("pre-wrap")
  await expect
    .poll(async () =>
      added.evaluate((element) => {
        const parent = element.parentElement
        return parent ? parent.scrollWidth <= parent.clientWidth : false
      })
    )
    .toBe(true)
  await expect
    .poll(async () => added.evaluate((element) => getComputedStyle(element).color))
    .toBe("rgb(23, 52, 33)")
  await expect
    .poll(async () =>
      added.evaluate((element) => getComputedStyle(element).backgroundColor)
    )
    .toBe("rgb(220, 255, 220)")
  await expect
    .poll(async () => removed.evaluate((element) => getComputedStyle(element).color))
    .toBe("rgb(74, 32, 32)")
  await expect
    .poll(async () =>
      removed.evaluate((element) => getComputedStyle(element).backgroundColor)
    )
    .toBe("rgb(255, 225, 225)")
})

test("WIKIDOT History keeps the seven-cell source table contract at phone widths", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 })
    await page.goto("/authoring-history-probe")
    await waitForSvelteDelegatedHandler(page, "#history-button")
    await page.getByRole("link", { name: "history", exact: true }).click()
    const table = page.locator("#action-area table.page-history")
    await expect(table).toBeVisible()
    await expect(table.locator("tbody > tr").first().locator("td")).toHaveCount(7)

    const rows = table.locator('tbody > tr[id^="revision-row-"]')
    await expect(rows.first().locator("td")).toHaveCount(7)
    const currentRow = rows.first()
    const currentCells = currentRow.locator("td")
    const revisionId =
      (await currentRow.getAttribute("id"))?.replace("revision-row-", "") ?? ""
    await expect(
      currentCells.nth(1).locator('input[type="radio"][name="from"]')
    ).toBeVisible()
    await expect(
      currentCells.nth(1).locator('input[type="radio"][name="to"]')
    ).toBeVisible()
    await expect(currentCells.nth(1)).toHaveAttribute("style", "width: 5em")
    await expect(currentCells.nth(3)).toHaveAttribute("style", "width: 5em")
    await expect(currentCells.nth(4)).toHaveAttribute("style", "width: 15em")
    await expect(currentCells.nth(5)).toHaveAttribute(
      "style",
      "padding: 0 0.5em; width: 7em;"
    )
    await expect(currentCells.nth(6)).toHaveAttribute("style", "font-size: 90%")
    await expect(currentCells.nth(1).locator('input[type="radio"]')).toHaveCount(2)
    await expect(
      currentCells.nth(1).locator('input[type="radio"]').nth(0)
    ).toHaveAttribute("id", revisionId)
    await expect(
      currentCells.nth(1).locator('input[type="radio"]').nth(1)
    ).toHaveAttribute("id", revisionId)
    await expect(
      currentCells.nth(3).locator("a").filter({ hasText: /^V$/u })
    ).toBeVisible()
    await expect(
      currentCells.nth(3).locator("a").filter({ hasText: /^S$/u })
    ).toBeVisible()
    await expect(
      currentCells.nth(3).locator("a").filter({ hasText: /^R$/u })
    ).toHaveCount(0)
    await expect(currentCells.nth(4).locator(".printuser.avatarhover")).toBeVisible()
    await expect(
      currentCells.nth(4).locator(".printuser.avatarhover img.small")
    ).toHaveCount(1)
    await expect(currentCells.nth(4).locator(".printuser.avatarhover a")).toHaveCount(2)
    await expect(
      currentCells.nth(4).locator(".printuser.avatarhover a").nth(1)
    ).toHaveAttribute("href", "https://www.wikidot.com/user:info/history-author")
    await expect(currentCells.nth(5).locator(".odate")).toHaveCount(1)
    await expect(currentCells.nth(5).locator(".odate")).toHaveText("15 Aug 2026 00:00")

    const olderRow = rows.nth(1)
    await expect(olderRow.locator("td")).toHaveCount(7)
    await expect(
      olderRow.locator("td").nth(3).locator("a").filter({ hasText: /^R$/u })
    ).toBeVisible()
    await olderRow.locator('input[type="radio"][name="from"]').check()
    const selectedRevisionNumber = await olderRow
      .locator("td")
      .first()
      .innerText()
      .then((value) => Number.parseInt(value, 10) - 1)
    await expect(page.locator("#revision-diff-from")).toHaveValue(
      String(selectedRevisionNumber)
    )
  }
})

test("WIKIDOT History preserves source row selectors and functional actions across desktop and tablet widths", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  for (const width of [1440, 1024, 768]) {
    await page.setViewportSize({ width, height: 900 })
    await page.goto("/authoring-history-probe")
    await waitForSvelteDelegatedHandler(page, "#history-button")
    await page.locator("#history-button").click()
    const table = page.locator("#action-area table.page-history")
    const row = table.locator('tbody > tr[id^="revision-row-"]').first()
    await expect(row).toBeVisible()
    await expect(row).toHaveAttribute("id", /^revision-row-\d+$/u)
    await expect(row.locator("td")).toHaveCount(7)
    await expect(row.locator("td.optionstd[style*='width: 5em'] a")).toHaveCount(2)
    await expect(row.locator(".printuser.avatarhover")).toBeVisible()
    await expect(row.locator(".odate[class*='format_%25e']")).toHaveCount(1)
    await expect(row.locator(".odate")).toHaveText("15 Aug 2026 00:00")
    await expect(row.locator(".odate")).toBeVisible()
    await expect(row.locator('input[type="radio"][name="from"]')).toBeVisible()
    await expect(row.locator('input[type="radio"][name="to"]')).toBeVisible()
    const cells = await row
      .locator("td")
      .evaluateAll((items) => items.map((cell) => cell.textContent?.trim() ?? ""))
    expect(cells).toHaveLength(7)

    const revisionDates = table.locator('tbody > tr[id^="revision-row-"] .odate')
    const dateCount = await revisionDates.count()
    for (let dateIndex = 0; dateIndex < dateCount; dateIndex += 1) {
      await expect(revisionDates.nth(dateIndex)).toBeVisible()
    }
  }
})

test("WIKIDOT reveals populated odate spans after hydration and dynamic insertion", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/page-workflow-probe")
  await page.evaluate(() => {
    const content = document.querySelector("#page-content")
    if (!content) throw new Error("page content was not rendered")

    const populated = document.createElement("span")
    populated.className = "odate time_1785638315 format_%25e%20%25b%20%25Y"
    populated.textContent = "02 Aug 2026 02:38"
    populated.dataset.testid = "odate-populated"
    content.append(populated)

    const empty = document.createElement("span")
    empty.className = "odate time_1785638315 format_%25e%20%25b%20%25Y"
    empty.dataset.testid = "odate-empty"
    content.append(empty)
  })

  await expect(page.getByTestId("odate-populated")).toBeVisible()
  await expect(page.getByTestId("odate-empty")).not.toBeVisible()
})

test("mobile Files pane keeps long Japanese filenames and actions horizontally accessible", async ({
  page
}) => {
  const filename =
    "theme-lab-visual-fixture_日本語長名_320px_readability_and_download_controls.txt"
  await page.setViewportSize({ width: 390, height: 844 })
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/page-workflow-probe")
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()

  const list = page.locator(".file-list-scroll")
  await expect(list).toBeVisible()
  await expect(list).toContainText(filename)
  const geometry = await page.evaluate(() => ({
    viewport_width: window.innerWidth,
    document_width: document.documentElement.scrollWidth,
    list_width: document.querySelector<HTMLElement>(".file-list-scroll")?.clientWidth,
    table_width: document.querySelector<HTMLElement>(".file-list")?.scrollWidth,
    list_tab_index: document.querySelector(".file-list-scroll")?.getAttribute("tabindex")
  }))

  expect(geometry.document_width).toBeLessThanOrEqual(geometry.viewport_width)
  expect(geometry.table_width).toBeGreaterThan(geometry.list_width ?? 0)
  expect(geometry.list_tab_index).toBe("0")
  await page.setViewportSize({ width: 320, height: 800 })
  const narrowFilename = page.locator(".file-row .file-name a").last()
  const narrowGeometry = await narrowFilename.evaluate((anchor) => {
    const range = document.createRange()
    range.selectNodeContents(anchor)
    return {
      line_count: range.getClientRects().length,
      client_width: anchor.parentElement?.clientWidth ?? 0,
      overflow_width: anchor.scrollWidth
    }
  })
  expect(narrowGeometry.line_count).toBeGreaterThan(1)
  expect(narrowGeometry.overflow_width).toBeLessThanOrEqual(narrowGeometry.client_width)
  await list.evaluate((element) => {
    element.scrollLeft = element.scrollWidth
  })
  await expect(page.locator(".file-row a").last()).toBeVisible()
})

test("mobile edit-permission dialog keeps its message and close control in the viewport", async ({
  page
}) => {
  await page.setViewportSize({ width: 320, height: 800 })
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.locator("#more-options-button").click()
  await expect(page.locator("#view-source-button")).toBeVisible()
  await page.locator("#edit-button").click()

  const dialog = page.locator("#odialog-container .owindow.error")
  const close = page.locator("#odialog-container .button-close-message")
  await expect(dialog).toBeVisible()
  await expect(close).toBeVisible()

  const layout = await page.evaluate(() => {
    const modal = document.querySelector<HTMLElement>(
      "#odialog-container .owindow.error"
    )!
    const message = modal.querySelector<HTMLElement>("#modal-title")!
    const buttonBar = modal.querySelector<HTMLElement>(".button-bar")!
    const close = modal.querySelector<HTMLElement>(".button-close-message")!
    const rect = (element: HTMLElement) => {
      const { x, y, right, bottom, width, height } = element.getBoundingClientRect()
      return { x, y, right, bottom, width, height }
    }
    return {
      viewport_width: window.innerWidth,
      document_width: document.documentElement.scrollWidth,
      modal: rect(modal),
      message: rect(message),
      message_scroll_width: message.scrollWidth,
      message_client_width: message.clientWidth,
      button_bar: rect(buttonBar),
      button_bar_scroll_width: buttonBar.scrollWidth,
      close: rect(close)
    }
  })

  expect(layout.document_width).toBeLessThanOrEqual(layout.viewport_width)
  expect(layout.modal.x).toBeGreaterThanOrEqual(0)
  expect(layout.modal.right).toBeLessThanOrEqual(layout.viewport_width)
  expect(layout.message_scroll_width).toBeLessThanOrEqual(layout.message_client_width)
  expect(layout.button_bar_scroll_width).toBeLessThanOrEqual(layout.button_bar.width + 1)
  expect(layout.close.x).toBeGreaterThanOrEqual(layout.modal.x)
  expect(layout.close.right).toBeLessThanOrEqual(layout.modal.right)
  await close.click()
  await expect(dialog).toHaveCount(0)
})

test("history ignores a stale successful diff after page navigation", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-read-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.getByRole("link", { name: "history", exact: true }).click()
  await expect(page.locator(".revision-diff-controls")).toBeVisible()
  await page.locator(".revision-diff-controls button").nth(1).click()
  await expect
    .poll(async () => {
      const response = await request.get(`${FIXTURE_URL}/page-revision-diff-requests`)
      return (await response.json()).length
    })
    .toBe(1)

  await page.locator("#navigate-history-target").click()
  await expect(page).toHaveURL(/\/scp-173$/u)
  await expect(page.locator("#page-title")).toHaveText("SCP-173")
  await expect(page.locator(".revision-diff-controls")).toBeVisible()

  const release = await request.get(
    `${FIXTURE_URL}/release-page-revision-diff?outcome=success`
  )
  expect(await release.json()).toEqual({ released: true })
  await expect(page.locator(".revision-diff")).toHaveCount(0)
  await expect(page.locator("#odialog-container")).toHaveCount(0)
})

test("history ignores a stale failure after page navigation", async ({
  page,
  request
}) => {
  await request.get(`${FIXTURE_URL}/last-page-read-requests`)
  await page.setExtraHTTPHeaders(AUTHENTICATED_HEADERS)
  await page.goto("/authoring-history-probe")
  await waitForSvelteDelegatedHandler(page, "#history-button")
  await page.getByRole("link", { name: "history", exact: true }).click()
  await expect(page.locator(".revision-diff-controls")).toBeVisible()
  await page.locator(".revision-diff-controls button").nth(1).click()
  await expect
    .poll(async () => {
      const response = await request.get(`${FIXTURE_URL}/page-revision-diff-requests`)
      return (await response.json()).length
    })
    .toBe(1)

  await page.locator("#navigate-history-target").click()
  await expect(page).toHaveURL(/\/scp-173$/u)
  await expect(page.locator("#page-title")).toHaveText("SCP-173")
  await expect(page.locator(".revision-diff-controls")).toBeVisible()

  const release = await request.get(
    `${FIXTURE_URL}/release-page-revision-diff?outcome=failure`
  )
  expect(await release.json()).toEqual({ released: true })
  await expect(page.locator(".revision-diff")).toHaveCount(0)
  await expect(page.locator("#odialog-container")).toHaveCount(0)
})
