import { textValue } from "@wikijump/codemirror"
import { EditorView, linter, type Diagnostic } from "@wikijump/codemirror/cm"
import Locale, { format } from "@wikijump/fluent"
import FTML from "@wikijump/ftml-wasm-worker"

// promise that the linter waits on,
// makes sure that messages are ready before doing anything
const loadingMessages = Locale.load("cmftml")

interface WarningInfo {
  message: string
  severity: "info" | "warning" | "error"
}

// null is an ignored rule
const warningConfig: Record<string, "info" | "warning" | "error" | null> = {
  "recursion-depth-exceeded": "error",
  "end-of-input": null,
  "no-rules-match": null,
  "rule-failed": null,
  "not-start-of-line": null,
  "invalid-include": "error",
  "list-empty": "warning",
  "list-contains-non-item": "error",
  "list-item-outside-list": "error",
  "list-depth-exceeded": "error",
  "table-contains-non-row": "error",
  "table-row-contains-non-cell": "error",
  "table-row-outside-table": "error",
  "table-cell-outside-table": "error",
  "footnotes-nested": "error",
  "blockquote-depth-exceeded": "error",
  "no-such-block": "error",
  "block-disallows-star": "warning",
  "block-disallows-score": "warning",
  "block-missing-name": "error",
  "block-missing-close-brackets": "error",
  "block-malformed-arguments": "error",
  "block-missing-arguments": "error",
  "block-expected-end": "error",
  "block-end-mismatch": "error",
  "no-such-module": "error",
  "module-missing-name": "error",
  "no-such-page": "error",
  "invalid-url": "warning"
}

// generate warnings from configuration
// involves turning SCREAMING_SNAKE_CASE into screaming-snake-case,
// as FTML warnings are kebab case when emitted
const warningInfo: Record<string, WarningInfo | null> = {}
for (const warningName in warningConfig) {
  const type = warningConfig[warningName]
  warningInfo[warningName] = !type
    ? null
    : {
        message: `cmftml-lint.${warningName}`,
        severity: type
      }
}

async function lint(view: EditorView) {
  try {
    await loadingMessages

    const doc = view.state.doc
    const str = await textValue(doc)
    const len = str.length

    const diagnostics: Diagnostic[] = []
    const warnings = await FTML.warnings(str)

    for (const warning of warnings) {
      const { kind, rule, token } = warning
      const { start: from, end: to } = warning.span

      if (from === undefined || to === undefined || to > len) continue
      if (!warningInfo[kind]) continue

      let { message, severity } = warningInfo[kind]!

      // format and translate

      const slice = doc.sliceString(from, to)
      message = format(message, { rule, slice })

      const source = format("cmftml-lint.warning-source", { rule, kind, token, from, to })

      diagnostics.push({ from, to, message, severity, source })
    }

    return diagnostics
  } catch {
    return []
  }
}

export const ftmlLinter = linter(lint)
