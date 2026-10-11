import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const appURL = (projectName: string) =>
  projectName === "webkit-https-edit-meta"
    ? `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`
    : `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

const EDIT_META_MODULE = "edit/EditMetaModule"

/**
 * The fixture Deepwell server does not implement the Edit Meta module, so
 * this per-test API fixture serves its legacy-shaped response from
 * isolated persisted rows. Action-bearing requests are recorded and update
 * only this disposable fixture state.
 */
const interceptEditMetaModule = async (
  page: import("@playwright/test").Page,
  baseURL: string
) => {
  const mutations: string[] = []
  const writeRequests: string[] = []
  const mutationPayloads: Array<{
    event: string | null
    pageId: string | null
    metaName: string | null
    metaContent: string | null
    allPages: string | null
  }> = []
  // This per-test fixture is the authoritative persisted state for the
  // intercepted module, and survives pane remounts and page reloads.
  const persistedRows = [
    { name: "fixture-existing", content: "unchanged-value", all_pages: false }
  ]
  const responseBody = () => `<h1>Meta tags for the page</h1>
<p>Using the interface below you can edit special HTML &lt;meta&gt; tags for the page.</p>
<h2>Current meta tags:</h2>
<div style="padding-left:3em;">${persistedRows
    .map(
      (row) =>
        `<div>remove &lt;meta name="${row.name}" content="${row.content}"/&gt;${row.all_pages ? " (all pages)" : ""}</div>`
    )
    .join("")}</div>
<p></p>`
  const appOrigin = new URL(baseURL).origin
  page.on("request", (request) => {
    if (request.method() === "GET" || request.method() === "HEAD") return
    const url = new URL(request.url())
    if (url.origin !== appOrigin) return

    const params = new URLSearchParams(request.postData() ?? "")
    const editMetaRead =
      url.pathname === "/ajax-module-connector.php" &&
      params.get("moduleName") === EDIT_META_MODULE &&
      !params.has("action") &&
      params.get("event") !== "saveMetaTag" &&
      params.get("event") !== "deleteMetaTag"
    if (!editMetaRead) writeRequests.push(`${request.method()} ${url.pathname}`)
  })
  await page.route("**/ajax-module-connector.php", async (route) => {
    const params = new URLSearchParams(route.request().postData() ?? "")
    if (params.get("moduleName") !== EDIT_META_MODULE) {
      await route.continue()
      return
    }
    const event = params.get("event")
    if (params.has("action") || event === "saveMetaTag" || event === "deleteMetaTag") {
      mutations.push(`${event}:${params.get("allPages") ?? "false"}`)
      mutationPayloads.push({
        event,
        pageId: params.get("pageId"),
        metaName: params.get("metaName"),
        metaContent: params.get("metaContent"),
        allPages: params.get("allPages")
      })
      const name = params.get("metaName")
      const allPages = params.get("allPages") === "true"
      if (event === "saveMetaTag" && name) {
        const existing = persistedRows.findIndex(
          (row) => row.name === name && row.all_pages === allPages
        )
        const row = {
          name,
          content: params.get("metaContent") ?? "",
          all_pages: allPages
        }
        if (existing === -1) persistedRows.push(row)
        else persistedRows[existing] = row
      } else if (event === "deleteMetaTag" && name) {
        const existing = persistedRows.findIndex(
          (row) => row.name === name && row.all_pages === allPages
        )
        if (existing !== -1) persistedRows.splice(existing, 1)
      }
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ status: "ok", body: responseBody() })
    })
  })
  return {
    mutations,
    mutationPayloads,
    writeRequests,
    persistedRows: () => structuredClone(persistedRows)
  }
}

const waitForWikidotHydration = (page: import("@playwright/test").Page) =>
  page.waitForFunction(() => {
    const pageContent = document.querySelector("#page-content")
    return (
      pageContent !== null &&
      pageContent.firstChild?.nodeType !== Node.COMMENT_NODE &&
      pageContent.lastChild?.nodeType !== Node.COMMENT_NODE
    )
  })

/** Opens Options -> Edit Meta and stops with the add button visible. */
const openEditMetaPaneFromOptions = async (page: import("@playwright/test").Page) => {
  const editMetaButton = page.locator("#edit-meta-button")
  if (!(await editMetaButton.isVisible())) {
    await waitForSvelteDelegatedHandler(page, "#more-options-button")
    await page.locator("#more-options-button").click()
  }
  await waitForSvelteDelegatedHandler(page, "#edit-meta-button")
  await editMetaButton.click()
  await expect(page.locator("#action-area #edit-meta-addbutton")).toBeVisible()
  await waitForSvelteDelegatedHandler(page, "#edit-meta-addbutton button")
}

const openEditMetaPane = async (
  page: import("@playwright/test").Page,
  baseURL: string
) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${baseURL}/scp-173`)
  await waitForWikidotHydration(page)
  await openEditMetaPaneFromOptions(page)
}

