import { expect, test } from "./hermetic-playwright"

test("rendered inline link whitespace remains readable and selectable across browsers", async ({
  page
}) => {
  await page.setContent(`
    <main id="page-content">
      <p id="authored-separator"><a class="home" href="#">Front Page</a> <a href="#u-target-1">About Fog</a></p>
      <p id="adjacent-links"><a href="#one">one</a><a href="#two">two</a></p>
    </main>
    <style>
      #page-content { white-space: normal; }
      #page-content a { display: inline; }
    </style>
  `)

  const separator = page.locator("#authored-separator")
  const boundary = await page.locator("#authored-separator a.home").evaluate((home) => {
    const next = home.nextSibling
    return {
      nodeType: next?.nodeType,
      text: next?.textContent
    }
  })
  expect(boundary).toEqual({ nodeType: 3, text: " " })
  await expect(separator).toHaveText("Front Page About Fog")
  expect(await separator.evaluate((node) => node.textContent)).toBe(
    "Front Page About Fog"
  )

  const selection = await separator.evaluate((node) => {
    const range = document.createRange()
    range.selectNodeContents(node)
    const current = window.getSelection()
    current?.removeAllRanges()
    current?.addRange(range)
    return current?.toString()
  })
  expect(selection).toBe("Front Page About Fog")

  const adjacent = page.locator("#adjacent-links")
  await expect(adjacent).toHaveText("onetwo")
  expect(await adjacent.evaluate((node) => node.textContent)).toBe("onetwo")
  expect(
    await page
      .locator("#adjacent-links a")
      .first()
      .evaluate((link) => link.nextSibling?.nodeType)
  ).toBe(1)
})
