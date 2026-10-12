import { readFileSync } from "node:fs"
import { test, expect } from "@playwright/test"

// Exercise the product's actual mobile rules with both dialog shells. These
// rules contain plain CSS even though their enclosing file uses SCSS.
const source = readFileSync(
  new URL("../src/lib/popup/error.svelte", import.meta.url),
  "utf8"
)
const start = source.indexOf("@media (max-width: 600px)")
const brace = source.indexOf("{", start)
let end = brace + 1,
  depth = 1
while (depth > 0) {
  if (source[end] === "{") depth += 1
  else if (source[end] === "}") depth -= 1
  end += 1
}
const css = source.slice(start, end)

for (const native of [false, true]) {
  test(`mobile error rules apply to ${native ? "the native popup" : "no standard OZONE popup"}`, async ({
    page
  }) => {
    await page.setViewportSize({ width: 390, height: 844 })
    await page.setContent(
      `<style>${css}</style><div id="odialog-container"${native ? ' class="odialog-container"' : ""}><div class="owindow error"><div class="content"><p>Permission message</p></div><div class="button-bar"><a href="javascript:;">close message</a></div></div></div><div class="button-bar" id="unrelated">Unrelated</div>`
    )
    const state = await page.evaluate(() => ({
      bar: getComputedStyle(document.querySelector("#odialog-container .button-bar")!)
        .display,
      wrapping: getComputedStyle(document.querySelector("#odialog-container p")!)
        .overflowWrap,
      unrelated: getComputedStyle(document.getElementById("unrelated")!).display
    }))
    expect(state).toEqual({
      bar: native ? "flex" : "block",
      wrapping: native ? "anywhere" : "normal",
      unrelated: "block"
    })
  })
}
