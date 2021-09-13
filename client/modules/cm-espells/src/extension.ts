import { EditorField, underline } from "@wikijump/codemirror"
import { EditorView, hoverTooltip } from "@wikijump/codemirror/cm"
import espells from "./espells"
import { misspelledTooltip } from "./hover"
import { spellcheckLinterPlugin } from "./linter"
import { SpellcheckState } from "./state"

/** Theme for the spellchecker extension. */
const theme = EditorView.baseTheme({
  ".cm-spellcheckRange": {
    backgroundPosition: "left bottom",
    backgroundRepeat: "repeat-x"
  },

  ".cm-spellcheckRange-misspelled": { backgroundImage: underline("#d11") },
  ".cm-spellcheckRange-forbidden": { backgroundImage: underline("orange") },
  ".cm-spellcheckRange-warn": { backgroundImage: underline("orange") }
})

/**
 * Extension, more specifically a field, that adds spellchecking to a
 * CodeMirror instance. The spellchecker requires that a language provide
 * the `spellcheck` property within its `languageData` object in order for
 * it to be spellchecked. The value of this property needs to be a
 * {@link SpellcheckFilter} function.
 */
export const Spellcheck = new EditorField<SpellcheckState>({
  default: new SpellcheckState(true, espells.locale),

  // maps the decorations across document changes
  update: (state, tr, changed) => {
    if (!tr.docChanged || changed) return
    const mapped = state.flagged.map(tr.changes)
    return state.set(mapped)
  },

  provide: field => EditorView.decorations.from(field, state => state.flagged),

  reconfigure: (state, last) => {
    if (last && last.enabled === state.enabled && last.locale === state.locale) {
      return false
    }
    if (!state.enabled) return null
    espells.set(state.locale)
    return [
      spellcheckLinterPlugin,
      hoverTooltip(misspelledTooltip, { hideOnChange: true }),
      theme
    ]
  }
})
