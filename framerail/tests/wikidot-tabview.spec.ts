import { expect, test } from "@playwright/test"

test("Wikidot-compatible tabviews switch panels without inline script execution", async ({
  page
}) => {
  const consoleErrors: string[] = []
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text())
  })

  await page.goto("/wikidot-tabview")

  const tabs = page.locator(".yui-navset > .yui-nav > li")
  const panels = page.locator(".yui-navset > .yui-content > div")
  await expect(tabs.nth(0)).toHaveClass(/selected/)
  await expect(panels.nth(0)).toBeVisible()
  await expect(panels.nth(1)).toBeHidden()

  await tabs.nth(1).locator("a").click()

  await expect(tabs.nth(0)).not.toHaveClass(/selected/)
  await expect(tabs.nth(1)).toHaveClass(/selected/)
  await expect(panels.nth(0)).toBeHidden()
  await expect(panels.nth(1)).toBeVisible()
  expect(
    consoleErrors.filter(
      (message) =>
        message.includes("Running the JavaScript URL") ||
        (message.includes("Content Security Policy") && message.includes("script-src"))
    )
  ).toEqual([])
})
