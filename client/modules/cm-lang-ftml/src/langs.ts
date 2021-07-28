import { languages as originalLanguages } from "wj-codemirror/cm"
import { FTMLTokensGrammar, LezerTreeGrammar } from "./grammars/ast"
import { FTMLLanguage } from "./grammars/ftml"

export const languages = [
  ...originalLanguages,
  FTMLLanguage.description,
  FTMLTokensGrammar.description,
  LezerTreeGrammar.description
]
