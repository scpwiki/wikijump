import { expect, test } from "./hermetic-playwright"

import { wikidotFoldableLists } from "../src/lib/wikidot/wikidot-foldable-lists.js"

test("native foldable menus preserve independent controls, replacement initialization and cleanup", async ({
  page
}) => {
  await page.setContent(
    '<main><div class="foldable-list-container"><ul><li>Main<ul><li>Child</li></ul></li><li><a href="/elsewhere">Navigation</a><ul><li>Destination</li></ul></li><li class="plain">Plain</li></ul></div><ul class="creditRate"><li class="folded"><ul><li>_</li></ul><div class="creditButton foldable-list-container"><a href="javascript:;">Open</a></div><ul class="otherwise"><li class="folded"><ul><li>_</li></ul><div class="foldable-list-container"><a href="javascript:;">Other</a></div><div class="return-credits foldable-list-container"><a href="javascript:;">Return</a></div></li></ul></li></ul></main>'
  )
  await page.addScriptTag({
    content: `window.foldableAction = (${wikidotFoldableLists.toString()})(document.querySelector("main"));`
  })
  await page.evaluate(() => {
    Object.assign(window, { navigationPrevented: null })
    document.addEventListener("click", (event) => {
      if ((event.target as HTMLElement).textContent === "Navigation") {
        Object.assign(window, { navigationPrevented: event.defaultPrevented })
        event.preventDefault()
      }
    })
  })

  const menu = page.locator("main > div > ul > li").first()
  await expect(menu).toHaveClass("folded")
  await expect(menu.locator(":scope > ul")).toBeHidden()
  await menu.locator(":scope > a").click()
  await expect(menu).toHaveClass("unfolded")
  await expect(menu.locator(":scope > ul")).toBeVisible()
  await page.locator(".creditButton a").click()
  await expect(page.locator(".creditRate > li")).toHaveClass("unfolded")
  await page.getByText("Other", { exact: true }).click()
  await expect(page.locator(".otherwise > li")).toHaveClass("unfolded")
  await page.locator(".return-credits a").click()
  await expect(page.locator(".otherwise > li")).toHaveClass("folded")
  await expect(page.locator(".creditRate > li")).toHaveClass("unfolded")
  await page.getByText("Navigation", { exact: true }).click()
  expect(
    await page.evaluate(
      () => (window as Window & { navigationPrevented: boolean }).navigationPrevented
    )
  ).toBe(false)
  await expect(page.getByText("Navigation", { exact: true }).locator("..")).toHaveClass(
    "folded"
  )
  await page.locator(".plain").click()
  await expect(page.locator(".plain")).toHaveClass("plain")

  await page.evaluate(() =>
    document
      .querySelector("main")!
      .insertAdjacentHTML(
        "beforeend",
        '<div class="foldable-list-container"><ul><li>Replacement<ul><li>Nested</li></ul></li></ul></div>'
      )
  )
  const replacement = page.locator("main > div:last-child li").first()
  await expect(replacement).toHaveClass("folded")
  await page.getByText("Replacement", { exact: true }).click()
  await expect(replacement).toHaveClass("unfolded")
  await page.evaluate(() =>
    (window as Window & { foldableAction: { destroy(): void } }).foldableAction.destroy()
  )
  await page.locator(".creditButton a").click()
  await expect(page.locator(".creditRate > li")).toHaveClass("unfolded")
  expect(await page.evaluate(() => location.hash)).toBe("")
})

