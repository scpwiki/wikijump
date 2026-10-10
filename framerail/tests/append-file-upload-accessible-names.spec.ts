import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const FIXTURE_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`
const appURL = (projectName: string) =>
  projectName === "webkit-https-edit-meta"
    ? `https://localhost:${process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"}`
    : `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

const waitForPageActionsHydration = (page: import("@playwright/test").Page) =>
  page.waitForFunction(() => {
    const pageContent = document.querySelector("#page-content")
    return (
      pageContent !== null &&
      pageContent.firstChild?.nodeType !== Node.COMMENT_NODE &&
      pageContent.lastChild?.nodeType !== Node.COMMENT_NODE
    )
  })

test("Append and Files Upload fields expose their associated names", async ({
  page,
  request
}, testInfo) => {
  await page.setViewportSize({ width: 320, height: 812 })
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await request.get(`${FIXTURE_URL}/last-file-requests`)
  const response = await page.goto(`${appURL(testInfo.project.name)}/page-workflow-probe`)
  expect(response?.status()).toBe(200)
  await waitForPageActionsHydration(page)

  await expect(page.locator("#more-options-button")).toBeVisible()
  await page.locator("#more-options-button").click()
  await expect(page.locator("#page-options-bottom-2")).toBeVisible()
  await expect(page.locator("#edit-append-button")).toBeVisible()
  await page.locator("#edit-append-button").click()

  const append = page.locator("#page-append-input")
  await expect(append).toBeVisible()
  await expect(page.getByRole("textbox", { name: "Wikitext to append" })).toHaveCount(1)
  expect(
    await append.evaluate((element: HTMLTextAreaElement) =>
      Array.from(element.labels ?? []).some((label) => label.htmlFor === element.id)
    )
  ).toBe(true)
  await page.locator('label[for="page-append-input"]').click()
  await expect(append).toBeFocused()

  const appendGeometry = await append.boundingBox()
  expect(appendGeometry).not.toBeNull()
  expect(appendGeometry!.x + appendGeometry!.width).toBeLessThanOrEqual(320)

  await page.locator('#page-append input[type="button"]').click()
  await expect(page.locator("#edit-append-button")).toBeVisible()
  await page.locator("#edit-append-button").click()
  await expect(page.getByRole("textbox", { name: "Wikitext to append" })).toHaveCount(1)
  await page.locator('#page-append input[type="button"]').click()

  await expect(page.locator("#files-button")).toBeVisible()
  await page.locator("#files-button").click()
  await page.getByRole("button", { name: /upload/u }).click()

  const file = page.locator("#file-upload-file-input")
  const filename = page.locator("#file-upload-name-input")
  await expect(file).toBeVisible()
  await expect(filename).toBeVisible()
  const [fileLabel, filenameLabel] = await Promise.all([
    file.evaluate((element: HTMLInputElement) =>
      element.labels?.[0]?.textContent?.trim()
    ),
    filename.evaluate((element: HTMLInputElement) =>
      element.labels?.[0]?.textContent?.trim()
    )
  ])
  expect(fileLabel).toBeTruthy()
  expect(filenameLabel).toBeTruthy()
  expect(fileLabel).not.toBe(filenameLabel)
  await expect(file).toHaveAccessibleName(fileLabel!)
  await expect(filename).toHaveAccessibleName(filenameLabel!)
  expect(
    await file.evaluate(
      (element: HTMLInputElement) => element.labels?.[0]?.control === element
    )
  ).toBe(true)
  expect(
    await filename.evaluate(
      (element: HTMLInputElement) => element.labels?.[0]?.control === element
    )
  ).toBe(true)
  await page.locator('label[for="file-upload-name-input"]').click()
  await expect(filename).toBeFocused()

  const fileChooserPromise = page.waitForEvent("filechooser")
  await page.locator('label[for="file-upload-file-input"]').click()
  const fileChooser = await fileChooserPromise
  expect(await fileChooser.element().getAttribute("id")).toBe("file-upload-file-input")

  const uploadGeometry = await Promise.all([file.boundingBox(), filename.boundingBox()])
  for (const [name, geometry] of [
    ["file", uploadGeometry[0]],
    ["filename", uploadGeometry[1]]
  ] as const) {
    expect(geometry).not.toBeNull()
    expect(
      geometry!.x + geometry!.width,
      `${name} control geometry: ${JSON.stringify(geometry)}`
    ).toBeLessThanOrEqual(320)
  }

  expect(
    await page
      .locator("#file-upload [id]")
      .evaluateAll((elements) => elements.map((element) => element.id))
  ).toEqual(["file-upload-file-input", "file-upload-name-input"])

  await page.locator('#file-upload input[type="button"]').click()
  const pageWrites = await (
    await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  ).json()
  const fileWrites = await (await request.get(`${FIXTURE_URL}/last-file-requests`)).json()
  expect(pageWrites.pageEdit).toEqual([])
  expect(fileWrites.blobUpload).toEqual([])
  expect(fileWrites.fileCreate).toEqual([])
})

test("authorized Append and Upload round-trip only through the isolated fixture actor", async ({
  page,
  context,
  request
}, testInfo) => {
  const baseURL = appURL(testInfo.project.name)
  const uploadName = `issue-2259-fixture-roundtrip-${testInfo.project.name}.txt`
  await context.addCookies([
    {
      name: "wikijump_token",
      value: "fixture-session-token",
      url: baseURL
    }
  ])
  await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  await request.get(`${FIXTURE_URL}/last-file-requests`)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const response = await page.goto(`${baseURL}/page-workflow-probe`)
  expect(response?.status()).toBe(200)
  await waitForPageActionsHydration(page)

  await page.locator("#more-options-button").click()
  await page.locator("#edit-append-button").click()
  await page.locator("#page-append-input").fill("run-owned issue 2259 append")
  await page.locator('#page-append input[type="submit"]').click()
  await expect(page.locator("#page-append")).toHaveCount(0)

  const pageWrites = await (
    await request.get(`${FIXTURE_URL}/last-page-write-requests`)
  ).json()
  expect(pageWrites.pageEdit).toHaveLength(1)
  expect(pageWrites.pageEdit[0].params.user_id).toBe(123)
  expect(pageWrites.pageEdit[0].params.page).toBe(3000340)
  expect(pageWrites.pageEdit[0].params.wikitext).toContain("run-owned issue 2259 append")

  await page.goto(`${baseURL}/page-workflow-probe`)
  await waitForPageActionsHydration(page)
  await page.locator("#files-button").click()
  await page.getByRole("button", { name: /upload/u }).click()
  await page.locator("#file-upload-file-input").setInputFiles({
    name: uploadName,
    mimeType: "text/plain",
    buffer: Buffer.from("run-owned issue 2259 upload")
  })
  await page.locator("#file-upload-name-input").fill(uploadName)
  await page.locator('#file-upload input[type="submit"]').click()
  await expect(page.locator("#file-upload")).toHaveCount(0)

  const fileWrites = await (await request.get(`${FIXTURE_URL}/last-file-requests`)).json()
  expect(fileWrites.blobUpload).toHaveLength(1)
  expect(fileWrites.blobUpload[0].params.user_id).toBe(123)
  expect(fileWrites.fileCreate).toHaveLength(1)
  expect(fileWrites.fileCreate[0].params.user_id).toBe(123)
  expect(fileWrites.fileCreate[0].params.name).toBe(uploadName)
})
