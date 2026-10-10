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
  { path: "/-/login", headers: SITE_HEADERS },
  { path: "/-/user/guest", headers: SITE_HEADERS },
  { path: "/platform:search", headers: PLATFORM_HEADERS }
]
const WIDTHS = [280, 320, 360, 375]

test("native footer stays reachable across mobile widths and browser engines", async ({
  page
}, testInfo) => {
  test.setTimeout(120_000)

  for (const { path, headers } of PAGES) {
    await page.setExtraHTTPHeaders(headers)

    for (const width of WIDTHS) {
      await page.setViewportSize({ width, height: 812 })
      const response = await page.goto(`${APP_URL}${path}`)
      expect(response?.status(), `${path} at ${width}px`).toBe(200)

      const footer = page.locator(".footer-inner")
      const links = footer.locator(".footer-item a")
      await expect(links).toHaveCount(4)
      await expect(footer.locator(".footer-powered-by")).toBeVisible()

      const dimensions = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth,
        document: document.documentElement.scrollWidth,
        overflow: Array.from(document.querySelectorAll("body *"))
          .map((element) => {
            const bounds = element.getBoundingClientRect()
            return {
              tag: element.tagName.toLowerCase(),
              id: element.id,
              className: typeof element.className === "string" ? element.className : "",
              right: Math.round(bounds.right),
              width: Math.round(bounds.width)
            }
          })
          .filter((element) => element.right > document.documentElement.clientWidth + 1)
          .slice(0, 8)
      }))
      // SearchAll's intrinsic input is independently tracked in #2258; WebKit
      // measures it 51px beyond the viewport at 320px. Keep this issue's
      // assertion on the footer bounds below rather than attributing that
      // route-specific control overflow to the shared footer.
      if (path !== "/platform:search") {
        expect(
          dimensions.document,
          `${testInfo.project.name}: ${path} at ${width}px: ${JSON.stringify(dimensions)}`
        ).toBeLessThanOrEqual(width + 1)
      }

      const footerBounds = await footer.boundingBox()
      expect(footerBounds, `${path} footer at ${width}px`).not.toBeNull()
      expect(footerBounds!.x + footerBounds!.width).toBeLessThanOrEqual(width + 1)

      for (const link of await links.all()) {
        await expect(link).toBeVisible()
        await expect(link).toHaveAttribute("href", /\S/u)
        expect(
          await link.evaluate((element) => (element as HTMLAnchorElement).tabIndex)
        ).toBe(0)
        const bounds = await link.boundingBox()
        expect(bounds, `${path} footer link at ${width}px`).not.toBeNull()
        expect(bounds!.x).toBeGreaterThanOrEqual(0)
        expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width + 1)
      }

      const credit = footer.locator(".footer-powered-by")
      const creditBounds = await credit.boundingBox()
      expect(creditBounds, `${path} credit at ${width}px`).not.toBeNull()
      expect(creditBounds!.x).toBeGreaterThanOrEqual(0)
      expect(creditBounds!.x + creditBounds!.width).toBeLessThanOrEqual(width + 1)

      // Starting on the first footer link and tabbing checks the native order
      // without following links or changing fixture state.
      const footerLinks = await links.all()
      await footerLinks[0].focus()
      for (const link of footerLinks) {
        await expect(link).toBeFocused()
        if (link !== footerLinks[footerLinks.length - 1]) {
          await page.keyboard.press("Tab")
        }
      }

      if (width === 320) {
        await page.evaluate(() => {
          document.documentElement.style.fontSize = "200%"
        })
        if (path !== "/platform:search") {
          expect(
            await page.evaluate(() => document.documentElement.scrollWidth),
            `${testInfo.project.name}: ${path} at 320px with 200% text`
          ).toBeLessThanOrEqual(321)
        }
        const zoomedCredit = await credit.boundingBox()
        expect(zoomedCredit, `${path} zoomed credit`).not.toBeNull()
        expect(zoomedCredit!.x + zoomedCredit!.width).toBeLessThanOrEqual(321)

        await page.evaluate(() => {
          const labels = [
            "Terms and conditions for using this service",
            "Privacy and personal information policy",
            "Documentation and community help center",
            "Security reporting and responsible disclosure"
          ]
          document.querySelectorAll(".footer-item a").forEach((link, index) => {
            link.textContent = labels[index]
          })
          const footer = document.querySelector<HTMLElement>(".footer-inner")
          if (footer) {
            footer.style.fontSize = "200%"
          }
          const credit = document.querySelector<HTMLElement>(".footer-powered-by")
          if (credit) {
            credit.textContent =
              "Powered by Wikijump, the community maintained platform for collaborative documentation"
          }
        })
        const expandedLayout = await page.evaluate(() => {
          const links = document.querySelector<HTMLElement>(".footer-items")
          const credit = document.querySelector<HTMLElement>(".footer-powered-by")
          if (!links || !credit) return null
          const linkBounds = links.getBoundingClientRect()
          const creditBounds = credit.getBoundingClientRect()
          return {
            viewport: document.documentElement.clientWidth,
            document: document.documentElement.scrollWidth,
            links: {
              left: linkBounds.left,
              right: linkBounds.right,
              top: linkBounds.top,
              bottom: linkBounds.bottom
            },
            credit: {
              left: creditBounds.left,
              right: creditBounds.right,
              top: creditBounds.top,
              bottom: creditBounds.bottom
            }
          }
        })
        expect(expandedLayout, `${path} long-label footer geometry`).not.toBeNull()
        if (path !== "/platform:search") {
          expect(
            expandedLayout!.document,
            `${testInfo.project.name}: ${path} at 320px with long labels: ${JSON.stringify(expandedLayout)}`
          ).toBeLessThanOrEqual(321)
        }
        expect(expandedLayout!.links.left).toBeGreaterThanOrEqual(0)
        expect(expandedLayout!.credit.right).toBeLessThanOrEqual(321)
        expect(
          expandedLayout!.links.bottom <= expandedLayout!.credit.top ||
            expandedLayout!.credit.bottom <= expandedLayout!.links.top,
          `${path} long labels overlap the powered-by credit: ${JSON.stringify(expandedLayout)}`
        ).toBe(true)
      }
    }
  }
})
