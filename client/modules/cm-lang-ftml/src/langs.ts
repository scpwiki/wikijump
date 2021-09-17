import { addLanguages } from "@wikijump/codemirror"
// import { FTMLTokensGrammar, LezerTreeGrammar } from "./grammars/ast"
import { FTMLLanguage } from "./grammars/ftml"

export const ftmlLanguages = addLanguages(
  FTMLLanguage.description
  // FTMLTokensGrammar.description,
  // LezerTreeGrammar.description
)
