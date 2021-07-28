import { stringInput, Tree } from "lezer-tree"

function indent(depth: number, str = "  ") {
  return depth > 0 ? str.repeat(depth) : ""
}

/** Pretty-prints a Lezer tree. Doesn't log the result - just returns it. */
export function printTree(tree: Tree, src: string) {
  const input = stringInput(src)

  let output = ""
  let depth = -1

  tree.iterate({
    enter(type, from, to) {
      depth++

      const len = to - from
      let slice: string
      if (len <= 40) slice = input.read(from, to)
      else slice = `${input.read(from, from + 20)} ... ${input.read(to - 20, to)}`

      slice = slice.replaceAll("\n", "\\n").replaceAll('"', '\\"')

      output += `\n${indent(depth, "│ ")}${type.name} [${from}, ${to}]: "${slice}"`
    },

    leave() {
      depth--
      if (depth === 0) output += "\n│"
    }
  })

  output = output.replace(/\u2502\s*$/, "").trim()

  return output
}
