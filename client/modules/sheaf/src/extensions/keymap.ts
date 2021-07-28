import {
  closeBracketsKeymap,
  commentKeymap,
  completionKeymap,
  copyLineDown,
  defaultKeymap,
  defaultTabBinding,
  foldKeymap,
  historyKeymap,
  KeyBinding,
  keymap,
  nextDiagnostic,
  openLintPanel,
  redo,
  searchKeymap
} from "wj-codemirror/cm"

/** Additional key bindings for the editor. */
const KEY_MAP: KeyBinding[] = [
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
    ...commentKeymap,
    ...completionKeymap,
    ...KEY_MAP,
    defaultTabBinding
  ])
}
