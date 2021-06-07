import { EditorView, hoverTooltip } from "wj-codemirror/cm"
import { EditorField } from "../../util/editor-field"
import { misspelledTooltip } from "./hover"
import { spellcheckLinter } from "./linter"
import { Spellchecker } from "./spellchecker"
import { SpellcheckState } from "./state"

// this is apparently how CodeMirror does underlines,
// I figured it was just a text underline, but no, it's actually this
// kind of interesting
function underline(color: string) {
  if (typeof btoa !== "function") return "none"
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="6" height="3">
    <path d="m0 3 l2 -2 l1 0 l2 2 l1 0" stroke="${color}" fill="none" stroke-width=".7"/>
  </svg>`
  return `url('data:image/svg+xml;base64,${btoa(svg)}')`
}

/** Theme for the spellchecker extension. */
const theme = EditorView.baseTheme({
  ".cm-misspellingRange": {
    backgroundPosition: "left bottom",
    backgroundRepeat: "repeat-x",
    backgroundImage: underline("#d11")
  }
})

/**
 * Extension, more specifically a field, that adds spellchecking to a
 * CodeMirror instance. The spellchecker requires the `ContentFacet` facet
 * to have a value within the instance, as the spellchecker by itself has
 * no way of knowing what it should actually be checking.
 */
export const Spellcheck = new EditorField<SpellcheckState>({
  default: new SpellcheckState(true, Spellchecker.locale),

  // maps the decorations across document changes
  update: (state, tr, changed) => {
    if (!tr.docChanged || changed) return
    const mapped = state.misspellings.map(tr.changes)
    return state.set(mapped)
  },

  provide: field => EditorView.decorations.from(field, state => state.misspellings),

  reconfigure: (state, last) => {
    if (last && last.enabled === state.enabled && last.locale === state.locale) {
      return false
    }
    if (!state.enabled) return null
    Spellchecker.setSpellchecker(state.locale)
    return [
      spellcheckLinter,
      hoverTooltip(misspelledTooltip, { hideOnChange: true }),
      theme
    ]
  }
})
