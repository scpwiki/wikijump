import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("imported Discuss uses the Wikidot request without an inline handler", async ({
  page
}) => {
  const requests: { url: string; body: string | null }[] = []
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.route("**/ajax-module-connector.php", async (route) => {
    requests.push({
      url: new URL(route.request().url()).pathname,
      body: route.request().postData()
    })
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        status: "no_page",
        message: "The page does not exist",
        callbackIndex: null,
        CURRENT_TIMESTAMP: 1794528000
      })
    })
  })

  const response = await page.goto("/wikidot-discussion-probe")
  expect(response?.status()).toBe(200)

  const discuss = page.locator("#discuss-button")
  await expect(discuss).toHaveText("Discuss (2)")
  await expect(discuss).not.toHaveAttribute("onclick")
  await waitForSvelteDelegatedHandler(page, "#discuss-button")
  await discuss.click()

  await expect(page.locator("#odialog-container")).toBeVisible()
  await expect(page.locator("#odialog-container")).toContainText(
    "The page does not exist"
  )
  expect(requests).toHaveLength(1)
  expect(requests[0].url).toBe("/ajax-module-connector.php")
  expect(new URLSearchParams(requests[0].body ?? "")).toEqual(
    new URLSearchParams({
      moduleName: "Empty",
      page_id: "3000396",
      action: "ForumAction",
      event: "createPageDiscussionThread"
    })
  )
})

test("successful Wikidot discussion responses open the returned thread path", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.route("**/ajax-module-connector.php", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        status: "ok",
        thread_id: 18232631,
        thread_unix_title: "lane-4-discussion"
      })
    })
  })
  await page.route("**/forum/t-18232631/lane-4-discussion", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "text/html",
      body: "<title>discussion route fixture</title>"
    })
  })

  const response = await page.goto("/wikidot-discussion-probe")
  expect(response?.status()).toBe(200)
  await waitForSvelteDelegatedHandler(page, "#discuss-button")

  const destination = page.waitForURL("**/forum/t-18232631/lane-4-discussion")
  await page.locator("#discuss-button").click()
  await destination
  expect(new URL(page.url()).pathname).toBe("/forum/t-18232631/lane-4-discussion")
})
