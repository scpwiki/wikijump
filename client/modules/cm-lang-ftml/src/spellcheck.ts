import type { Word } from "cm-nspell"
import type { Tree } from "lezer-tree"
import type { EditorState } from "wj-codemirror/cm"

const BLACKLIST = [
  "BlockName",
  "BlockNameUnknown",
  "BlockNameModule",
  "BlockNameInclude",
  "BlockLabel",
  "BlockComment",
  "BlockNodeArgumentValue",
  "BlockNodeArgumentName",
  "IncludeParameterProperty",
  "BlockValue",
  "IncludeValue",
  "ValueName",
  "ModuleName",
  "ModuleNameUnknown",
  "link",
  "color"
]

export function spellcheckFTML(state: EditorState, tree: Tree, word: Word) {
  const node = tree.resolve(word.from, 1)
  if (BLACKLIST.includes(node.name)) return false
  return true
}