test.beforeEach(({ browserName }, testInfo) => {
  test.skip(
    browserName === "webkit" && testInfo.project.name === "webkit",
    "WebKit Edit Meta acceptance runs against the real-TLS browser-support fixture"
  )
})

const clickAddNewMetaTag = async (page: import("@playwright/test").Page) => {
  await page.locator("#edit-meta-addbutton button").click()
  await expect(page.locator("#edit-meta-newtag-form")).toBeVisible()
}

const measure = (page: import("@playwright/test").Page) =>
  page.evaluate(() => {
    const actionArea = document.querySelector<HTMLElement>("#action-area")
    const rect = actionArea?.getBoundingClientRect()
    return {
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      actionAreaClientWidth: actionArea?.clientWidth ?? 0,
      actionAreaLeft: rect?.x ?? 0,
      actionAreaRight: rect?.right ?? 0
    }
  })

const controlBoxes = (page: import("@playwright/test").Page) =>
  page.evaluate(() => {
    const form = document.querySelector<HTMLElement>("#edit-meta-newtag-form")
    if (!form) throw new Error("edit meta new tag form is missing")
    const box = (element: Element) => {
      const rect = element.getBoundingClientRect()
      return { x: rect.x, right: rect.right, width: rect.width }
    }
    const actionByText = (text: string) => {
      const action = [...form.querySelectorAll("button, a")].find((candidate) =>
        candidate.textContent?.includes(text)
      )
      if (!action) throw new Error(`missing action: ${text}`)
      return action
    }
    return {
      form: box(form),
      nameInput: box(form.querySelector('input[name="metaName"]')!),
      contentInput: box(form.querySelector('input[name="metaContent"]')!),
      cancel: box(form.querySelector("button.btn-danger")!),
      addAllPages: box(actionByText("Add to All Pages")),
      addThisPage: box(actionByText("Add to This Page"))
    }
  })

/**
 * Asserts the issue contract: opening the form must not enlarge the
 * document's horizontal overflow beyond the pre-form baseline, and every
 * control must stay inside the action-area pane (not merely inside the
 * current page scroll extent, which may itself carry an unrelated article
 * overflow).
 */
const expectFormBounded = async (
  page: import("@playwright/test").Page,
  before: Awaited<ReturnType<typeof measure>>,
  withForm: Awaited<ReturnType<typeof measure>>
) => {
  expect(
    withForm.scrollWidth,
    `form must not enlarge document overflow: ${JSON.stringify({ before, withForm })}`
  ).toBeLessThanOrEqual(before.scrollWidth)

  const boxes = await controlBoxes(page)
  for (const [key, value] of Object.entries(boxes)) {
    expect(
      value.x,
      `${key} left edge leaves the pane: ${JSON.stringify(value)}`
    ).toBeGreaterThanOrEqual(before.actionAreaLeft - 1)
    expect(
      value.right,
      `${key} right edge leaves the pane: ${JSON.stringify(value)}`
    ).toBeLessThanOrEqual(before.actionAreaRight + 1)
  }
  expect(
    boxes.form.width,
    `form must stay inside the pane: ${JSON.stringify(boxes.form)}`
  ).toBeLessThanOrEqual(before.actionAreaClientWidth + 1)
  return boxes
}

