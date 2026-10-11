import {
  expect,
  installNativeEventListenerProbe,
  test,
  waitForNativeEventListener
} from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const APP_URL = `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`

test("printer font and source-info controls work without changing the page URL", async ({
  page
}) => {
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto(`${APP_URL}/printer--friendly//main`)
  await waitForNativeEventListener(page, "#container", "click")

  const printerRoot = page.locator("#container")
  const content = page.locator("#print-content")
  const fontControls = page.locator(
    '#print-options a[onclick^="WIKIDOT.printview.listeners.changeFontFamily"]'
  )
  const sourceInfo = page.locator("#print-source-info")
  const sourceToggle = page.locator(
    '#print-options a[onclick="WIKIDOT.printview.listeners.toggleSourceInfo(event)"]'
  )
  const originalUrl = page.url()

  await expect(printerRoot).toBeVisible()
  await expect(content).toContainText("Main")
  await expect(fontControls).toHaveCount(5)
  await expect(fontControls).toHaveText([
    "original font",
    "Georgia",
    "Times New Roman",
    "Serif (generic)",
    "Arial/Helvetica"
  ])
  await expect(fontControls.nth(0)).toHaveAttribute("aria-pressed", "true")

  for (const [label, expectedFamily] of [
    ["Georgia", "Georgia"],
    ["Times New Roman", '"Times New Roman"'],
    ["Serif (generic)", "serif"],
    ["Arial/Helvetica", "Arial, Helvetica, sans-serif"]
  ]) {
    await fontControls.getByText(label, { exact: true }).click()
    await expect(content).toHaveCSS("font-family", expectedFamily)
    await expect(printerRoot).toHaveCSS("font-family", "monospace")
    await expect(fontControls.getByText(label, { exact: true })).toHaveAttribute(
      "aria-pressed",
      "true"
    )
    expect(page.url()).toBe(originalUrl)
  }

  await fontControls.getByText("original font", { exact: true }).click()
  await expect(content).toHaveCSS("font-family", "monospace")
  await expect(fontControls.nth(0)).toHaveAttribute("aria-pressed", "true")
  expect(page.url()).toBe(originalUrl)

  await page
    .locator('#print-options a[onclick*="changeFontSize"]')
    .getByText("12pt", { exact: true })
    .click()
  await expect(content).toHaveCSS("font-size", "16px")
  await fontControls.getByText("Georgia", { exact: true }).click()
  await expect(content).toHaveCSS("font-family", "Georgia")
  await expect(content).toHaveCSS("font-size", "16px")

  await sourceToggle.click()
  await expect(sourceInfo).toBeHidden()
  await expect(sourceToggle).toHaveAttribute("aria-pressed", "true")
  await expect(content).toBeVisible()
  await page.emulateMedia({ media: "print" })
  await expect(sourceInfo).toHaveCSS("display", "none")
  await expect(content).toBeVisible()

  await page.emulateMedia({ media: "screen" })
  await sourceToggle.click()
  await expect(sourceInfo).toBeVisible()
  await expect(sourceToggle).toHaveAttribute("aria-pressed", "false")
  expect(page.url()).toBe(originalUrl)
})
