import { closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete"
import {
  copyLineDown,
  defaultKeymap,
  historyKeymap,
  indentWithTab,
  redo
} from "@codemirror/commands"
import { foldKeymap } from "@codemirror/language"
import { nextDiagnostic, openLintPanel } from "@codemirror/lint"
import { searchKeymap } from "@codemirror/search"
import { keymap } from "@codemirror/view"

/** Additional key bindings for the editor. */
const KEY_MAP = [
  { key: "Mod-l", run: openLintPanel, preventDefault: true },
  { key: "F8", run: nextDiagnostic, preventDefault: true },
  { key: "Mod-Shift-z", run: redo, preventDefault: true },
  { key: "Mod-d", run: copyLineDown, preventDefault: true }
]

/** Returns an extension for Sheaf's full keybinding set. */
export function getSheafKeymap() {
  return keymap.of([
    ...defaultKeymap,
    ...closeBracketsKeymap,
    ...searchKeymap,
    ...historyKeymap,
    ...foldKeymap,
    ...completionKeymap,
    ...KEY_MAP,
    indentWithTab
  ])
}
