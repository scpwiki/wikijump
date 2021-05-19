import { Diagnostic, linter } from "@codemirror/lint"
import type { EditorView } from "@codemirror/view"
import { warnings } from "ftml-wasm-worker"

interface WarningInfo {
  message: string | ((rule: string, slice: string) => string)
  severity: "info" | "warning" | "error"
}

// TODO: translation handling (need a schema for that first)

const warningInfo: Record<string, WarningInfo | null> = {
  // ignored warnings
  "no-rules-match": null,
  "end-of-input": null,
  "rule-failed": null,

  "recursion-depth-exceeded": {
    message: "Too much recursion in markup.",
    severity: "error"
  },

  "not-implemented": {
    message: (rule, slice) =>
      `The syntax '${slice}' hasn\'t been implemented in FTML yet.`,
    severity: "warning"
  },

  "invalid-include": {
    message: "This include isn't valid and can't be rendered.",
    severity: "error"
  },

  "list-depth-exceeded": {
    message: "This list is nested too deeply, and can't be rendered.",
    severity: "error"
  },

  "blockquote-depth-exceeded": {
    message: "This blockquote is nested too deeply, and can't be rendered.",
    severity: "error"
  },

  "no-such-block": {
    message: (rule, slice) => `Unknown block '${slice}'.`,
    severity: "error"
  },

  "invalid-special-block": {
    message: (rule, slice) =>
      `Block '${slice}' doesn't have a special invocation. (starting '*' character)`,
    severity: "warning"
  },

  "block-missing-name": {
    message: (rule, slice) =>
      `Block '${slice}' requires a name/value, but none is specified.`,
    severity: "error"
  },

  "block-missing-close-brackets": {
    message: "Block is missing closing ']]' brackets.",
    severity: "error"
  },

  "block-malformed-arguments": {
    message: (rule, slice) =>
      `Block '${slice}' is missing one or more required arguments.`,
    severity: "error"
  },

  "block-expected-end": {
    message: (rule, slice) =>
      `Block of type '${rule}' was expected to end by at least this point.`,
    severity: "error"
  },

  "block-end-mismatch": {
    message: (rule, slice) => `Block of type '${rule}' was expected to end here.`,
    severity: "error"
  },

  "no-such-module": {
    message: (rule, slice) => `Unknown module '${slice}'.`,
    severity: "error"
  },

  "module-missing-name": {
    message: "A module name was expected to be provided here.",
    severity: "error"
  },

  "invalid-url": {
    message: (rule, slice) => `The URL '${slice}' is invalid.`,
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
      const source = `ftml(${rule}: ${kind} at ${token}) [${from}, ${to}]`

      if (typeof message === "function") {
        message = message(rule, doc.sliceString(from, to))
      }

      diagnostics.push({ from, to, message, severity, source })
    }

    return diagnostics
  } catch {
    return []
  }
}

export const FTMLLinter = linter(lint)
