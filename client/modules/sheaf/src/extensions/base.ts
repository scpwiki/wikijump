import {
  autocompletion,
  bracketMatching,
  closeBrackets,
  drawSelection,
  EditorState,
  EditorView,
  highlightActiveLine,
  highlightSelectionMatches,
  highlightSpecialChars,
  history,
  indentOnInput,
  rectangularSelection
} from "wj-codemirror/cm"

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
