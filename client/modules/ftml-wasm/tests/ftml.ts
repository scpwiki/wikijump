import { assert } from "@esm-bundle/chai"
import pkg from "@root/ftml/Cargo.toml"
import * as lib from "../src/index"

lib.init()

describe("ftml-wasm", () => {
  it("check for out of date", async () => {
    await lib.loading
    const [, thisVersion] = /ftml v([\d.]+).*$/.exec(lib.version())!
    const version = (pkg as any).package.version
    assert.isString(version)
    assert.equal(
      thisVersion,
      version,
      `FTML version was ${version}, expected ${thisVersion}! Run "ftml-pack" in this package's directory.`
    )
  })

  it("version", async () => {
    await lib.loading
    assert.isString(lib.version())
  })

  it("preprocess", async () => {
    await lib.loading
    const str = lib.preprocess(
      "\nApple Banana \\\nCherry\\\nPineapple \\ Grape\nBlueberry\n"
    )
    assert.equal(str, "Apple Banana CherryPineapple \\ Grape\nBlueberry")
  })

  it("tokenize", async () => {
    await lib.loading
    const str = "//1//"
    assert.deepEqual(lib.tokenize(str), [
      { token: "input-start", slice: "", span: { start: 0, end: 0 } },
      { token: "italics", slice: "//", span: { start: 0, end: 2 } },
      { token: "identifier", slice: "1", span: { start: 2, end: 3 } },
      { token: "italics", slice: "//", span: { start: 3, end: 5 } },
      { token: "input-end", slice: "", span: { start: 5, end: 5 } }
    ])
  })

  it("parse", async () => {
    await lib.loading
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

  it("renderHTML", async () => {
    await lib.loading
    const str = "//1//"
    // other returned values are not tested due to their dynamic nature
    // e.g. `meta` depends on version of FTML
    const { html, styles } = lib.renderHTML(str)
    assert.equal(html, "<p><em>1</em></p>")
    assert.equal(styles.join(""), "")
  })

  it("detailRenderHTML", async () => {
    await lib.loading
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

  it("renderText", async () => {
    await lib.loading
    const str = "//1//"
    const text = lib.renderText(str)
    assert.equal(text, "1")
  })

  it("detailRenderText", async () => {
    await lib.loading
    const str = "//1//"
    const render = lib.detailRenderText(str)
    assert.isObject(render.ast)
    assert.isString(render.text)
    assert.isArray(render.tokens)
    assert.isArray(render.warnings)
  })

  it("warnings", async () => {
    await lib.loading
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

  it("inspectTokens", async () => {
    await lib.loading
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
