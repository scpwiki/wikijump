import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("Files Delete waits for an accessible file-specific confirmation", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/page-workflow-probe")
  await waitForSvelteDelegatedHandler(page, "#files-button")
  await page.locator("#files-button").click()

  const row = page.locator(".file-row").first()
  await expect(row).toBeVisible()
  const filename = (await row.locator(".file-name").innerText()).trim()
  const deleteAction = row.locator(".delete-file")
  const deleteRequests: import("@playwright/test").Request[] = []
  page.on("request", (request) => {
    if (new URL(request.url()).search === "?/fileDelete") deleteRequests.push(request)
  })

  await deleteAction.click()
  const dialog = page.getByRole("dialog")
  await expect(dialog).toBeVisible()
  await expect(dialog).toHaveAccessibleName("Delete this file?")
  await expect(dialog).toContainText(filename)
  await expect(dialog).toContainText("page-workflow-probe")
  const cancel = page.getByRole("button", { name: "Cancel" })
  const confirm = page.getByRole("button", { name: "Confirm delete" })
  await expect(cancel).toBeFocused()
  await expect(confirm).toBeVisible()
  expect(deleteRequests).toHaveLength(0)

  await page.keyboard.press("Escape")
  await expect(dialog).toBeHidden()
  await expect(deleteAction).toBeFocused()
  expect(deleteRequests).toHaveLength(0)

  await deleteAction.focus()
  await page.keyboard.press("Enter")
  await expect(dialog).toBeVisible()
  await expect(cancel).toBeFocused()
  expect(deleteRequests).toHaveLength(0)
  await page.keyboard.press("Escape")
  await expect(dialog).toBeHidden()
  await expect(deleteAction).toBeFocused()

  await deleteAction.click()
  await expect(dialog).toBeVisible()
  await cancel.click()
  await expect(dialog).toBeHidden()
  await expect(deleteAction).toBeFocused()
  expect(deleteRequests).toHaveLength(0)

  const confirmedRequest = page.waitForRequest(
    (request) => new URL(request.url()).search === "?/fileDelete"
  )
  await deleteAction.click()
  await expect(dialog).toBeVisible()
  await confirm.click()
  const request = await confirmedRequest
  expect(request.method()).toBe("POST")
  const payload = JSON.parse(request.postData() ?? "{}") as Record<string, unknown>
  expect(payload).toMatchObject({
    siteId: 6000005,
    pageId: 3000340
  })
  expect(payload.fileId).toBe(Number(await row.getAttribute("data-id")))
  expect(Number(payload.lastRevisionId)).toBeGreaterThan(0)
  const response = await request.response()
  expect(response).not.toBeNull()
  expect(await response?.json()).toMatchObject({
    type: "failure",
    status: 401
  })
  await expect(row).toBeVisible()
  expect(deleteRequests).toHaveLength(1)
})
