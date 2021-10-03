import { foldGutter, lineNumbers } from "../cm"
import { EditorField } from "./editor-field"

/**
 * `EditorField` extension that enables a field that controls whether or
 * not the editor gutter is mounted.
 */
export const Gutters = new EditorField<boolean>({
  default: true,
  reconfigure: state => (state ? [lineNumbers(), foldGutter()] : null)
})
