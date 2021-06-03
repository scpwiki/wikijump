import { Action, Diagnostic, linter } from "@codemirror/lint"
import type { EditorView } from "@codemirror/view"
import FTML from "ftml-wasm-worker"
import { format } from "wj-state"
import { Content } from "."

interface WarningInfo {
  message: string
  severity: "info" | "warning" | "error"
}

// null is an ignored rule
const warningConfig: Record<string, "info" | "warning" | "error" | null> = {
  RECURSION_DEPTH_EXCEEDED: "error",
  END_OF_INPUT: null,
  NOT_IMPLEMENTED: "warning",
  NO_RULES_MATCH: null,
  RULE_FAILED: null,
  INVALID_INCLUDE: "error",
  LIST_DEPTH_EXCEEDED: "error",
  BLOCKQUOTE_DEPTH_EXCEEDED: "error",
  NO_SUCH_BLOCK: "error",
  BLOCK_DISALLOWS_STAR: "warning",
  BLOCK_DISALLOWS_SCORE: "warning",
  BLOCK_MISSING_NAME: "error",
  BLOCK_MISSING_CLOSE_BRACKETS: "error",
  BLOCK_MALFORMED_ARGUMENTS: "error",
  BLOCK_MISSING_ARGUMENTS: "error",
  BLOCK_EXPECTED_END: "error",
  BLOCK_END_MISMATCH: "error",
  NO_SUCH_MODULE: "error",
  MODULE_MISSING_NAME: "error",
  INVALID_URL: "warning"
}

// generate warnings from configuration
// involves turning SCREAMING_SNAKE_CASE into screaming-snake-case,
// as FTML warnings are kebab case when emitted
const warningInfo: Record<string, WarningInfo | null> = {}
for (const warningName in warningConfig) {
  const type = warningConfig[warningName as keyof typeof warningConfig]
  const warningNameKebabed = warningName.toLowerCase().replaceAll("_", "-")
  if (!type) {
    warningInfo[warningNameKebabed] = null
  } else {
    warningInfo[warningNameKebabed] = {
      message: `cmftml.lint.${warningName}`,
      severity: type
    }
  }
}

async function lint(view: EditorView) {
  try {
    const doc = view.state.doc
    const str = doc.toString()
    const content = await Content.extract(str)
    const len = str.length

    const diagnostics: Diagnostic[] = []

    // do spellcheck and ftml warnings in parallel
    await Promise.all([
      // process FTML warnings
      FTML.warnings(str).then(warnings => {
        for (const warning of warnings) {
          const { kind, rule, token } = warning
          const { start: from, end: to } = warning.span

          if (from === undefined || to === undefined || to > len) continue
          if (!warningInfo[kind]) continue

          let { message, severity } = warningInfo[kind]!

          // format and translate

          const slice = doc.sliceString(from, to)
          message = format(message, { values: { rule, slice } })

          const source = format("cmftml.lint.WARNING_SOURCE", {
            values: { rule, kind, token, from, to }
          })

          diagnostics.push({ from, to, message, severity, source })
        }
      }),
      // spellcheck
      Content.spellcheckWords(content).then(misspellings => {
        if (!misspellings) return
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
              Content.saveToDictionary(slice)
              // reinsert the same word back so that the document gets updated still
              view.dispatch({ changes: { from, to, insert: word } })
            }
          })

          diagnostics.push({ from, to, message, severity: "error", source, actions })
        }
      })
    ])

    // diagnostics have to be in order
    diagnostics.sort((a, b) => a.from - b.from)

    return diagnostics
  } catch {
    return []
  }
}

export const ftmlLinter = linter(lint)
