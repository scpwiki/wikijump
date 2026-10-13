import { fileURLToPath } from "node:url"

import { expect, test } from "./hermetic-playwright"

const responsiveTableStyles = fileURLToPath(
  new URL("../src/routes/[slug]/[...extra]/responsive-tables.css", import.meta.url)
)

test("narrow Wikidot transcript tables wrap without widening the document", async ({
  page,
  browserName
}) => {
  test.skip(browserName !== "chromium", "Focused width proof runs in Chromium")

  await page.setContent(`
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      * { box-sizing: border-box; }
      body { margin: 0; }
      #page-content { width: calc(100vw - 32px); margin: 16px; }
      table { border-collapse: collapse; }
      td { padding: 12px; }
      blockquote { width: 100%; margin: 0; }
    </style>
    <div id="page-content">
      <table id="transcript"><tbody><tr><td><blockquote>
        <p>Subject: Dr. Harold Blank (Chair, Archives and Review).</p>
        <p>https://example.test/${"unbroken".repeat(30)}</p>
      </blockquote></td></tr></tbody></table>
      <table id="transcript-fixed"><tbody><tr><td style="padding: 12px;"><blockquote style="width: 336px; margin: 0;">
        <p>Subject: Margherita Villar (civilian)</p>
        <p>https://example.test/${"unbroken".repeat(30)}</p>
      </blockquote></td></tr></tbody></table>
    </div>
    <table id="authored-width" style="width: 480px" hidden><tbody><tr><td>desktop table</td></tr></tbody></table>
  `)
  await page.addStyleTag({ path: responsiveTableStyles })

  for (const width of [280, 320, 375, 390]) {
    await page.setViewportSize({ width, height: 900 })
    const geometry = await page.evaluate(() => {
      const content = document.querySelector<HTMLElement>("#page-content")!
      const table = document.querySelector<HTMLElement>("#transcript")!
      const cell = table.querySelector<HTMLElement>("td")!
      const fixed = document.querySelector<HTMLElement>("#transcript-fixed")!
      const fixedCell = fixed.querySelector<HTMLElement>("td")!
      return {
        viewport: document.documentElement.clientWidth,
        document: document.documentElement.scrollWidth,
        content: content.getBoundingClientRect().width,
        table: table.getBoundingClientRect().width,
        cell: cell.getBoundingClientRect().width,
        cellScroll: cell.scrollWidth,
        text: cell.textContent,
        fixedTable: fixed.getBoundingClientRect().width,
        fixedCell: fixedCell.getBoundingClientRect().width,
        fixedCellScroll: fixedCell.scrollWidth,
        fixedText: fixedCell.textContent
      }
    })

    expect(geometry.viewport).toBe(width)
    expect(geometry.document).toBeLessThanOrEqual(width)
    expect(geometry.table).toBeLessThanOrEqual(geometry.content)
    expect(geometry.cellScroll).toBeLessThanOrEqual(geometry.cell)
    expect(geometry.text).toContain("Subject: Dr. Harold Blank")
    expect(geometry.text).toContain("https://example.test/")
    expect(geometry.text).toContain("unbrokenunbroken")
    // The fixed 336px interior block reproduces the SCP-7000 dialogue tables
    // (issue #2166): it must wrap inside the article column instead of
    // forcing the table and document past the viewport.
    expect(geometry.fixedTable).toBeLessThanOrEqual(geometry.content)
    expect(geometry.fixedCellScroll).toBeLessThanOrEqual(geometry.fixedCell)
    expect(geometry.fixedText).toContain("Subject: Margherita Villar")
    expect(geometry.fixedText).toContain("unbrokenunbroken")
  }

  await page.setViewportSize({ width: 1280, height: 900 })
  await page
    .locator("#authored-width")
    .evaluate((table) => table.removeAttribute("hidden"))
  await expect(page.locator("#authored-width")).toHaveCSS("width", "480px")
})
