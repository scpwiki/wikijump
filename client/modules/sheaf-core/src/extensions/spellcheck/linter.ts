import { Action, Diagnostic, linter } from "@codemirror/lint"
import type { EditorView } from "@codemirror/view"
import { format } from "wj-state"
import { Spellchecker } from "."
import { ContentFacet } from "../content"

async function lint(view: EditorView) {
  try {
    const extract = view.state.facet(ContentFacet)
    const content = await extract(view.state, true)

    const diagnostics: Diagnostic[] = []
    const misspellings = await Spellchecker.spellcheckWords(content)

    if (misspellings) {
      for (const { word, from, to, suggestions } of misspellings) {
        const slice = word
        const message = format("cmftml.lint.MISSPELLED_WORD", { values: { slice } })
        const source = format("cmftml.lint.SPELLCHECKER_SOURCE", {
          values: { slice, from, to }
        })

        const actions: Action[] = suggestions.map(suggestion => ({
          name: suggestion.term,
          apply(view, from, to) {
            view.dispatch({ changes: { from, to, insert: suggestion.term } })
          }
        }))

        actions.push({
          name: format("cmftml.lint.SPELLCHECKER_ADD_TO_DICTIONARY", {
            values: { slice }
          }),
          apply(view, from, to) {
            Spellchecker.saveToDictionary(slice)
            // reinsert the same word back so that the document gets updated still
            view.dispatch({ changes: { from, to, insert: word } })
          }
        })

        diagnostics.push({ from, to, message, severity: "error", source, actions })
      }
    }

    return diagnostics
  } catch {
    return []
  }
}

export const spellcheckLinter = linter(lint, { delay: 250 })
