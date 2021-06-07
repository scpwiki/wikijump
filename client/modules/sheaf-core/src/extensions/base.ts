import { autocompletion } from "@codemirror/autocomplete"
import { closeBrackets } from "@codemirror/closebrackets"
import { history } from "@codemirror/history"
import { indentOnInput } from "@codemirror/language"
import { bracketMatching } from "@codemirror/matchbrackets"
import { rectangularSelection } from "@codemirror/rectangular-selection"
import { highlightSelectionMatches } from "@codemirror/search"
import { EditorState } from "@codemirror/state"
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars
} from "@codemirror/view"

export function getSheafBasicExtensions() {
  return [
    highlightSpecialChars(),
    history(),
    drawSelection(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    highlightSelectionMatches(),
    autocompletion(),
    rectangularSelection(),
    highlightActiveLine(),
    EditorView.lineWrapping
  ]
}
