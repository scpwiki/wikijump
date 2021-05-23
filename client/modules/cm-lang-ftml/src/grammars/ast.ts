import { TarnationLanguage } from "cm-tarnation"

export const FTMLTokensGrammar = new TarnationLanguage({
  name: "FTMLTokens",
  // prettier-ignore
  grammar: {
    fallback: ["t.string"],
    brackets: [
      { name: "t.squareBracket", pair: ["[", "]"] }
    ],
    states: {
      root: [
        [/^(\[)(\d+)( <-> )(\d+)(\])(:)(\s*)(\S+)(\s*)(=>)(.*$)/,
          [
            "@BR", "t.integer", "t.operator", "t.integer", "@BR", "t.separator", "",
            "t.content", "", "t.operator", "t.string"
          ]
        ]
      ]
    }
  }
})

export const LezerTreeGrammar = new TarnationLanguage({
  name: "LezerTree",
  // prettier-ignore
  grammar: {
    fallback: ["t.content"],
    brackets: [
      { name: "t.squareBracket", pair: ["[", "]"] }
    ],
    states: {
      root: [
        [/[\u251C\u2514\u2502\u2500]/, "t.punctuation"],
        [/".+"$/, "t.string"],
        [/(\[)(\d+)(\s*,\s*)(\d+)(\])(:?)/, [
          "@BR", "t.integer", "t.punctuation", "t.integer", "@BR", "t.punctuation"
        ]]
      ]
    }
  }
})
