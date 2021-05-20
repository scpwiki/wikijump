import { Diagnostic, linter } from "@codemirror/lint"
import type { EditorView } from "@codemirror/view"
import { warnings } from "ftml-wasm-worker"
import { format } from "wj-state"

interface WarningInfo {
  message: string
  severity: "info" | "warning" | "error"
}

const warningInfo: Record<string, WarningInfo | null> = {
  // ignored warnings
  "no-rules-match": null,
  "end-of-input": null,
  "rule-failed": null,

  "recursion-depth-exceeded": {
    message: "cmftml.lint.RECURSION_DEPTH_EXCEEDED",
    severity: "error"
  },

  "not-implemented": {
    message: "cmftml.lint.NOT_IMPLEMENTED",
    severity: "warning"
  },

  "invalid-include": {
    message: "cmftml.lint.INVALID_INCLUDE",
    severity: "error"
  },

  "list-depth-exceeded": {
    message: "cmftml.lint.LIST_DEPTH_EXCEEDED",
    severity: "error"
  },

  "blockquote-depth-exceeded": {
    message: "cmftml.lint.BLOCKQUOTE_DEPTH_EXCEEDED",
    severity: "error"
  },

  "no-such-block": {
    message: "cmftml.lint.NO_SUCH_BLOCK",
    severity: "error"
  },

  "invalid-special-block": {
    message: "cmftml.lint.INVALID_SPECIAL_BLOCK",
    severity: "warning"
  },

  "block-missing-name": {
    message: "cmftml.lint.BLOCK_MISSING_NAME",
    severity: "error"
  },

  "block-missing-close-brackets": {
    message: "cmftml.lint.BLOCK_MISSING_CLOSE_BRACKETS",
    severity: "error"
  },

  "block-malformed-arguments": {
    message: "cmftml.lint.BLOCK_MALFORMED_ARGUMENTS",
    severity: "error"
  },

  "block-expected-end": {
    message: "cmftml.lint.BLOCK_EXPECTED_END",
    severity: "error"
  },

  "block-end-mismatch": {
    message: "cmftml.lint.BLOCK_END_MISMATCH",
    severity: "error"
  },

  "no-such-module": {
    message: "cmftml.lint.NO_SUCH_MODULE",
    severity: "error"
  },

  "module-missing-name": {
    message: "cmftml.lint.MODULE_MISSING_NAME",
    severity: "error"
  },

  "invalid-url": {
    message: "cmftml.lint.INVALID_URL",
    severity: "error"
  }
}

async function lint(view: EditorView) {
  try {
    const doc = view.state.doc
    const str = doc.toString()
    const len = str.length

    const emitted = await warnings(str)

    const diagnostics: Diagnostic[] = []
    for (const warning of emitted) {
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

    return diagnostics
  } catch {
    return []
  }
}

export const FTMLLinter = linter(lint)