test("Edit Meta new-tag form stays bounded at 320px and sends no mutation on cancel", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests } = await interceptEditMetaModule(page, baseURL)
  await page.setViewportSize({ width: 320, height: 720 })
  await openEditMetaPane(page, baseURL)

  const before = await measure(page)
  await clickAddNewMetaTag(page)
  const withForm = await measure(page)
  await expectFormBounded(page, before, withForm)

  // Wikidot keeps both source actions as enabled, keyboard-focusable links
  // even when the fields are empty. Do not activate them here: empty-submit
  // behavior is not established, and this fixture must make no mutation.
  const addAllPages = page.getByRole("link", { name: "Add to All Pages" })
  const addThisPage = page.getByRole("link", { name: "Add to This Page" })
  await expect(addAllPages).toHaveAttribute("href", "javascript:;")
  await expect(addThisPage).toHaveAttribute("href", "javascript:;")
  await expect(addAllPages).toBeEnabled()
  await expect(addThisPage).toBeEnabled()
  await page.locator('input[name="metaContent"]').focus()
  await page.keyboard.press("Tab")
  await expect(page.locator("#edit-meta-newtag-form .btn-danger")).toBeFocused()
  await page.keyboard.press("Tab")
  await expect(addAllPages).toBeFocused()
  await page.keyboard.press("Tab")
  await expect(addThisPage).toBeFocused()
  expect(mutations, "checking blank-state focus must not submit").toEqual([])
  expect(writeRequests, "checking blank-state focus must not write").toEqual([])

  // Empty activation stays a local no-op until Wikidot's submission result is
  // evidenced. Keep the affordance clickable without weakening server checks.
  await addAllPages.click()
  await addThisPage.click()
  await expect(page.locator("#edit-meta-newtag-form")).toBeVisible()
  expect(mutations, "blank Add actions must not mutate persisted metadata").toEqual(
    []
  )
  expect(writeRequests, "blank Add actions must not issue a write request").toEqual(
    []
  )

  // Fields remain operable through the keyboard.
  await page.locator('input[name="metaName"]').focus()
  await expect(page.locator('input[name="metaName"]')).toBeFocused()
  await page.keyboard.type("description")
  await page.locator('input[name="metaContent"]').fill("boundary check")
  expect(await page.locator('input[name="metaName"]').inputValue()).toBe("description")
  expect(await page.locator('input[name="metaContent"]').inputValue()).toBe(
    "boundary check"
  )
  await expect(addAllPages).toBeEnabled()
  await expect(addThisPage).toBeEnabled()

  // Cancel discards both draft fields and sends no action-bearing request.
  await page.locator("#edit-meta-newtag-form button.btn-danger").click()
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  await expect(page.locator("#action-area #edit-meta-addbutton")).toBeVisible()
  await clickAddNewMetaTag(page)
  expect(await page.locator('input[name="metaName"]').inputValue()).toBe("")
  expect(await page.locator('input[name="metaContent"]').inputValue()).toBe("")
  expect(mutations, "cancel must not send an Edit Meta action").toEqual([])
  expect(writeRequests, "cancel must not issue any app write request").toEqual([])
})

test("Edit Meta Cancel preserves persisted meta rows", async ({ page }, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests, persistedRows } = await interceptEditMetaModule(
    page,
    baseURL
  )
  const beforeCancel = persistedRows()
  await openEditMetaPane(page, baseURL)

  const actionArea = page.locator("#action-area")
  await expect(actionArea).toContainText("fixture-existing")
  await expect(actionArea).toContainText("unchanged-value")
  await clickAddNewMetaTag(page)
  await page.locator('input[name="metaName"]').fill("wj-cancel-nonsaved")
  await page.locator('input[name="metaContent"]').fill("should-clear-unpersisted")

  await page.locator("#edit-meta-newtag-form button.btn-danger").click()
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  await clickAddNewMetaTag(page)
  await expect(actionArea).toContainText("fixture-existing")
  await expect(actionArea).toContainText("unchanged-value")
  expect(persistedRows()).toEqual(beforeCancel)
  expect(mutations, "Cancel must not send an Edit Meta action").toEqual([])
  expect(writeRequests, "Cancel must not send any app write request").toEqual([])
})

