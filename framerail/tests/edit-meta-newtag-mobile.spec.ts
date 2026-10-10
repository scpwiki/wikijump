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
 * Legacy-shaped Edit Meta response with one persisted row. Cancellation
 * coverage verifies that the row survives remounts without a write
 * request.
 */
const FIXTURE_EDIT_META_BODY = `<h1>Meta tags for the page</h1>
<p>Using the interface below you can edit special HTML &lt;meta&gt; tags for the page.</p>
<h2>Current meta tags:</h2>
<div style="padding-left:3em;"><div>remove &lt;meta name="fixture-existing" content="unchanged-value"/&gt;</div></div>
<p></p>`

/**
 * The fixture Deepwell server does not implement the Edit Meta module, so
 * the pane's requests are answered with a canned legacy body. Every
 * intercepted action-bearing request is recorded so the spec can prove
 * cancellation makes no write request without any server mutation.
 */
const interceptEditMetaModule = async (
  page: import("@playwright/test").Page,
  baseURL: string
) => {
  const mutations: string[] = []
  const writeRequests: string[] = []
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
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ status: "ok", body: FIXTURE_EDIT_META_BODY })
    })
  })
  return { mutations, writeRequests }
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
    const buttonByText = (text: string) => {
      const button = [...form.querySelectorAll("button")].find((candidate) =>
        candidate.textContent?.includes(text)
      )
      if (!button) throw new Error(`missing button: ${text}`)
      return button
    }
    return {
      form: box(form),
      nameInput: box(form.querySelector('input[name="metaName"]')!),
      contentInput: box(form.querySelector('input[name="metaContent"]')!),
      cancel: box(form.querySelector("button.btn-danger")!),
      addAllPages: box(buttonByText("Add to All Pages")),
      addThisPage: box(buttonByText("Add to This Page"))
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

  // Fields remain operable through the keyboard.
  await page.locator('input[name="metaName"]').focus()
  await expect(page.locator('input[name="metaName"]')).toBeFocused()
  await page.keyboard.type("description")
  await page.locator('input[name="metaContent"]').fill("boundary check")
  expect(await page.locator('input[name="metaName"]').inputValue()).toBe("description")
  expect(await page.locator('input[name="metaContent"]').inputValue()).toBe(
    "boundary check"
  )
  await expect(page.locator('button:has-text("Add to All Pages")')).toBeEnabled()
  await expect(page.locator('button:has-text("Add to This Page")')).toBeEnabled()

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

test("Edit Meta Cancel resets the draft without changing persisted rows", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests } = await interceptEditMetaModule(page, baseURL)
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
  await expect(page.locator('input[name="metaName"]')).toHaveValue("")
  await expect(page.locator('input[name="metaContent"]')).toHaveValue("")
  await expect(actionArea).toContainText("fixture-existing")
  await expect(actionArea).toContainText("unchanged-value")
  expect(mutations, "Cancel must not send an Edit Meta action").toEqual([])
  expect(writeRequests, "Cancel must not send any app write request").toEqual([])
})

test("Edit Meta cancel discards drafts across pane close, pane switch, and reload", async ({
  page
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const { mutations, writeRequests } = await interceptEditMetaModule(page, baseURL)
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
  const appendButton = page.locator("#edit-append-button")
  if (!(await appendButton.isVisible())) {
    await waitForSvelteDelegatedHandler(page, "#more-options-button")
    await page.locator("#more-options-button").click()
  }
  await expect(appendButton).toBeVisible()
  await waitForSvelteDelegatedHandler(page, "#edit-append-button")
  await page.locator("#edit-append-button").click()
  await expect(page.locator("#action-area .page-append-header")).toBeVisible()
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
  const { mutations } = await interceptEditMetaModule(page, baseURL)
  await page.setViewportSize({ width: 320, height: 720 })
  await openEditMetaPane(page, baseURL)

  const before = await measure(page)
  await clickAddNewMetaTag(page)
  const withForm = await measure(page)
  await expectFormBounded(page, before, withForm)

  await page.locator('input[name="metaName"]').fill("description")
  await page.locator('button:has-text("Add to This Page")').click()
  // The intercepted save resolves, the pane reloads, and the form closes.
  await expect(page.locator("#edit-meta-newtag-form")).toHaveCount(0)
  await expect(page.locator("#action-area #edit-meta-addbutton")).toBeVisible()
  expect(mutations).toEqual(["saveMetaTag:false"])
})
