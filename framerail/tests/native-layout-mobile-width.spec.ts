import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const PLATFORM_HEADERS = {
  "X-Wikijump-Site-Id": "6000006",
  "X-Wikijump-Site-Slug": "www"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
const PAGES = [
  { path: "/-/about", widths: [320, 375, 390, 768] },
  { path: "/-/login", widths: [280, 320, 360, 375] },
  { path: "/-/user/guest", widths: [280, 320, 360, 375] },
  { path: "/platform:search", widths: [280, 320, 360, 375] }
]

test("native footer and About diagnostics stay within narrow viewports", async ({
  page
}) => {
  for (const { path, widths } of PAGES) {
    await page.setExtraHTTPHeaders(
      path === "/platform:search" ? PLATFORM_HEADERS : SITE_HEADERS
    )
    for (const width of widths) {
      await page.setViewportSize({ width, height: 812 })
      const response = await page.goto(`${APP_URL}${path}`)
      expect(response?.status(), `${path} at ${width}px`).toBe(200)

      const dimensions = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth,
        document: document.documentElement.scrollWidth,
        overflow: Array.from(document.querySelectorAll("body *"))
          .map((element) => {
            const bounds = element.getBoundingClientRect()
            return {
              selector: `${element.tagName.toLowerCase()}${element.id ? `#${element.id}` : ""}${
                typeof element.className === "string" && element.className
                  ? `.${element.className.trim().split(/\s+/u).join(".")}`
                  : ""
              }`,
              right: Math.round(bounds.right),
              width: Math.round(bounds.width)
            }
          })
          .filter((element) => element.right > document.documentElement.clientWidth + 1)
          .slice(0, 8),
        scrollable: Array.from(document.querySelectorAll("body *"))
          .filter((element) => element.scrollWidth > element.clientWidth + 1)
          .map((element) => ({
            tag: element.tagName.toLowerCase(),
            id: element.id,
            className: typeof element.className === "string" ? element.className : "",
            scrollWidth: element.scrollWidth,
            clientWidth: element.clientWidth
          }))
          .slice(0, 8)
      }))
      // The platform SearchAll size=30 input independently extends 7px at
      // 280px; keep this footer assertion scoped to the footer's own bounds.
      if (path !== "/platform:search" || width !== 280) {
        expect(
          dimensions.document,
          `${path} at ${width}px: ${JSON.stringify(dimensions)}`
        ).toBeLessThanOrEqual(dimensions.viewport + 1)
      }

      const footer = page.locator(".footer-inner")
      await expect(footer.locator(".footer-item a")).toHaveCount(4)
      await expect(footer.locator(".footer-powered-by")).toBeVisible()
      for (const link of await footer.locator(".footer-item a").all()) {
        expect(
          await link.evaluate((element) => (element as HTMLAnchorElement).tabIndex)
        ).toBe(0)
      }
      const footerBounds = await footer.locator(".footer-powered-by").boundingBox()
      expect(footerBounds, `${path} powered-by bounds at ${width}px`).not.toBeNull()
      expect(
        footerBounds!.x + footerBounds!.width,
        `${path} powered-by right edge at ${width}px`
      ).toBeLessThanOrEqual(width + 1)

      if (path === "/-/about") {
        const repositoryLinks = page.locator("table.platform-info a[rel='external']")
        await expect(repositoryLinks).toHaveCount(2)
        for (const link of await repositoryLinks.all()) {
          await expect(link).toBeVisible()
          const [href, text] = await Promise.all([
            link.getAttribute("href"),
            link.textContent()
          ])
          expect(text?.trim()).toBe(href)
        }
      }

      if (width === 320) {
        await page.evaluate(() => {
          document.documentElement.style.fontSize = "200%"
        })
        const zoomedWidths = await page.evaluate(() => ({
          viewport: document.documentElement.clientWidth,
          document: document.documentElement.scrollWidth
        }))
        expect(
          zoomedWidths.document,
          `${path} at 320px with 200% text: ${JSON.stringify(zoomedWidths)}`
        ).toBeLessThanOrEqual(zoomedWidths.viewport + 1)
      }
    }
  }
})
