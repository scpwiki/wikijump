import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

test("Wikidot page-option and footer controls show their hovertips", async ({ page }) => {
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/wikidot-login-probe")
  await page.mouse.move(683, 10)

  const container = page.locator("body > #odialog-hovertips")
  const editTip = container.locator("#edit-button-hovertip")
  await expect(editTip).toHaveAttribute("class", "hovertip ")
  await expect(editTip.locator("> div.content")).toHaveText(
    "Click here to edit contents of this page."
  )
  await expect(editTip).toBeHidden()
  for (const control of ["wikidot-help-button", "abuse-report-button"]) {
    await expect(container.locator(`#${control}-hovertip`)).toHaveCount(1)
  }

  await page.locator("#edit-button").hover()
  await expect(editTip).toBeVisible()
  expect(
    await editTip.evaluate((tip) => [
      tip.style.position,
      tip.style.display,
      tip.style.border,
      tip.style.left.endsWith("px"),
      tip.style.top.endsWith("px")
    ])
  ).toEqual(["absolute", "block", "1px solid black", true, true])
  const editBox = await page.locator("#edit-button").boundingBox()
  const tipBox = await editTip.boundingBox()
  expect(tipBox && editBox && tipBox.y).toBeGreaterThan((editBox?.y ?? 0) + 0)

  await page.mouse.move(683, 10)
  await expect(editTip).toBeHidden()

  // Controls in the second option row get their tips once the row opens.
  await page.locator("#more-options-button").click()
  await page.locator("#view-source-button").hover()
  await expect(container.locator("#view-source-button-hovertip")).toBeVisible()
  await expect(container.locator("#view-source-button-hovertip")).toHaveText(
    "View wiki source for this page without editing."
  )
  expect(pageErrors).toEqual([])
})
