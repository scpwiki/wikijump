import { expect, test } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("ListPages pagination survives direct loads, reload, and browser history", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  await page.goto("/listpages-navigation/p/2?q=1#fragment")
  await expect(page).toHaveURL(/\/listpages-navigation\/p\/2\?q=1#fragment$/u)
  await expect(page.locator("#listpages-route")).toHaveText("p/2")
  await expect(page.locator("#listpages-page-one")).toHaveAttribute(
    "href",
    "/listpages-navigation/p/1"
  )

  await page.reload()
  await expect(page).toHaveURL(/\/listpages-navigation\/p\/2\?q=1#fragment$/u)
  await expect(page.locator("#listpages-route")).toHaveText("p/2")

  await page.locator("#listpages-page-one").click()
  await expect(page).toHaveURL(/\/listpages-navigation\/p\/1$/u)
  await expect(page.locator("#listpages-route")).toHaveText("p/1")

  await page.goBack()
  await expect(page).toHaveURL(/\/listpages-navigation\/p\/2\?q=1#fragment$/u)
  await expect(page.locator("#listpages-route")).toHaveText("p/2")

  await page.goForward()
  await expect(page).toHaveURL(/\/listpages-navigation\/p\/1$/u)
  await expect(page.locator("#listpages-route")).toHaveText("p/1")

  await page.goto("/listpages-navigation?q=1")
  await expect(page).toHaveURL(/\/listpages-navigation\?q=1$/u)
  await expect(page.locator("#listpages-route")).toHaveText("root")
  await expect(page.locator("#listpages-page-one")).toHaveAttribute(
    "href",
    "/listpages-navigation/p/1"
  )
})
