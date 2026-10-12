import fs from "node:fs"
import { expect, test, waitForSvelteDelegatedHandler } from "./hermetic-playwright"

const observed = JSON.parse(
  fs.readFileSync(
    new URL("./fixtures/wikidot-standard-page-chrome.json", import.meta.url),
    "utf8"
  )
)
const english = observed.cases.find((entry: { locale: string }) => entry.locale === "en")
const headers = { "X-Wikijump-Site-Id": "6000005", "X-Wikijump-Site-Slug": "scp-wiki" }

test("Wikidot secondary actions retain their standard order, names and anchor DOM", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(headers)
  await page.goto("/wikidot-discussion-probe")
  await expect(page.locator("#discuss-button")).toBeAttached()
  const row = page.locator("#page-options-bottom-2")
  await expect(row).toBeHidden()
  expect(
    await row
      .locator("a")
      .evaluateAll((links) =>
        links.map((link) => ({ id: link.id, text: link.textContent?.trim() }))
      )
  ).toEqual(english.secondary_actions)
  await expect(row.locator("#layout-button")).toHaveCount(0)
  await expect(row.locator("#lock-page-button")).toHaveCount(0)
  expect(
    await row
      .locator("a")
      .evaluateAll((links) => links.map((link) => link.getAttribute("type")))
  ).toEqual(english.secondary_actions.map(() => null))
  await waitForSvelteDelegatedHandler(page, "#more-options-button")
  await page.locator("#more-options-button").click()
  await expect(row).toBeVisible()
  await expect(page.locator("#more-options-button")).toHaveText("-\u00a0Options")
  for (const [control, html] of Object.entries(english.tips)) {
    const tip = page.locator(`#${control}-hovertip > .content`)
    expect(await tip.innerHTML()).toBe(html)
  }
  await page.locator("#more-options-button").click()
  await expect(row).toBeHidden()
})

for (const locale of ["en-US", "ja-JP"]) {
  test.describe(`locally authored secondary actions remain available (${locale})`, () => {
    test.use({ locale })
    test("local authoring retains its layout action and lock identifier", async ({
      page
    }) => {
      await page.setExtraHTTPHeaders(headers)
      await page.goto("/scp-173")
      const row = page.locator("#page-options-bottom-2")
      await expect(row.locator("#layout-button")).toHaveCount(1)
      await expect(row.locator("#lock-page-button")).toHaveCount(1)
      await expect(row.locator("#page-block-button")).toHaveCount(0)
      expect(
        await row
          .locator("a")
          .evaluateAll((links) => links.slice(0, 2).map((link) => link.id))
      ).toEqual(["edit-append-button", "edit-sections-button"])
    })
  })
}
