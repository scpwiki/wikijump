import type { Tree } from "@lezer/common"
import type { Word } from "@wikijump/cm-espells"
import type { EditorState } from "@wikijump/codemirror/cm"

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
