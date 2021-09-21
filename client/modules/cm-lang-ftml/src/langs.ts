import { addLanguages } from "@wikijump/codemirror"
import { FTMLTokensGrammar, PrintTreeGrammar } from "./grammars/ast"
import { FTMLLanguage } from "./grammars/ftml"

export const ftmlLanguages = addLanguages(
  FTMLLanguage.description,
  FTMLTokensGrammar.description,
  PrintTreeGrammar.description
)
