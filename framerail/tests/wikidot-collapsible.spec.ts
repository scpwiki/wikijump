import { expect, test } from "@playwright/test"

test("Wikidot-compatible collapsibles preserve legacy interaction", async ({ page }) => {
  const consoleErrors: string[] = []
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text())
  })

  await page.goto("/wikidot-collapsible")

  const folded = page.locator("#folded-collapsible")
  const foldedHandle = folded.locator(":scope > .collapsible-block-folded")
  const unfolded = folded.locator(":scope > .collapsible-block-unfolded")
  const showLink = foldedHandle.locator("a.collapsible-block-link")
  const hideLinks = unfolded.locator(
    ":scope > .collapsible-block-unfolded-link > a.collapsible-block-link"
  )

  await expect(foldedHandle).toBeVisible()
  await expect(unfolded).toBeHidden()
  await showLink.click()
  await expect(foldedHandle).toBeHidden()
  await expect(unfolded).toBeVisible()

  await hideLinks.first().press("Space")
  await expect(foldedHandle).toBeVisible()
  await expect(unfolded).toBeHidden()

  await showLink.press("Enter")
  await expect(foldedHandle).toBeHidden()
  await expect(unfolded).toBeVisible()
  await hideLinks.last().click()
  await expect(foldedHandle).toBeVisible()
  await expect(unfolded).toBeHidden()

  const initiallyOpen = page.locator("#open-collapsible")
  await expect(initiallyOpen.locator(":scope > .collapsible-block-folded")).toBeHidden()
  await expect(
    initiallyOpen.locator(":scope > .collapsible-block-unfolded")
  ).toBeVisible()

  const native = page.locator("#native-collapsible")
  await expect(native).not.toHaveAttribute("open", "")
  await native.locator(":scope > summary").click()
  await expect(native).toHaveAttribute("open", "")

  expect(
    consoleErrors.filter(
      (message) =>
        message.includes("Running the JavaScript URL") ||
        (message.includes("Content Security Policy") && message.includes("script-src"))
    )
  ).toEqual([])
})
