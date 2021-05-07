import * as uvu from "uvu"
import * as assert from "uvu/assert"
import * as fse from "fs-extra"
import * as fs from "fs/promises"
import * as path from "path"
import { sync as globby } from "globby"

import * as lib from "../src/index"
import type { IPageInfo, IParseWarning, ISyntaxTree } from "../vendor/ftml"

const wasm = fs.readFile("modules/ftml-wasm/vendor/ftml_bg.wasm")
lib.init(wasm as any)

/** Test categories to skip, e.g. "image" or "include". */
const SKIP: string[] = ["include"]

const PAGE_INFO: IPageInfo = {
  alt_title: null,
  category: null,
  locale: "en_US",
  rating: 0,
  page: "unknown",
  site: "test",
  tags: ["fruit", "component"],
  title: ""
}

const FTML = uvu.suite("ftml-wasm")

interface TestInput {
  input: string
  tree: ISyntaxTree
  warnings: IParseWarning[]
}

function fileExists(path: string) {
  return fse
    .access(path)
    .then(() => true)
    .catch(() => false)
}

function clean(str: string) {
  return str.replaceAll(/\r\n/g, "\n").trim()
}

const TEST_DIR = path.resolve("../ftml/test/")

function assembleTests() {
  const inputs = globby("*.json", { cwd: TEST_DIR, absolute: true })

  for (const file of inputs) {
    const name = path.basename(file, ".json")

    if (SKIP.some(str => name.startsWith(str))) continue

    FTML(name, async () => {
      await lib.loading
      const src = await fs.readFile(file, "utf-8")
      const test: TestInput = JSON.parse(src)

      assert.ok(test, `Invalid test: '${name}'`)
      const { input, tree, warnings } = test

      const pathHTML = file.replace(/\.json$/, ".html")
      const pathTXT = file.replace(/\.json$/, ".txt")

      const hasHTML = await fileExists(pathHTML)
      const hasTXT = await fileExists(pathTXT)

      if (!hasHTML && !hasTXT) throw new Error(`Test '${name}' has no HTML or TXTs!`)

      const info = { ...PAGE_INFO, title: name, page: `page-${name}` }

      if (hasHTML) {
        const expected = await fs.readFile(pathHTML, "utf-8")
        const { html, meta, style } = await lib.render(input, { info })
        assert.fixture(clean(html), clean(expected))
      }

      if (hasTXT) {
        const expected = await fs.readFile(pathTXT, "utf-8")
        const text = await lib.render(input, { mode: "text", info })
        assert.fixture(clean(text), clean(expected))
      }
    })
  }
}

assembleTests()

FTML.run()
