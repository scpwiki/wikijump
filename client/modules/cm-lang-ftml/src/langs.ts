import { languages as originalLanguages } from "@codemirror/language-data"
import { FTMLTokensGrammar, LezerTreeGrammar } from "./grammars/ast"
import { FTMLLanguage } from "./grammars/ftml"

export const languages = [
  ...originalLanguages,
  FTMLLanguage.description,
  FTMLTokensGrammar.description,
  LezerTreeGrammar.description
]
