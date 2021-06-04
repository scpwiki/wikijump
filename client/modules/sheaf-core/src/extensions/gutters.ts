import { foldGutter } from "@codemirror/fold"
import { lineNumbers } from "@codemirror/gutter"
import { EditorField } from "../util/editor-field"

/**
 * `EditorField` extension that enables a field that controls whether or
 * not the editor gutter is mounted.
 */
export const gutters = new EditorField<boolean>({
  default: true,
  reconfigure: state => (state ? [lineNumbers(), foldGutter()] : null)
})
