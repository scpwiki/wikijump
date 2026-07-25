import { expect, test } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("Wikidot styleFrame injects and removes its parent stylesheet", async ({ page }) => {
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/wikidot-tabview")

  const css = ".styleframe-browser-marker { color: rgb(1, 2, 3); }"
  const source = `/-/wikidot-interwiki/styleFrame.html?priority=2&css=${encodeURIComponent(css)}`
  await page.evaluate((iframeSource) => {
    const iframe = document.createElement("iframe")
    iframe.id = "styleframe-browser-fixture"
    iframe.src = iframeSource
    document.body.appendChild(iframe)
  }, source)

  const injected = page.locator(
    'head style[data-wikidot-style-frame="wikidot-style-frame"]'
  )
  await expect(injected).toHaveCount(1)
  expect(await injected.textContent()).toBe(css)
  await expect(injected).toHaveAttribute("data-wikidot-style-priority", "2")
  await expect(injected).toHaveAttribute(
    "data-wikidot-style-owner",
    /^wikidot-style-frame-/u
  )

  await page.locator("#styleframe-browser-fixture").evaluate((iframe) => iframe.remove())
  await expect(injected).toHaveCount(0)
  expect(pageErrors).toEqual([])
})
