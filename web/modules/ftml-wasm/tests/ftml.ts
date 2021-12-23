import pkg from "@root/ftml/Cargo.toml"
import fs from "fs/promises"
import { assert, describe, it } from "vitest"
import * as lib from "../src/index"

const wasm = await fs.readFile("modules/ftml-wasm/vendor/ftml_bg.wasm")

await lib.init(wasm)

describe("ftml-wasm", () => {
  it("check for out of date", () => {
    const [, thisVersion] = /ftml v([\d.]+).*$/.exec(lib.version())!
    const version = (pkg as any).package.version
    assert.isString(version)
    assert.equal(
      thisVersion,
      version,
      `FTML version was ${version}, expected ${thisVersion}! Run "ftml-pack" in this package's directory.`
    )
  })

  it("version", () => {
    assert.isString(lib.version())
  })

  it("preprocess", () => {
    const str = lib.preprocess(
      "\nApple Banana \\\nCherry\\\nPineapple \\ Grape\nBlueberry\n"
    )
    assert.equal(str, "Apple Banana CherryPineapple \\ Grape\nBlueberry")
  })

  it("tokenize", () => {
    const str = "//1//"
    assert.deepEqual(lib.tokenize(str), [
      { token: "input-start", slice: "", span: { start: 0, end: 0 } },
      { token: "italics", slice: "//", span: { start: 0, end: 2 } },
      { token: "identifier", slice: "1", span: { start: 2, end: 3 } },
      { token: "italics", slice: "//", span: { start: 3, end: 5 } },
      { token: "input-end", slice: "", span: { start: 5, end: 5 } }
    ])
  })

  it("parse", () => {
    const str = "//1//"
    assert.deepEqual(lib.parse(str) as any, {
      "ast": {
        "elements": [
          {
            "element": "container",
            "data": {
              "type": "paragraph",
              "attributes": {},
              "elements": [
                {
                  "element": "container",
                  "data": {
                    "type": "italics",
                    "attributes": {},
                    "elements": [
                      {
                        "element": "text",
                        "data": "1"
                      }
                    ]
                  }
                }
              ]
            }
          },
          {
            "element": "footnote-block",
            "data": {
              "title": null,
              "hide": false
            }
          }
        ],
        "styles": [],
        "table-of-contents": [],
        "footnotes": []
      },
      "warnings": []
    })
  })

  // following two tests are failing, reason appears to be
  // due to getrandom failing. I think this is an issue
  // with how getrandom calls up to the browser for randomness.
  // other tests don't fail, but these do, for some reason.

  it.skip("renderHTML", () => {
    const str = "//1//"
    // other returned values are not tested due to their dynamic nature
    // e.g. `meta` depends on version of FTML
    const { html, styles } = lib.renderHTML(str)
    assert.equal(html, '<wj-body class="wj-body"><p><em>1</em></p></wj-body>')
    assert.equal(styles.join(""), "")
  })

  it.skip("detailRenderHTML", () => {
    const str = "//1//"
    const render = lib.detailRenderHTML(str)
    // considering this function just does what the previous have tested
    // we're only going to do simple type checks
    assert.isObject(render.ast)
    assert.isString(render.html)
    // assert.isObject(render.meta[0])
    assert.isArray(render.styles)
    assert.isArray(render.tokens)
    assert.isArray(render.warnings)
  })

  it("renderText", () => {
    const str = "//1//"
    const text = lib.renderText(str)
    assert.equal(text, "1")
  })

  it("detailRenderText", () => {
    const str = "//1//"
    const render = lib.detailRenderText(str)
    assert.isObject(render.ast)
    assert.isString(render.text)
    assert.isArray(render.tokens)
    assert.isArray(render.warnings)
  })

  it("warnings", () => {
    const str = "[[div]]foo"
    assert.deepEqual(lib.warnings(str), [
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

  it("inspectTokens", () => {
    const str = "//1//"
    const list = lib.inspectTokens(str)
    // prettier-ignore
    assert.equal(list,
`[0000 <-> 0000]: input-start      => ''
[0000 <-> 0002]: italics          => '//'
[0002 <-> 0003]: identifier       => '1'
[0003 <-> 0005]: italics          => '//'
[0005 <-> 0005]: input-end        => ''
`)
  })
})
