import { assert } from "@esm-bundle/chai"
import type * as DF from "../src/grammar/definition"
import * as lib from "../src/grammar/grammar"

// TODO: a lot more tests here
// there is a better way of testing this lib, which is Lezer's method
// but for right now this is kind of a smoke test

describe("Tarnation Grammar", () => {
  let grammar: lib.Grammar

  it("makes a grammar", () => {
    // wikimath (tex)
    const def: DF.Grammar = {
      start: "root",
      variables: {
        texBrackets: ["(", ")", "[", "]", "{", "}"],
        texSymbols: ["+", "-", "=", "!", "/", "<", ">", "|", "'", ":", "*", "^"]
      },
      brackets: [
        { name: "t.paren", pair: ["(", ")"] },
        { name: "t.brace", pair: ["{", "}"] },
        { name: "t.squareBracket", pair: ["[", "]"] }
      ],
      fallback: ["t.emphasis"],
      states: {
        root: [
          [/%.*$/, "t.comment"],
          [/([a-zA-Z]+)(?=\([^]*?\))/, "Function"],
          [
            /(\\#?[a-zA-Z\d]+)(\{)([^]*?)(\})/,
            "CommandGroup",
            ["Command", "@BR", "t.string", "@BR"]
          ],
          [/\\#?[a-zA-Z\d]+/, "Command"],
          [/\\[,>;!]/, "t.string"],
          [/\\+/, "t.escape"],
          [/\^(?!\d)|[_&]/, "t.keyword"],
          [/\d+/, "t.unit"],
          [/@texSymbols/, "t.operator"],
          [/@texBrackets/, "@BR"]
        ]
      }
    }

    grammar = new lib.Grammar(def)
    assert.instanceOf(grammar, lib.Grammar)
  })

  it("matches", () => {
    const context = { state: "root", context: {} }
    const match = grammar.match(context, "% foobar", 0)!
    assert.deepEqual(match.compile(), [
      {
        type: "comment",
        from: 0,
        to: 8,
        empty: false,
        open: undefined,
        close: undefined,
        next: undefined,
        switchTo: undefined,
        embedded: undefined,
        context: undefined
      }
    ])
  })
})