test("Edit Meta cancel discards drafts across pane close, pane switch, and reload", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests, persistedRows } = await interceptEditMetaModule(
    page,
    baseURL
  )
  const beforeNavigation = persistedRows()
  await openEditMetaPane(page, baseURL)

  const expectPersistedFixtureRow = async () => {
    await expect(page.locator("#action-area")).toContainText("fixture-existing")
    await expect(page.locator("#action-area")).toContainText("unchanged-value")
  }
  const expectBlankDraft = async () => {
    await clickAddNewMetaTag(page)
    expect(await page.locator('input[name="metaName"]').inputValue()).toBe("")
    expect(await page.locator('input[name="metaContent"]').inputValue()).toBe("")
  }
  const fillDraft = async (name: string) => {
    await page.locator('input[name="metaName"]').fill(name)
    await page.locator('input[name="metaContent"]').fill("must-not-return")
  }

  await expectPersistedFixtureRow()
  await expectBlankDraft()
  await fillDraft("pane-close-draft")
  await page.locator("#action-area .action-area-close").click()
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await openEditMetaPaneFromOptions(page)
  await expectPersistedFixtureRow()
  await expectBlankDraft()

  await fillDraft("pane-switch-draft")
  // View Source is the pane switch here. It replaces the Edit Meta pane without
  // a request, while the Append form is permission-gated for this anonymous actor (#2177).
  await waitForSvelteDelegatedHandler(page, "#view-source-button")
  await page.locator("#view-source-button").click()
  await expect(page.locator("#action-area .page-source-header")).toBeVisible()
  await openEditMetaPaneFromOptions(page)
  await expectPersistedFixtureRow()
  await expectBlankDraft()

  await fillDraft("reload-draft")
  await page.reload()
  await waitForWikidotHydration(page)
  await expect(page.locator("#action-area")).toHaveClass(/hidden/u)
  await openEditMetaPaneFromOptions(page)
  await expectPersistedFixtureRow()
  await expectBlankDraft()
  expect(persistedRows()).toEqual(beforeNavigation)
  expect(mutations, "discarding drafts must not issue any Edit Meta action").toEqual([])
  expect(writeRequests, "discarding drafts must not issue any app write request").toEqual(
    []
  )
})

test("Edit Meta new-tag form stays bounded at 375px", async ({ page }, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests } = await interceptEditMetaModule(page, baseURL)
  await page.setViewportSize({ width: 375, height: 812 })
  await openEditMetaPane(page, baseURL)

  const before = await measure(page)
  await clickAddNewMetaTag(page)
  const withForm = await measure(page)
  await expectFormBounded(page, before, withForm)

  await page.locator("#edit-meta-newtag-form button.btn-danger").click()
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  expect(mutations).toEqual([])
  expect(writeRequests).toEqual([])
})

test("Edit Meta new-tag form stays usable on desktop", async ({ page }, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests } = await interceptEditMetaModule(page, baseURL)
  await page.setViewportSize({ width: 1280, height: 800 })
  await openEditMetaPane(page, baseURL)

  const before = await measure(page)
  await clickAddNewMetaTag(page)
  const withForm = await measure(page)
  const boxes = await expectFormBounded(page, before, withForm)
  expect(boxes.form.width).toBeGreaterThan(0)

  await page.locator("#edit-meta-newtag-form button.btn-danger").click()
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  expect(mutations).toEqual([])
  expect(writeRequests).toEqual([])
})

test("Add to This Page keeps the saveMetaTag wire contract without widening the layout", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, mutationPayloads, persistedRows } = await interceptEditMetaModule(
    page,
    baseURL
  )
  await page.setViewportSize({ width: 320, height: 720 })
  await openEditMetaPane(page, baseURL)

  const before = await measure(page)
  await clickAddNewMetaTag(page)
  const withForm = await measure(page)
  await expectFormBounded(page, before, withForm)

  await page.locator('input[name="metaName"]').fill("description")
  await page.getByRole("link", { name: "Add to This Page" }).click()
  // The intercepted save resolves, the pane reloads, and the form closes.
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  await expect(page.locator("#action-area #edit-meta-addbutton")).toBeVisible()
  expect(mutations).toEqual(["saveMetaTag:false"])
  expect(mutationPayloads).toEqual([
    {
      event: "saveMetaTag",
      pageId: expect.stringMatching(/^\d+$/u),
      metaName: "description",
      metaContent: "",
      allPages: null
    }
  ])
  expect(persistedRows()).toContainEqual({
    name: "description",
    content: "",
    all_pages: false
  })
})

test("Add to All Pages sends the distinct site-wide metadata payload", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, mutationPayloads, persistedRows } = await interceptEditMetaModule(
    page,
    baseURL
  )
  await openEditMetaPane(page, baseURL)
  await clickAddNewMetaTag(page)
  await page.locator('input[name="metaName"]').fill("robots")
  await page.locator('input[name="metaContent"]').fill("noindex")
  await page.locator('button:has-text("Add to All Pages")').click()

  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  expect(mutations).toEqual(["saveMetaTag:true"])
  expect(mutationPayloads).toEqual([
    {
      event: "saveMetaTag",
      pageId: expect.stringMatching(/^\d+$/u),
      metaName: "robots",
      metaContent: "noindex",
      allPages: "true"
    }
  ])
  expect(persistedRows()).toContainEqual({
    name: "robots",
    content: "noindex",
    all_pages: true
  })
})
