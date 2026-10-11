import {
  expect,
  installNativeEventListenerProbe,
  test,
  waitForNativeEventListener
} from "./hermetic-playwright"

const headers = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}
const inertWikidotHref = ["java", "script:;"].join("")

test("authenticated account toggle preserves its Wikidot URL without a CSP violation", async ({
  page
}) => {
  const cspMessages: string[] = []
  page.on("console", (message) => {
    if (/Content Security Policy/i.test(message.text())) {
      cspMessages.push(message.text())
    }
  })
  await installNativeEventListenerProbe(page)
  await page.context().addCookies([
    {
      name: "wikijump_token",
      value: "fixture-authenticated-session-token",
      url: `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}`
    }
  ])
  await page.setExtraHTTPHeaders(headers)
  await page.goto("/wikidot-tabview", { waitUntil: "domcontentloaded" })
  await waitForNativeEventListener(page, null, "click")

  const toggle = page.locator("#account-topbutton")
  const menu = page.locator("#account-options")
  await expect(toggle).toHaveCount(1)
  await expect(toggle).toHaveAttribute("href", inertWikidotHref)
  await expect(toggle).toHaveText("▼")
  await expect(menu).toBeHidden()

  // The hermetic imported theme has an external-script CSP message at load;
  // scope this assertion to the top-bar action.
  cspMessages.length = 0
  const beforeClick = page.url()
  await toggle.click()

  await expect(menu).toBeVisible()
  expect(page.url()).toBe(beforeClick)
  await expect(toggle).toHaveAttribute("href", inertWikidotHref)
  expect(cspMessages).toEqual([])
})

test("Wikidot header extension hooks exist at the initial no-script paint", async ({
  browser
}) => {
  const context = await browser.newContext({
    extraHTTPHeaders: headers,
    javaScriptEnabled: false
  })
  const page = await context.newPage()
  try {
    await page.goto(
      `http://localhost:${process.env.PLAYWRIGHT_APP_PORT ?? "4173"}/wikidot-tabview`,
      { waitUntil: "domcontentloaded" }
    )
    await expect
      .poll(() =>
        page
          .locator("#header")
          .evaluate((header) =>
            [...header.children]
              .map((child) => child.id)
              .filter((id) => id.startsWith("header-extra-div-"))
          )
      )
      .toEqual(["header-extra-div-1", "header-extra-div-2", "header-extra-div-3"])
    await expect(page.locator("#header > [id^='header-extra-div-'] > span")).toHaveCount(
      3
    )
    const close = page.locator("#side-bar > a.close-menu")
    await expect(close).toHaveAttribute("href", "##")
    await expect(close.locator(":scope > br")).toHaveCount(2)
    await expect(close.locator(":scope > img.image")).toHaveAttribute("alt", "black.png")
    await expect(close.locator(":scope > img.image")).toHaveAttribute(
      "src",
      /^data:image\/png;base64,/u
    )
  } finally {
    await context.close()
  }
})

test("mobile sidebar close control clears the native target state", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.setExtraHTTPHeaders(headers)
  await page.goto("/wikidot-tabview#side-bar", { waitUntil: "load" })
  const close = page.locator("#side-bar > a.close-menu")
  await expect(close).toBeVisible()
  expect(
    await page.locator("#side-bar").evaluate((element) => element.matches(":target"))
  ).toBe(true)
  await close.click({ position: { x: 350, y: 400 } })
  await expect(close).toBeHidden()
  expect(
    await page.locator("#side-bar").evaluate((element) => element.matches(":target"))
  ).toBe(false)
  expect(await page.evaluate(() => location.hash)).toBe("##")
})

test("Wikidot-compatible search chrome preserves its two inputs and focus behavior", async ({
  page
}) => {
  await installNativeEventListenerProbe(page)
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))

  await page.setExtraHTTPHeaders(headers)

  await page.goto("/wikidot-tabview")
  await waitForNativeEventListener(page, "#search-top-box-input", "focus")
  await waitForNativeEventListener(page, "#search-top-box-form", "submit")

  const searchBox = page.locator("#search-top-box")
  const form = page.locator("#search-top-box-form")
  const query = form.locator('input[type="text"]')
  const submit = form.locator('input[type="submit"]')
  // Retained live DOM evidence: install/local/wikidot-verification/artifacts/
  // listpages-campaign-live-fixtures.json, case lp-live-parent-selectors,
  // raw_page_html_sha256 246ac8385251557163536aef081026d3751425c13e85d49e74ff56b527354290.
  await expect(searchBox).toHaveClass("form-search")
  await expect(form).toHaveCount(1)
  await expect(form).toHaveClass("input-append")
  await expect(query).toHaveValue("Search this site")
  await expect(query).toHaveClass("text empty search-query")
  await expect(submit).toHaveValue("Search")
  await expect(submit).toHaveClass("button btn")

  // Sigma intentionally hides the text field, so exercise the listener without
  // requiring an element that the imported theme makes non-focusable.
  await query.dispatchEvent("focus")
  expect(pageErrors).toEqual([])
  await expect(query).toHaveValue("")
  await expect(query).toHaveClass("text search-query")

  await query.evaluate((input: HTMLInputElement) => {
    input.value = "codex search probe"
    input.dispatchEvent(new Event("input", { bubbles: true }))
  })
  await expect(query).toHaveValue("codex search probe")
  await form.locator('input[type="submit"]').click({ force: true })
  await expect(page).toHaveURL(/\/search:site\/q\/codex%20search%20probe$/u)
  await expect(page.locator("#page-content")).toContainText(
    "Search is temporarily unavailable, we are working to bring it online!"
  )
})

test("SearchAll module submits the selected live area route", async ({ page }) => {
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(headers)

  await page.goto("/search:all")
  await waitForNativeEventListener(page, null, "submit")

  const form = page.locator("#search-form-all")
  await expect(form).toHaveCount(1)
  await expect(form.locator('input[name="area"]:checked')).toHaveValue("pf")
  await form.locator("#search-form-all-input").fill("  a/b? c  ")
  await form.locator("#search-all-f").check()
  await form.locator('input[type="submit"]').click()

  await expect(page).toHaveURL(/\/search:all\/a\/f\/q\/%20%20a%2Fb%3F%20c%20%20$/u)
})

test("seeded platform SearchAll intercepts submit after hydration", async ({ page }) => {
  await installNativeEventListenerProbe(page)
  await page.setExtraHTTPHeaders(headers)
  await page.goto("/platform:search")
  await waitForNativeEventListener(page, null, "submit")

  const form = page.locator("#search-form-all")
  const query = form.locator("#search-form-all-input")
  await expect(form).toHaveAttribute("action", "dummy")

  await query.fill("SCP")
  await form.locator('input[type="submit"]').click()
  await expect(page).toHaveURL(/\/search:all\/a\/pf\/q\/SCP$/u)

  await page.goto("/platform:search")
  await waitForNativeEventListener(page, null, "submit")
  await page.getByRole("link", { name: "terms-conditions", exact: true }).click()
  await expect(page).toHaveURL(/\/$/u)
  await page.goBack()
  await expect(page).toHaveURL(/\/platform:search$/u)
  await waitForNativeEventListener(page, null, "submit")

  const keyboardForm = page.locator("#search-form-all")
  const keyboardQuery = keyboardForm.locator("#search-form-all-input")
  await keyboardForm.locator("#search-all-p").check()
  await keyboardQuery.fill("  a/b? c  ")
  await keyboardQuery.press("Enter")
  await expect(page).toHaveURL(/\/search:all\/a\/p\/q\/%20%20a%2Fb%3F%20c%20%20$/u)
})
