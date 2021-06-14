import { EditorField, underline } from "wj-codemirror"
import { EditorView, hoverTooltip } from "wj-codemirror/cm"
import { misspelledTooltip } from "./hover"
import { spellcheckLinterPlugin } from "./linter"
import nspell from "./nspell"
import { SpellcheckState } from "./state"

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
 * CodeMirror instance. The spellchecker requires that a language provide
 * the `spellcheck` property within its `languageData` object in order for
 * it to be spellchecked. The value of this property needs to be a
 * {@link SpellcheckFilter} function.
 */
export const Spellcheck = new EditorField<SpellcheckState>({
  default: new SpellcheckState(true, nspell.locale),

  // maps the decorations across document changes
  update: (state, tr, changed) => {
    if (!tr.docChanged || changed) return
    const mapped = state.words.map(tr.changes)
    return state.set(mapped)
  },

  provide: field => EditorView.decorations.from(field, state => state.words),

  reconfigure: (state, last) => {
    if (last && last.enabled === state.enabled && last.locale === state.locale) {
      return false
    }
    if (!state.enabled) return null
    nspell.set(state.locale)
    return [
      spellcheckLinterPlugin,
      hoverTooltip(misspelledTooltip, { hideOnChange: true }),
      theme
    ]
  }
})
