import { TarnationLanguage } from "@wikijump/cm-tarnation"
import printTreeGrammar from "./print-tree.yaml"
import tokensGrammar from "./tokens.yaml"

export const FTMLTokensGrammar = new TarnationLanguage({
  name: "FTMLTokens",
  grammar: tokensGrammar as any
})

export const PrintTreeGrammar = new TarnationLanguage({
  name: "PrintTree",
  grammar: printTreeGrammar as any
})
