import { addLanguages } from "@wikijump/codemirror"
import { PrintTreeGrammar } from "./grammars/ast"
// import { FTMLTokensGrammar, LezerTreeGrammar } from "./grammars/ast"
import { FTMLLanguage } from "./grammars/ftml"

export const ftmlLanguages = addLanguages(
  FTMLLanguage.description,
  PrintTreeGrammar.description
  // FTMLTokensGrammar.description,
  // LezerTreeGrammar.description
)
