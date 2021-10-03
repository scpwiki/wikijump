import type { Tree } from "@lezer/common"
import type { Word } from "@wikijump/cm-espells"
import type { EditorState } from "@wikijump/codemirror/cm"

const BLACKLIST_REGEX = [
  /^BlockName/,
  /^BlockNode/,
  /^Include/,
  /^Module/,
  /^Link/,
  /^PageVariable/,
  /^TripleLink/,
  /^SingleLink/
]

const BLACKLIST = [
  "BlockComment",
  "BlockLabel",
  "BlockValue",
  "String",
  "EmbeddedParser",
  "RawEscapeBlock",
  "Escaped",
  "ColoredTextColor"
]

export function spellcheckFTML(state: EditorState, tree: Tree, word: Word) {
  const node = tree.resolve(word.from, 1)
  if (BLACKLIST.includes(node.name)) return false
  if (BLACKLIST_REGEX.some(re => re.test(node.name))) return false
  return true
}
