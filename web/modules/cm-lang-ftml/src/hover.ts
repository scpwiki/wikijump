import type { EditorView } from "@wikijump/codemirror/cm"
import { ensureSyntaxTree, hoverTooltip, Tooltip } from "@wikijump/codemirror/cm"
import { BlockMap } from "./data/data"

function tooltip(view: EditorView, pos: number, side: -1 | 1): Tooltip | null {
  const tree = ensureSyntaxTree(view.state, pos)
  if (tree) {
    const node = tree.resolve(pos, side)
    const slice = view.state.doc.sliceString(node.from, node.to).toLowerCase()
    if (node && node.name === "BlockName" && BlockMap.has(slice)) {
      // isn't safe to reuse the DOM node, unfortunately, so we have to clone it
      const instance = BlockMap.get(slice)!.tip.clone()
      // set a class on the instance that we can style, as otherwise there isn't a decent
      // way to target the tooltip without affecting something else
      instance.dom.classList.add("cm-ftml-hover-tip")
      return {
        pos: node.from,
        end: node.to,
        create: _view => ({
          dom: instance.dom
        })
      }
    }
  }
  return null
}

export const ftmlHoverTooltips = hoverTooltip(tooltip)
