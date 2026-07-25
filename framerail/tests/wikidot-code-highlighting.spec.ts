import { expect, test } from "@playwright/test"

test("Wikidot code highlighting applies and restores token markup", async ({ page }) => {
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  await page.goto("/wikidot-code-highlighting")

  const code = page.locator('.code[data-wj-language="css"] pre > code')
  await expect(code).toHaveText("#header h2 span { color: red; }")
  await expect(code.locator("span.wj-code-token.wj-code-selector")).toHaveCount(1)
  await expect(code.locator("span.wj-code-token.wj-code-property")).toHaveCount(1)

  await code.evaluate((element) => {
    element.replaceChildren(document.createTextNode(element.textContent ?? ""))
  })

  await expect(code.locator("span.wj-code-token.wj-code-selector")).toHaveCount(1)
  await expect(code.locator("span.wj-code-token.wj-code-property")).toHaveCount(1)
  await expect(code).toHaveText("#header h2 span { color: red; }")
})