test("Info/Credit controls keep the panel hidden until opened and support keyboard and close controls", async ({
  page
}) => {
  const infoBox =
    '<li class="rateBox folded"><ul><li>' +
    '<div id="credit-view"><div class="fader foldable-list-container">close</div>' +
    '<div class="modalcontainer"><div class="modalbox">' +
    '<div class="close-credits foldable-list-container">X</div><div class="credit">Author credits</div>' +
    "</div></div></div></li></ul>" +
    '<div class="creditButton foldable-list-container">Info</div></li>'
  await page.setContent(`<main><ul class="creditRate">${infoBox}${infoBox}</ul></main>`)
  await page.addScriptTag({
    content: `window.foldableAction = (${wikidotFoldableLists.toString()})(document.querySelector("main"));`
  })

  const owner = page.locator(".creditRate > li").nth(0)
  const otherOwner = page.locator(".creditRate > li").nth(1)
  const opener = owner.getByRole("button", { name: "Info", exact: true })
  const panel = owner.locator("#credit-view")
  const content = owner.locator("ul").first()
  await expect(owner).toHaveClass("rateBox folded")
  await expect(otherOwner).toHaveClass("rateBox folded")
  await expect(content).toBeHidden()
  await expect(otherOwner.locator("ul").first()).toBeHidden()
  await expect(panel).toHaveAttribute("role", "dialog")
  await expect(panel).toHaveAttribute("aria-hidden", "true")
  await expect(opener).toHaveAttribute("aria-expanded", "false")

  await opener.focus()
  await page.keyboard.press("Enter")
  await expect(owner).toHaveClass("rateBox unfolded")
  await expect(content).toBeVisible()
  await expect(panel).toHaveAttribute("aria-hidden", "false")
  await expect(opener).toHaveAttribute("aria-expanded", "true")
  await expect(owner.getByText("X", { exact: true })).toBeFocused()
  await page.keyboard.press("Tab")
  await expect(owner.getByText("X", { exact: true })).toBeFocused()
  await page.keyboard.press("Shift+Tab")
  await expect(owner.getByText("X", { exact: true })).toBeFocused()
  await expect(otherOwner).toHaveClass("rateBox folded")
  await expect(otherOwner.locator("ul").first()).toBeHidden()
  await page.keyboard.press("Escape")
  await expect(owner).toHaveClass("rateBox folded")
  await expect(content).toBeHidden()
  await expect(opener).toBeFocused()

  await opener.click()
  await owner.getByText("close", { exact: true }).click()
  await expect(owner).toHaveClass("rateBox folded")
  await expect(content).toBeHidden()
  await expect(opener).toBeFocused()

  await opener.click()
  await owner.getByText("X", { exact: true }).click()
  await expect(owner).toHaveClass("rateBox folded")
  await expect(content).toBeHidden()
  await expect(opener).toBeFocused()

  const otherOpener = otherOwner.getByRole("button", { name: "Info", exact: true })
  await otherOpener.press("Space")
  await expect(otherOwner).toHaveClass("rateBox unfolded")
  await expect(otherOwner.locator("ul").first()).toBeVisible()
  await page.keyboard.press("Escape")
  await expect(otherOwner).toHaveClass("rateBox folded")
  await expect(otherOwner.locator("ul").first()).toBeHidden()
})

test("PageView initializes replacement menus and unfolds ancestors of the current page", async ({
  page
}) => {
  await page.setExtraHTTPHeaders({
    "X-Wikijump-Site-Id": "6000005",
    "X-Wikijump-Site-Slug": "scp-wiki"
  })
  await page.goto("/wikidot-tabview", { waitUntil: "load" })
  await expect(page.locator("#page-content > .yui-navset")).toHaveClass(/yui-navset-top/)
  await page.locator("#page-content").evaluate((content) => {
    content.innerHTML =
      '<div class="foldable-list-container"><ul><li>Current ancestor<ul><li><a href="/wikidot-tabview">Current page</a></li></ul></li><li>Other ancestor<ul><li>Other child</li></ul></li></ul></div>'
  })
  const current = page.locator("#page-content .foldable-list-container > ul > li").nth(0)
  const other = page.locator("#page-content .foldable-list-container > ul > li").nth(1)
  await expect(current).toHaveClass("unfolded")
  await expect(current.locator(":scope > ul")).toBeVisible()
  await expect(other).toHaveClass("folded")
  await expect(other.locator(":scope > ul")).toBeHidden()
  await other.locator(":scope > a").click()
  await expect(other).toHaveClass("unfolded")
  await expect(other.locator(":scope > ul")).toBeVisible()
})
