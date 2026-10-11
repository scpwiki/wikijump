import { strict as assert } from "node:assert"
import test from "node:test"
import { readFile } from "node:fs/promises"

import {
  buildWikidotPrintOptionsHtml,
  buildWikidotPrintSourceInfoHtml,
  buildWikidotPrinterFriendlyUrl,
  wikidotPrintView
} from "../src/lib/wikidot/wikidot-print-view.js"

test("printer-friendly URL preserves the live doubled slash", () => {
  assert.equal(
    buildWikidotPrinterFriendlyUrl("/doc-wiki-syntax:buttons"),
    "/printer--friendly//doc-wiki-syntax:buttons"
  )
  assert.equal(
    buildWikidotPrinterFriendlyUrl("/open43-issue777-fixture"),
    "/printer--friendly//open43-issue777-fixture"
  )
})

test("printer-friendly loader preserves the required empty route suffix", async () => {
  const source = await readFile(
    new URL("../src/routes/printer--friendly/[...path]/+page.server.ts", import.meta.url),
    "utf8"
  )
  const calls = []
  const load = new Function(
    "loadPage",
    "isReservedAdminAliasSlug",
    "redirect",
    `${source
      .split(/\r?\n/u)
      .filter((line) => !line.startsWith("import "))
      .join("\n")
      .replace("export async function", "async function")}\nreturn load`
  )(
    (...args) => calls.push(args),
    (slug) => slug?.replace(/^\/+|\/+$/gu, "").toLowerCase() === "_admin",
    (status, location) => {
      throw Object.assign(new Error("redirect"), { status, location })
    }
  )
  const request = {}
  const cookies = {}
  const locals = {}
  for (const [path, slug, extra] of [
    ["start", "start", ""],
    ["category:page", "category:page", ""],
    ["category:page/revision/2", "category:page", "revision/2"]
  ]) {
    await load({ params: { path }, request, cookies, locals })
    assert.deepEqual(calls.pop(), [slug, extra, request, cookies, locals])
  }
})

test("print options keep the verified font sizes and single native print control", () => {
  const html = buildWikidotPrintOptionsHtml()
  for (const size of ["6pt", "8pt", "10pt", "12pt", "14pt", "16pt"]) {
    assert.ok(
      html.includes(
        `onclick="WIKIDOT.printview.listeners.changeFontSize(event, '${size}')"`
      ),
      `missing font size ${size}: ${html}`
    )
  }
  assert.ok(
    html.includes(
      '<b><a href="javascript:;" onclick="window.print()">PRINT THE PAGE</a></b>'
    ),
    `missing native print control: ${html}`
  )
  assert.equal((html.match(/window\.print\(\)/gu) ?? []).length, 1)
  assert.ok(html.includes('<div id="print-options">'))
})

test("print source info escapes site and page identity", () => {
  const html = buildWikidotPrintSourceInfoHtml({
    siteName: 'Site <"unsafe">',
    siteUrl: "http://example.test/?a=1&b=2",
    pageTitle: "Page <script>",
    pageUrl: "http://example.test/page&x"
  })
  assert.ok(html.includes('Site &lt;"unsafe"&gt;'), html)
  assert.ok(html.includes("http://example.test/?a=1&amp;b=2"), html)
  assert.ok(html.includes("Page &lt;script&gt;"), html)
  assert.ok(!html.includes("<script>"), html)
})

test("print view delegates activation for generated handlers only", () => {
  const content = { style: {} }
  const listeners = new Map()
  const root = {
    querySelector: (selector) => (selector === "#print-content" ? content : null),
    addEventListener(type, listener) {
      listeners.set(type, listener)
    },
    removeEventListener(type) {
      listeners.delete(type)
    },
    hasListener(type) {
      return listeners.has(type)
    },
    dispatch(element) {
      const preventDefault = () => {}
      listeners.get("click")?.({
        target: {
          closest: (selector) =>
            ({
              'a[onclick="window.print()"]':
                element.onclick === "window.print()" ? element : null,
              'a[onclick^="WIKIDOT.printview.listeners.changeFontSize"]':
                element.onclick?.startsWith("WIKIDOT.printview.listeners.changeFontSize")
                  ? element
                  : null,
              '#print-options a[href="javascript:;"]:not([onclick])':
                // eslint-disable-next-line no-script-url -- Wikidot's observed control contract uses this exact inert href.
                element.href === "javascript:;" && !element.onclick ? element : null
            })[selector] ?? null
        },
        preventDefault
      })
    }
  }

  const printed = []
  const closed = []
  const previousPrint = globalThis.print
  const previousClose = globalThis.close
  globalThis.print = () => printed.push(true)
  globalThis.close = () => closed.push(true)
  try {
    const action = wikidotPrintView(root)
    assert.equal(root.hasListener("click"), true)

    const mountedPrint = { onclick: "window.print()" }
    const latePrint = { onclick: "window.print()" }
    const mountedChange = {
      getAttribute: (name) =>
        ({ onclick: "WIKIDOT.printview.listeners.changeFontSize(event, '12pt')" })[
          name
        ] ?? null,
      onclick: "WIKIDOT.printview.listeners.changeFontSize(event, '12pt')"
    }
    const lateChange = {
      getAttribute: (name) =>
        ({ onclick: "WIKIDOT.printview.listeners.changeFontSize(event, '14pt')" })[
          name
        ] ?? null,
      onclick: "WIKIDOT.printview.listeners.changeFontSize(event, '14pt')"
    }
    const unknownChange = {
      getAttribute: (name) =>
        ({ onclick: "WIKIDOT.printview.listeners.changeFontSize(document, '12pt')" })[
          name
        ] ?? null,
      onclick: "WIKIDOT.printview.listeners.changeFontSize(document, '12pt')"
    }
    const close = {
      // eslint-disable-next-line no-script-url -- Wikidot's observed control contract uses this exact inert href.
      href: "javascript:;"
    }

    root.dispatch(mountedPrint)
    root.dispatch(latePrint)
    root.dispatch(mountedChange)
    root.dispatch(lateChange)
    root.dispatch(unknownChange)
    root.dispatch(close)
    assert.equal(printed.length, 2)
    assert.equal(content.style.fontSize, "14pt")
    assert.equal(closed.length, 1)

    const inert = { onclick: null }
    root.dispatch(inert)
    assert.equal(printed.length, 2)

    action.destroy()
    assert.equal(root.hasListener("click"), false)
  } finally {
    globalThis.print = previousPrint
    globalThis.close = previousClose
  }
})
