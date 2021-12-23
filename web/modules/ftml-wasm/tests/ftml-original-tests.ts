import fs from "fs/promises"
import { assert, describe, it } from "vitest"
import * as lib from "../src/index"
import type { IPageInfo } from "../vendor/ftml"

const wasm = await fs.readFile("modules/ftml-wasm/vendor/ftml_bg.wasm")

await lib.init(wasm)

const jsons = import.meta.globEager("/../ftml/test/*.json")
const htmls = import.meta.globEager("/../ftml/test/*.html?raw")
const txts = import.meta.globEager("/../ftml/test/*.txt?raw")

/** Test categories to skip, e.g. "image" or "include". */
const SKIP: string[] = ["include"]

const PAGE_INFO: IPageInfo = {
  alt_title: null,
  category: null,
  language: "en",
  rating: 0,
  page: "unknown",
  site: "test",
  tags: ["fruit", "component"],
  title: ""
}

describe("ftml-tests", () => {
  for (const path in jsons) {
    const name = path
      .replace(/\.json$/, "")
      .split(/\/\\/)
      .pop()!

    if (SKIP.some(str => name.startsWith(str))) continue

    const json = jsons[path].default
    const html = htmls[path.replace(/\.json$/, ".html")]?.default
    const txt = txts[path.replace(/\.json$/, ".txt")]?.default

    it(name, () => {
      if (html || txt) {
        const info = { ...PAGE_INFO, title: name, page: `page-${name}` }

        const input = lib.preprocess(json.input)

        const result = lib.detailRenderHTML(input, info)

        if (html) {
          assert.equal(clean(result.html), clean(html))
        }

        if (txt) {
          const text = lib.renderText(input, info)
          assert.equal(clean(text), clean(txt))
        }

        assert.equal(result.ast, json.tree)
      }
    })
  }
})

// TODO: add this back in, perhaps as its own script

// let numTests = jsons.length
// let lastTest: string

// // measure overall performance
// let start: number
// FTML.before(() => void (start = performance.now()))
// FTML.after(() => {
//   if (!start || !numTests) return
//   const time = parseFloat((performance.now() - start).toFixed(2))
//   console.log(`\n\nFTML version: ${lib.version()}`)
//   console.log(`Executed ${numTests} tests in ${time}ms.`)
// })

// // measure individual test performance
// const testPerf: [name: string, time: number][] = []
// let testStart: number
// FTML.before.each(() => void (testStart = performance.now()))
// FTML.after.each(() => {
//   if (!lastTest || !testStart) return
//   const time = parseFloat((performance.now() - testStart).toFixed(2))
//   testPerf.push([lastTest, time])
// })

// // print out what the slowest tests were
// FTML.after(() => {
//   if (testPerf.length === 0) return
//   testPerf.sort(([, a], [, b]) => b - a)
//   // always ignore the slowest, because it'll just be the first test executed
//   const worst = testPerf.slice(1, 11)
//   console.log("Slowest:")
//   for (const [name, time] of worst) {
//     let str = String(time)
//     // check for no decimal point
//     if (/^\d+$/.test(str)) str += ".00"
//     const padded = `${str.padEnd(4, "0").padStart(5)}ms`.padEnd(8)
//     console.log(`${padded} ${name}`)
//   }
// })

function clean(str: string) {
  return str.replaceAll(/\r\n/g, "\n").trim()
}
