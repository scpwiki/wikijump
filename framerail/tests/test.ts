import { expect, test } from "@playwright/test"

test("Main page h1 has string", async ({ page }) => {
  // TODO
  await page.goto("/")
  expect(await page.textContent("h1")).toBe("UNTRANSLATED:Loaded main page")
})
