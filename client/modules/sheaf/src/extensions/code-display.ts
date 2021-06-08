import { IndentHack } from "wj-codemirror"
import { drawSelection, EditorView } from "wj-codemirror/cm"
import { confinement } from "./theme"

export function getCodeDisplayExtensions() {
  return [
    drawSelection(),
    EditorView.editable.of(false),
    EditorView.lineWrapping,
    IndentHack,
    confinement
  ]
}
