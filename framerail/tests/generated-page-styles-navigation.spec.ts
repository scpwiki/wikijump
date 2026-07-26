import { expect, test } from "@playwright/test"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

const generatedCss = (page: import("@playwright/test").Page) =>
  page
    .locator("head style[data-wikijump-generated-css]")
    .evaluateAll((styles) => styles.map((style) => style.textContent))

const generatedCssClones = (page: import("@playwright/test").Page) =>
  page
    .locator("head style[data-wikijump-generated-css-clone]")
    .evaluateAll((styles) => styles.map((style) => style.textContent))

test("client navigation replaces styleFrame-ordered generated page CSS", async ({
  page
}) => {
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))
  await page.route(/\/navigation-style-[a-d]\/_app\//u, async (route) => {
    const url = new URL(route.request().url())
    url.pathname = url.pathname.slice(url.pathname.indexOf("/_app/"))
    await route.continue({ url: url.href })
  })
  await page.setExtraHTTPHeaders(SITE_HEADERS)

  await page.goto("/navigation-style-a")
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
      })
  )
  expect(pageErrors).toEqual([])
  await expect(
    page.locator('head [data-wikidot-style-frame="wikidot-style-frame"]')
  ).toHaveCount(1)
  expect(await generatedCss(page)).toEqual([".generated-style-a { color: red; }"])

  await page.evaluate(() => {
    ;(
      window as Window & {
        wikijumpNavigationSentinel?: string
      }
    ).wikijumpNavigationSentinel = "client-runtime-alive"
  })
  await page.locator("#navigate-style-b").click()
  await expect(page).toHaveURL(/\/navigation-style-b$/u)
  await expect(
    page.locator('head [data-wikidot-style-frame="wikidot-style-frame"]')
  ).toHaveCount(1)
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(2)
  expect(
    await page.evaluate(
      () =>
        (
          window as Window & {
            wikijumpNavigationSentinel?: string
          }
        ).wikijumpNavigationSentinel
    )
  ).toBe("client-runtime-alive")
  expect(pageErrors).toEqual([])

  const clickedStyleB = await generatedCss(page)
  expect(clickedStyleB).toEqual([
    ".generated-style-b-one { color: blue; }",
    ".generated-style-b-two { color: green; }"
  ])
  expect(await generatedCssClones(page)).toEqual(clickedStyleB)

  await page.goto("/navigation-style-b")
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(2)
  await expect(page.locator("head style[data-wikijump-generated-css-clone]")).toHaveCount(
    2
  )
  expect(await generatedCss(page)).toEqual(clickedStyleB)
  expect(await generatedCssClones(page)).toEqual(clickedStyleB)

  await page.goto("/navigation-style-c")
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(2)
  await page.evaluate(() => {
    ;(
      window as Window & {
        wikijumpNavigationSentinel?: string
      }
    ).wikijumpNavigationSentinel = "client-runtime-alive"
  })
  await page.locator("#navigate-style-d").click()
  await expect(page).toHaveURL(/\/navigation-style-d$/u)
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(1)
  expect(
    await page.evaluate(
      () =>
        (
          window as Window & {
            wikijumpNavigationSentinel?: string
          }
        ).wikijumpNavigationSentinel
    )
  ).toBe("client-runtime-alive")

  const clickedStyleD = await generatedCss(page)
  expect(clickedStyleD).toEqual([".generated-style-d { color: black; }"])
  expect(await generatedCssClones(page)).toEqual(clickedStyleD)
  await page.goto("/navigation-style-d")
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(1)
  await expect(page.locator("head style[data-wikijump-generated-css-clone]")).toHaveCount(
    1
  )
  expect(await generatedCss(page)).toEqual(clickedStyleD)
  expect(await generatedCssClones(page)).toEqual(clickedStyleD)
})
