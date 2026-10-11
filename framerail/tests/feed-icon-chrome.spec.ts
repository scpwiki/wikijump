import crypto from "node:crypto"

import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://127.0.0.1:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const ICON_PATH = "/common--theme/base/images/feed/feed-icon-14x14.png"
const ICON_SHA256 = "332f1155f4635f7d79797ac40141990c62a140921feb68bc3d3d7a1bfa7ab8c6"

test("ForumStart RSS icon loads from the local origin with nonzero natural size", async ({
  page,
  request
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const wikidotRequests: string[] = []
  page.on("request", (request) => {
    if (new URL(request.url()).hostname.endsWith("wikidot.com")) {
      wikidotRequests.push(request.url())
    }
  })

  await page.goto(`${APP_URL}/forum/start`)
  const icon = page.locator('img[alt="rss icon"]')
  await expect(icon).toHaveCount(1)
  await expect(icon).toHaveAttribute("src", ICON_PATH)
  await expect
    .poll(() => icon.evaluate((image: HTMLImageElement) => image.naturalWidth))
    .toBe(14)
  expect(await icon.evaluate((image: HTMLImageElement) => image.naturalHeight)).toBe(14)
  expect(await icon.evaluate((image: HTMLImageElement) => image.currentSrc)).toBe(
    `${APP_URL}${ICON_PATH}`
  )
  expect(wikidotRequests).toEqual([])

  const response = await request.get(`${APP_URL}${ICON_PATH}`, {
    headers: SITE_HEADERS
  })
  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toBe("image/png")
  expect(response.headers()["cache-control"]).toBe("public, max-age=86400")
  expect(
    crypto
      .createHash("sha256")
      .update(await response.body())
      .digest("hex")
  ).toBe(ICON_SHA256)
})

test("unknown common--theme paths fall through to the app instead of proxying Wikidot", async ({
  request
}) => {
  const response = await request.get(
    `${APP_URL}/common--theme/base/images/feed/not-an-icon.png`,
    { headers: SITE_HEADERS }
  )
  // The hermetic fixture has no article for this slug, so the app fails closed
  // (500) through its article route. The point is that the path is neither
  // served as an icon nor proxied: no 2xx image response.
  expect(response.ok()).toBe(false)
  expect(response.headers()["content-type"] ?? "").not.toMatch(/^image\//u)
})
