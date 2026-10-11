import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki",
  Cookie: "wikijump_token=fixture-session-token"
}

test("Parents keeps a draft typed before the initial parentGet response", async ({
  page
}) => {
  const baseURL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
  let resolveResponseReady!: (status: number) => void
  const responseReady = new Promise<number>((resolve) => {
    resolveResponseReady = resolve
  })
  let releaseResponse!: () => void
  const responseRelease = new Promise<void>((resolve) => {
    releaseResponse = resolve
  })
  let resolveResponseDelivered!: () => void
  const responseDelivered = new Promise<void>((resolve) => {
    resolveResponseDelivered = resolve
  })
  let parentSetRequests = 0

  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.route("**/*", async (route) => {
    const request = route.request()
    const url = new URL(request.url())
    if (
      request.method() === "POST" &&
      url.pathname === "/scp-173" &&
      url.search === "?/parentGet"
    ) {
      const response = await route.fetch()
      resolveResponseReady(response.status())
      await responseRelease
      await route.fulfill({ response })
      resolveResponseDelivered()
      return
    }
    if (
      request.method() === "POST" &&
      url.pathname === "/scp-173" &&
      url.search === "?/parentSet"
    ) {
      parentSetRequests += 1
      await route.abort("blockedbyclient")
      return
    }
    await route.continue()
  })

  await page.goto(`${baseURL}/scp-173`)
  await expect(page.locator("#page-content")).toBeVisible()
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  const parentsButton = page.locator("#parent-page-button")
  await expect(parentsButton).toBeVisible()
  await waitForSvelteDelegatedHandler(page, "#parent-page-button")
  await parentsButton.click()

  const input = page.locator("#page-parent input[type=text]")
  await expect(input).toBeVisible()
  expect(await responseReady, "the real local parentGet response should succeed").toBe(
    200
  )

  const saveButton = page.locator("#page-parent [type=submit]")
  await expect(saveButton).toBeDisabled()
  await input.fill("run-owned:parent-race-unsubmitted")
  await expect(input).toHaveValue("run-owned:parent-race-unsubmitted")
  releaseResponse()
  await responseDelivered

  await expect(input).toHaveValue("run-owned:parent-race-unsubmitted")
  await expect(saveButton).toBeEnabled()
  expect(parentSetRequests, "the regression probe must not submit a mutation").toBe(0)
})
