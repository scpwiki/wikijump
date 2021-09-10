import * as fse from "fs-extra"
import * as fs from "fs/promises"
import { sync as globby } from "globby"
import * as path from "path"
import { performance } from "perf_hooks"
import * as uvu from "uvu"
import * as assert from "uvu/assert"
import * as lib from "../src/index"
import type { IPageInfo, IParseWarning, ISyntaxTree } from "../vendor/ftml"

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

const TEST_DIR = path.resolve("../ftml/test/")

const wasm = fs.readFile("modules/ftml-wasm/vendor/ftml_bg.wasm")
lib.init(wasm as any)

const FTML = uvu.suite("ftml-wasm")

interface TestInput {
  input: string
  tree: ISyntaxTree
  warnings: IParseWarning[]
}

let numTests = 0
let lastTest: string

function assembleTests() {
  const inputs = globby("*.json", { cwd: TEST_DIR, absolute: true })

  for (const file of inputs) {
    const name = path.basename(file, ".json")

    if (SKIP.some(str => name.startsWith(str))) continue

    numTests++

    FTML(name, async () => {
      await lib.loading

      let src = await fs.readFile(file, "utf-8")

      // only need this if we're testing warnings, which we aren't (see below)
      // src = transformRanges(src)

      const test: TestInput = JSON.parse(src)

      const pathHTML = file.replace(/\.json$/, ".html")
      const pathTXT = file.replace(/\.json$/, ".txt")

      const hasHTML = await fileExists(pathHTML)
      const hasTXT = await fileExists(pathTXT)

      if (hasHTML || hasTXT) {
        const info = { ...PAGE_INFO, title: name, page: `page-${name}` }

        const input = lib.preprocess(test.input)

        const result = lib.detailRenderHTML(input, info)

        if (hasHTML) {
          const expected = await fs.readFile(pathHTML, "utf-8")
          assert.fixture(clean(result.html), clean(expected))
        }

        if (hasTXT) {
          const expected = await fs.readFile(pathTXT, "utf-8")
          const text = lib.renderText(input, info)
          assert.fixture(clean(text), clean(expected))
        }

        assert.equal(result.ast, test.tree)

        // for now we won't compare warnings because they get UTF16 mapped for JS
        // this could get corrected later but for right now it's fine
        // assert.equal(result.warnings, test.warnings)

        lastTest = name
      } else {
        throw new Error(`Test '${name}' has no HTML or TXTs!`)
      }
    })
  }
}

assembleTests()

// measure overall performance
let start: number
FTML.before(() => void (start = performance.now()))
FTML.after(() => {
  if (!start || !numTests) return
  const time = parseFloat((performance.now() - start).toFixed(2))
  console.log(`\n\nFTML version: ${lib.version()}`)
  console.log(`Executed ${numTests} tests in ${time}ms.`)
})

// measure individual test performance
const testPerf: [name: string, time: number][] = []
let testStart: number
FTML.before.each(() => void (testStart = performance.now()))
FTML.after.each(() => {
  if (!lastTest || !testStart) return
  const time = parseFloat((performance.now() - testStart).toFixed(2))
  testPerf.push([lastTest, time])
})

// print out what the slowest tests were
FTML.after(() => {
  if (testPerf.length === 0) return
  testPerf.sort(([, a], [, b]) => b - a)
  // always ignore the slowest, because it'll just be the first test executed
  const worst = testPerf.slice(1, 11)
  console.log("Slowest:")
  for (const [name, time] of worst) {
    let str = String(time)
    // check for no decimal point
    if (/^\d+$/.test(str)) str += ".00"
    const padded = `${str.padEnd(4, "0").padStart(5)}ms`.padEnd(8)
    console.log(`${padded} ${name}`)
  }
})

function fileExists(path: string) {
  return fse
    .access(path)
    .then(() => true)
    .catch(() => false)
}

function clean(str: string) {
  return str.replaceAll(/\r\n/g, "\n").trim()
}

function transformRanges(str: string) {
  return str.replaceAll(
    /"span": \[(\d+), (\d+)\],/g,
    '"span": { "start": $1, "end": $2 },'
  )
}

FTML.run()
