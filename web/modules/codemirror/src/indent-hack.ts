import { EditorState, RangeSetBuilder, type Line } from "@codemirror/state"
import {
  Decoration,
  ViewPlugin,
  type DecorationSet,
  type EditorView,
  type ViewUpdate
} from "@codemirror/view"

const WHITESPACE_REGEX = /^\s+/

/**
 * Extension that makes it so that lines which wrap onto new lines preserve
 * their indentation. Called a "hack" because this is done through CSS
 * trickery, and not through any sort of complex DOM arrangement.
 */
export const IndentHack = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet
    constructor(view: EditorView) {
      this.decorations = generateIndentDecorations(view)
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = generateIndentDecorations(update.view)
      }
    }
  },
  { decorations: v => v.decorations }
)

function generateIndentDecorations(view: EditorView) {
  // get every line of the visible ranges
  const lines = new Set<Line>()
  for (const { from, to } of view.visibleRanges) {
    for (let pos = from; pos <= to; ) {
      let line = view.state.doc.lineAt(pos)
      lines.add(line)
      pos = line.to + 1
    }
  }

  // get the indentation of every line
  // and create an offset hack decoration if it has any
  const tabInSpaces = " ".repeat(view.state.facet(EditorState.tabSize))
  const builder = new RangeSetBuilder<Decoration>()
  for (const line of lines) {
    const WS = WHITESPACE_REGEX.exec(line.text)?.[0]
    const col = WS?.replaceAll("\t", tabInSpaces).length
    if (col) {
      builder.add(
        line.from,
        line.from,
        Decoration.line({
          attributes: { style: `padding-left: ${col}ch; text-indent: -${col}ch` }
        })
      )
    }
  }

  return builder.finish()
}
