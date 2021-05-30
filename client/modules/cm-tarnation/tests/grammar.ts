import * as uvu from "uvu"
import * as assert from "uvu/assert"
import type * as DF from "../src/grammar/definition"
import * as lib from "../src/grammar/grammar"

// TODO: a lot more tests here
// there is a better way of testing this lib, which is Lezer's method
// but for right now this is kind of a smoke test

const Grammar = uvu.suite<{ grammar: lib.Grammar }>("Grammar", {} as any)

Grammar("make Grammar", cx => {
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

  const grammar = new lib.Grammar(def)
  assert.instance(grammar, lib.Grammar)
  cx.grammar = grammar
})

Grammar("match Grammar", ({ grammar }) => {
  const context = { state: "root", context: {} }
  const match = grammar.match(context, "% foobar", 0)!
  assert.equal(match.compile(), [
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

Grammar.run()
