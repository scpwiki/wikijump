import { Input, NodeType, stringInput, Tree, TreeCursor } from "lezer-tree"

// credit to: https://discuss.codemirror.net/u/grayen/summary
function focusedNode(
  cursor: TreeCursor
): { readonly type: NodeType; readonly from: number; readonly to: number } {
  const { type, from, to } = cursor
  return { type, from, to }
}
// credit to: https://discuss.codemirror.net/u/grayen/summary
export function printTree(
  tree: Tree,
  input: Input | string,
  options: { from?: number; to?: number; start?: number; includeParents?: boolean } = {}
): string {
  const cursor = tree.cursor()
  if (typeof input === "string") input = stringInput(input)
  const { from = 0, to = input.length, start = 0, includeParents = false } = options
  let output = ""
  const prefixes: string[] = []
  for (;;) {
    const node = focusedNode(cursor)
    let leave = false
    if (node.from <= to && node.to >= from) {
      const enter = includeParents || (node.from >= from && node.to <= to)
      if (enter) {
        leave = true
        const isTop = output === ""
        if (!isTop || node.from > 0) {
          output += (!isTop ? "\n" : "") + prefixes.join("")
          const hasNextSibling = cursor.nextSibling() && cursor.prevSibling()
          if (hasNextSibling) {
            output += " ├─ "
            prefixes.push(" │  ")
          } else {
            output += " └─ "
            prefixes.push("    ")
          }
        }
        output += node.type.isAnonymous ? "<Anonymous>" : node.type.name
      }
      const isLeaf = !cursor.firstChild()
      if (enter) {
        const hasRange = node.from !== node.to
        output += ` ${
          hasRange ? `[${start + node.from}..${start + node.to}]` : start + node.from
        }`
        if (hasRange && isLeaf) {
          const str = input.read(node.from, node.to).trim().replaceAll("\n", "\\n")
          output += `: '${str.length > 30 ? `${str.slice(0, 27)}...` : str}'`
        }
      }
      if (!isLeaf) continue
    }
    for (;;) {
      if (leave) prefixes.pop()
      leave = cursor.type.isAnonymous
      if (cursor.nextSibling()) break
      if (!cursor.parent()) return output
      leave = true
    }
  }
}
