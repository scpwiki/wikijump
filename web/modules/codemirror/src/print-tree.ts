import type { Tree } from "@lezer/common"

function indent(depth: number, str = "  ") {
  return depth > 0 ? str.repeat(depth) : ""
}

/** Pretty-prints a Lezer tree. Doesn't log the result - just returns it. */
export function printTree(tree: Tree, src: string) {
  let output = ""
  let depth = -1

  tree.iterate({
    enter(node) {
      const from = node.from
      const to = node.to
      const len = to - from

      depth++

      let slice: string
      if (len <= 40) slice = src.slice(from, to)
      else slice = `${src.slice(from, from + 20)} ... ${src.slice(to - 20, to)}`

      slice = slice.replaceAll("\n", "\\n").replaceAll('"', '\\"')

      output += `\n${indent(depth, "│ ")}${node.name} [${from}, ${to}]: "${slice}"`
    },

    leave() {
      depth--
      if (depth === 0) output += "\n│"
    }
  })

  output = output.replace(/\u2502\s*$/, "").trim()

  return output
}
