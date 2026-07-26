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

type NavigationAnimationFrame = {
  generatedClones: number
  href: string
  markedStyles: number
  pageTitleDisplay: string | null
  sideBarDisplay: string | null
  themeStylesheetsReady: boolean
}

type PresentedFrame = {
  data: string
  probe: Promise<NavigationAnimationFrame | null>
}

test("Wikidot page links do not present the destination before its CSS", async ({
  page
}) => {
  const pageErrors: string[] = []
  let markThemeRequested!: () => void
  let releaseThemeResponse!: () => void
  const themeRequested = new Promise<void>((resolve) => {
    markThemeRequested = resolve
  })
  const themeResponseReleased = new Promise<void>((resolve) => {
    releaseThemeResponse = resolve
  })
  page.on("pageerror", (error) => pageErrors.push(error.message))
  await page.route("**/navigation-style-b-theme.css", async (route) => {
    markThemeRequested()
    await themeResponseReleased
    await route.fulfill({
      body: "body { background-color: rgb(1, 2, 3); } #side-bar { display: none !important; }",
      contentType: "text/css"
    })
  })
  await page.route(/\/navigation-style-[a-d]\/_app\//u, async (route) => {
    const url = new URL(route.request().url())
    url.pathname = url.pathname.slice(url.pathname.indexOf("/_app/"))
    await route.continue({ url: url.href })
  })
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  const destinationResponse = await page.request.get("/navigation-style-b", {
    headers: SITE_HEADERS
  })
  const destinationHtml = await destinationResponse.text()
  expect(destinationResponse.ok()).toBe(true)
  const themeIndex = destinationHtml.indexOf("navigation-style-b-theme.css")
  const inlineStyleIndex = destinationHtml.indexOf("#page-title{display:none}")
  const generatedStyleIndex = destinationHtml.indexOf(
    ".generated-style-b-one { color: blue; }"
  )
  const headEndIndex = destinationHtml.indexOf("</head>")
  expect(themeIndex).toBeGreaterThan(0)
  expect(inlineStyleIndex).toBeGreaterThan(themeIndex)
  expect(generatedStyleIndex).toBeGreaterThan(inlineStyleIndex)
  expect(generatedStyleIndex).toBeLessThan(headEndIndex)

  const cdp = await page.context().newCDPSession(page)
  await page.goto("/navigation-style-a")
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
      })
  )
  expect(pageErrors).toEqual([])
  await expect(page.locator("#skrollr-body")).toHaveAttribute("data-sveltekit-reload", "")
  await expect(
    page.locator('head [data-wikidot-style-frame="wikidot-style-frame"]')
  ).toHaveCount(1)
  expect(await generatedCss(page)).toEqual([".generated-style-a { color: red; }"])
  const presentedFrames: PresentedFrame[] = []
  let markFirstPresentedFrame!: () => void
  const firstPresentedFrame = new Promise<void>((resolve) => {
    markFirstPresentedFrame = resolve
  })
  cdp.on("Page.screencastFrame", (event) => {
    presentedFrames.push({
      data: event.data,
      probe: cdp
        .send("Runtime.evaluate", {
          expression: `(() => {
            const pageTitle = document.querySelector("#page-title");
            const sideBar = document.querySelector("#side-bar");
            return {
              generatedClones: document.querySelectorAll(
                "head style[data-wikijump-generated-css-clone]"
              ).length,
              href: location.pathname,
              markedStyles: document.querySelectorAll(
                'head [data-wikidot-style-frame="wikidot-style-frame"]'
              ).length,
              pageTitleDisplay: pageTitle ? getComputedStyle(pageTitle).display : null,
              sideBarDisplay: sideBar ? getComputedStyle(sideBar).display : null,
              themeStylesheetsReady: Array.from(
                document.querySelectorAll('head link[rel="stylesheet"]')
              )
                .filter((stylesheet) => stylesheet.hasAttribute("data-wikidot-style-frame"))
                .every((stylesheet) => stylesheet.sheet !== null)
            };
          })()`,
          returnByValue: true
        })
        .then((result) => (result.result.value as NavigationAnimationFrame) ?? null)
        .catch(() => null)
    })
    markFirstPresentedFrame()
    void cdp.send("Page.screencastFrameAck", { sessionId: event.sessionId })
  })
  await cdp.send("Page.startScreencast", {
    everyNthFrame: 1,
    format: "png",
    quality: 100
  })
  await firstPresentedFrame
  const oldDocumentFrame = presentedFrames.at(-1)?.data
  expect(oldDocumentFrame).toBeTruthy()
  presentedFrames.length = 0

  await page.evaluate(() => {
    ;(
      window as Window & {
        wikijumpNavigationSentinel?: string
      }
    ).wikijumpNavigationSentinel = "client-runtime-alive"
  })
  const navigation = page.waitForNavigation()
  const click = page.locator("#navigate-style-b").evaluate((link: HTMLAnchorElement) => {
    link.click()
  })
  await themeRequested
  await page.waitForTimeout(300)
  const framesDuringThemeDelay = [...presentedFrames]
  const onlyOldDocumentFramesDuringThemeDelay = framesDuringThemeDelay.every(
    (frame) => frame.data === oldDocumentFrame
  )
  releaseThemeResponse()
  await Promise.all([click, navigation])
  expect(onlyOldDocumentFramesDuringThemeDelay).toBe(true)
  await expect(
    page.locator('head [data-wikidot-style-frame="wikidot-style-frame"]')
  ).toHaveCount(2)
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
      })
  )
  await page.waitForTimeout(100)
  await cdp.send("Page.stopScreencast")
  const changedFrameProbes = presentedFrames
    .filter((frame) => frame.data !== oldDocumentFrame)
    .map((frame) => frame.probe)
  const changedFrameStates = await Promise.all(changedFrameProbes)
  const destinationFrames = changedFrameStates.filter(
    (frame): frame is NavigationAnimationFrame => frame?.href === "/navigation-style-b"
  )
  expect(destinationFrames.length).toBeGreaterThan(0)
  for (const frame of destinationFrames) {
    expect(frame).toMatchObject({
      generatedClones: 0,
      markedStyles: 2,
      pageTitleDisplay: "none",
      sideBarDisplay: "none",
      themeStylesheetsReady: true
    })
  }
  await expect(page).toHaveURL(/\/navigation-style-b$/u)
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
  ).toBeUndefined()
  expect(
    await page.evaluate(
      () =>
        (performance.getEntriesByType("navigation")[0] as PerformanceNavigationTiming)
          .type
    )
  ).toBe("navigate")
  expect(pageErrors).toEqual([])

  const clickedStyleB = await generatedCss(page)
  expect(clickedStyleB).toEqual([
    ".generated-style-b-one { color: blue; }",
    ".generated-style-b-two { color: green; }"
  ])
  expect(await generatedCssClones(page)).toEqual([])

  await page.goto("/navigation-style-b")
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(2)
  await expect(page.locator("head style[data-wikijump-generated-css-clone]")).toHaveCount(
    0
  )
  expect(await generatedCss(page)).toEqual(clickedStyleB)
  expect(await generatedCssClones(page)).toEqual([])

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
  ).toBeUndefined()
  expect(
    await page.evaluate(
      () =>
        (performance.getEntriesByType("navigation")[0] as PerformanceNavigationTiming)
          .type
    )
  ).toBe("navigate")

  const clickedStyleD = await generatedCss(page)
  expect(clickedStyleD).toEqual([".generated-style-d { color: black; }"])
  expect(await generatedCssClones(page)).toEqual([])
  await page.goto("/navigation-style-d")
  await expect(page.locator("head style[data-wikijump-generated-css]")).toHaveCount(1)
  await expect(page.locator("head style[data-wikijump-generated-css-clone]")).toHaveCount(
    0
  )
  expect(await generatedCss(page)).toEqual(clickedStyleD)
  expect(await generatedCssClones(page)).toEqual([])
})
