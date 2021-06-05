import { drawSelection, EditorView } from "@codemirror/view"
import { IndentHack } from "./indent-hack"
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
