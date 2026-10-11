import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

const styleFrameSource = (priority: number, css: string) =>
  `/-/wikidot-interwiki/styleFrame.html?priority=${priority}&css=${encodeURIComponent(css)}`

test("Wikidot styleFrames restyle the interwiki frame and never the parent page", async ({
  page
}) => {
  const pageErrors: string[] = []
  page.on("pageerror", (error) => pageErrors.push(error.message))
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/wikidot-tabview")
  const parentHeadStyles = await page.locator("head style, head link").count()

  const appendFrame = (id: string, source: string) =>
    page.evaluate(
      ([frameId, frameSource]) =>
        new Promise<void>((resolve) => {
          const iframe = document.createElement("iframe")
          iframe.id = frameId
          iframe.src = frameSource
          iframe.addEventListener("load", () => resolve(), { once: true })
          document.body.appendChild(iframe)
        }),
      [id, source]
    )

  await page.evaluate(() => {
    const marker = document.createElement("div")
    marker.className = "heading"
    marker.innerHTML = "<p id='styleframe-parent-marker'>parent</p>"
    document.body.appendChild(marker)
  })

  // A styleFrame that loads first is pulled when the interwiki frame starts.
  await appendFrame(
    "styleframe-early",
    styleFrameSource(1, ".heading p { color: rgb(1, 2, 3); }")
  )
  await appendFrame(
    "interwiki-fixture",
    "/-/wikidot-interwiki/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506"
  )
  const interwiki = page.frameLocator("#interwiki-fixture")
  await expect(interwiki.locator(".heading p")).toHaveCSS("color", "rgb(1, 2, 3)")

  // A later styleFrame pushes into the initialized interwiki frame; its higher
  // priority sorts after the earlier declaration and wins the cascade.
  await appendFrame(
    "styleframe-late",
    styleFrameSource(2, ".heading p { color: rgb(4, 5, 6); }")
  )
  await expect(interwiki.locator(".heading p")).toHaveCSS("color", "rgb(4, 5, 6)")
  expect(
    await interwiki
      .locator("head .custom-style")
      .evaluateAll((elements) =>
        elements.map((element) => element.getAttribute("data-priority"))
      )
  ).toEqual(["-1", "1", "2"])

  await expect(page.frameLocator("#styleframe-late").locator("style, link")).toHaveCount(
    0
  )
  await expect(page.locator("head style, head link")).toHaveCount(parentHeadStyles)
  await expect(page.locator("#styleframe-parent-marker")).not.toHaveCSS(
    "color",
    "rgb(4, 5, 6)"
  )
  await expect(page.locator("#styleframe-parent-marker")).not.toHaveCSS(
    "color",
    "rgb(1, 2, 3)"
  )
  expect(pageErrors).toEqual([])
})
