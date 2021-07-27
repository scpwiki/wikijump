import toml from "@ltd/j-toml"
import * as fs from "fs/promises"
import * as uvu from "uvu"
import * as assert from "uvu/assert"
import * as lib from "../src/index"

const wasm = fs.readFile("modules/ftml-wasm/vendor/ftml_bg.wasm")
lib.init(wasm as any)

const FTML = uvu.suite("ftml-wasm")

FTML("check for out of date", async () => {
  await lib.loading
  const thisVersionRaw = lib.version()
  const [, thisVersion] = /ftml v([\d.]+).*$/.exec(thisVersionRaw)!
  const file = await fs.readFile("../ftml/Cargo.toml", { encoding: "utf-8" })
  const pkg = toml.parse(file, 1.0, "\n", false, { order: true, null: true }) as any
  const version = pkg.package.version
  assert.type(version, "string")
  assert.is(
    thisVersion,
    version,
    `FTML version was ${version}, expected ${thisVersion}! Run "ftml-pack" in this package's directory.`
  )
})

FTML("version", async () => {
  await lib.loading
  assert.type(lib.version(), "string")
})

FTML("preprocess", async () => {
  await lib.loading
  const str = lib.preprocess(
    "\nApple Banana \\\nCherry\\\nPineapple \\ Grape\nBlueberry\n"
  )
  assert.is(str, "Apple Banana CherryPineapple \\ Grape\nBlueberry")
})

FTML("tokenize", async () => {
  await lib.loading
  const str = "//1//"
  assert.equal(lib.tokenize(str), [
    { token: "input-start", slice: "", span: { start: 0, end: 0 } },
    { token: "italics", slice: "//", span: { start: 0, end: 2 } },
    { token: "identifier", slice: "1", span: { start: 2, end: 3 } },
    { token: "italics", slice: "//", span: { start: 3, end: 5 } },
    { token: "input-end", slice: "", span: { start: 5, end: 5 } }
  ])
})

FTML("parse", async () => {
  await lib.loading
  const str = "//1//"
  assert.equal(lib.parse(str), {
    "ast": {
      "elements": [
        {
          "element": "container",
          "data": {
            "type": "paragraph",
            "elements": [
              {
                "element": "container",
                "data": {
                  "type": "italics",
                  "elements": [
                    {
                      "element": "text",
                      "data": "1"
                    }
                  ],
                  "attributes": {}
                }
              }
            ],
            "attributes": {}
          }
        }
      ],
      "styles": []
    },
    "warnings": []
  })
})

FTML("render html", async () => {
  await lib.loading
  const str = "//1//"
  // we won't test the `meta` property because
  // that property is a bit too dynamic to easily test. (it has version info in it)
  const { html, styles } = lib.render(str)

  assert.is(html, "<p><em>1</em></p>")
  assert.is(styles.join(""), "")

  // assert.equal(lib.render(str), {
  //   html: "<p><em>1</em></p>",
  //   meta: [
  //     {
  //       tag_type: "http-equiv",
  //       name: "Content-Type",
  //       value: "text/html"
  //     },
  //     { tag_type: "name", name: "generator", value: "ftml 0.7.0" },
  //     { tag_type: "name", name: "description", value: "" },
  //     { tag_type: "name", name: "keywords", value: "" }
  //   ],
  //   style: ""
  // })
})

FTML("render text", async () => {
  await lib.loading
  const str = "//1//"
  const text = lib.render(str, { mode: "text" })
  assert.is(text, "1")
})

FTML("warnings", async () => {
  await lib.loading
  const str = "[[div]]foo"
  assert.equal(lib.warnings(str), [
    {
      token: "input-end",
      rule: "block-div",
      span: { start: 10, end: 10 },
      kind: "end-of-input"
    },
    {
      token: "left-block",
      rule: "fallback",
      span: { start: 0, end: 2 },
      kind: "no-rules-match"
    },
    {
      token: "right-block",
      rule: "fallback",
      span: { start: 5, end: 7 },
      kind: "no-rules-match"
    }
  ])
})

FTML("detailedRender", async () => {
  await lib.loading
  const str = "//1//"
  const render = lib.detailedRender(str)
  // considering this function just does what the previous have tested
  // we're only going to do simple type checks
  assert.type(render.ast.elements, "object")
  assert.type(render.html, "string")
  // assert.type(render.meta[0], "object")
  assert.type(render.preprocessed, "string")
  assert.type(render.styles, "object")
  assert.type(render.tokens, "object")
  assert.type(render.warnings, "object")
})

FTML("inspectTokens", async () => {
  await lib.loading
  const str = "//1//"
  const list = lib.inspectTokens(str)
  // prettier-ignore
  assert.is(list,
`[0000 <-> 0000]: input-start      => ''
[0000 <-> 0002]: italics          => '//'
[0002 <-> 0003]: identifier       => '1'
[0003 <-> 0005]: italics          => '//'
[0005 <-> 0005]: input-end        => ''
`)
})

FTML.run()
