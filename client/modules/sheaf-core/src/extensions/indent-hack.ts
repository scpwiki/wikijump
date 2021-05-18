/*
 * Exports the "indent-hack" for CodeMirror.
 * This enables elegant, tab-preserving line-wrapping in the editor.
 */

import { RangeSetBuilder } from "@codemirror/rangeset"
import { EditorState } from "@codemirror/state"
import type { Line } from "@codemirror/text"
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate
} from "@codemirror/view"

const WHITESPACE_REGEX = /^\s+/

function indentDeco(view: EditorView) {
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

export const indentHack = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet
    constructor(view: EditorView) {
      this.decorations = indentDeco(view)
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = indentDeco(update.view)
      }
    }
  },
  { decorations: v => v.decorations }
)
