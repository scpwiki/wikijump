/* Exports SheafCore, the core class that wraps around CodeMirror 6. */

import { autocompletion, completionKeymap } from "@codemirror/autocomplete"
import { closeBrackets, closeBracketsKeymap } from "@codemirror/closebrackets"
import { copyLineDown, defaultKeymap, defaultTabBinding } from "@codemirror/commands"
import { commentKeymap } from "@codemirror/comment"
import { foldGutter, foldKeymap } from "@codemirror/fold"
import { lineNumbers } from "@codemirror/gutter"
import { history, historyKeymap, redo } from "@codemirror/history"
import { indentOnInput, syntaxTree } from "@codemirror/language"
import { nextDiagnostic, openLintPanel } from "@codemirror/lint"
import { bracketMatching } from "@codemirror/matchbrackets"
import { rectangularSelection } from "@codemirror/rectangular-selection"
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search"
import { EditorState, Extension } from "@codemirror/state"
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  ViewPlugin,
  ViewUpdate
} from "@codemirror/view"
import { writable } from "svelte/store"
import { createSheafBinding, SheafBindings } from "./bindings"
import { indentHack } from "./extensions/indent-hack"
import { printTree } from "./print-tree"
import { confinement } from "./theme"
import { EditorField } from "./util/editor-field"

export * from "./adapters/svelte-dom"
export * from "./adapters/svelte-lifecycle-element"
export * from "./adapters/svelte-panel"

interface EditorStore {
  /** Whether or not the editor has been mounted yet. */
  mounted: boolean
  /** The current document of the editor. */
  doc: EditorState["doc"]
  /** The current 'value' (content) of the editor. */
  value: string
  /** Reference to the editor core. */
  self: SheafCore
}

export class SheafCore {
  /** The element the editor is attached to. */
  parent!: Element

  /**
   * The CodeMirror `EditorState` the editor has currently. The state is
   * immutable and is replaced as the editor updates.
   */
  state = EditorState.create()

  /** The CodeMirror `EditorView` instance the editor interacts with the DOM with. */
  view!: EditorView

  /** A store that allows reactive access to editor state. */
  store = writable<EditorStore>({
    mounted: false,
    doc: this.state.doc,
    value: "",
    self: this
  })
  subscribe = this.store.subscribe
  set = this.store.set

  /**
   * The lines currently being interacted with by the user. This includes
   * all selected lines, the line the cursor is present on, etc.
   */
  activeLines = writable(new Set<number>())

  spellcheck = new EditorField<boolean>({
    default: true
  })

  gutters = new EditorField<boolean>({
    default: true,
    reconfigure: state => (state ? [lineNumbers(), foldGutter()] : null)
  })

  /** Starts the editor. */
  async init(
    parent: Element,
    doc: string,
    bindings: SheafBindings = {},
    extensions: Extension[] = []
  ) {
    this.parent = parent

    const updateHandler = ViewPlugin.define(() => ({
      update: (update: ViewUpdate) => {
        // update store on change
        if (update.docChanged) this.refresh()
        // get active lines
        if (update.selectionSet || update.docChanged) {
          const activeLines: Set<number> = new Set()
          for (const range of update.state.selection.ranges) {
            const lnStart = update.state.doc.lineAt(range.from).number
            const lnEnd = update.state.doc.lineAt(range.to).number
            if (lnStart === lnEnd) activeLines.add(lnStart - 1)
            else {
              const diff = lnEnd - lnStart
              for (let lineNo = 0; lineNo <= diff; lineNo++) {
                activeLines.add(lnStart + lineNo - 1)
              }
            }
          }
          this.activeLines.set(activeLines)
        }
      }
    }))

    this.view = new EditorView({
      parent,
      state: EditorState.create({
        doc,
        extensions: [
          ...getExtensions(),
          ...createSheafBinding(this, bindings),
          ...extensions,
          this.spellcheck,
          this.gutters,
          updateHandler
        ]
      })
    })

    this.refresh()
  }

  /** The `Text` object of the editor's current state. */
  get doc() {
    return this.view.state.doc
  }

  get scrollTop() {
    return this.view.scrollDOM.scrollTop
  }

  set scrollTop(val: number) {
    this.view.scrollDOM.scrollTop = val
  }

  /**
   * Destroys the editor. Usage of the editor object after destruction is
   * obviously not recommended.
   */
  destroy() {
    this.view.destroy()
  }

  refresh() {
    this.state = this.view.state
    let memo: string | null = null
    this.store.set({
      mounted: true,
      doc: this.doc,
      get value() {
        if (memo) return memo
        return (memo = this.doc.toString())
      },
      self: this
    })
  }

  /** Returns the scroll-offset from the top of the editor for the specified line. */
  heightAtLine(line: number) {
    return this.view.visualLineAt(this.doc.line(line).from).top
  }

  printTree() {
    return printTree(syntaxTree(this.state), this.doc.toString())
  }
}

export function getExtensions() {
  return [
    // gutter extensions are handled by the editor's guttersCompartment
    // lineNumbers(),
    // foldGutter(),
    highlightSpecialChars(),
    history(),
    drawSelection(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    highlightSelectionMatches(),
    autocompletion(),
    rectangularSelection(),
    highlightActiveLine(),
    EditorView.lineWrapping,
    indentHack,
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...commentKeymap,
      ...completionKeymap,
      { key: "Mod-l", run: openLintPanel, preventDefault: true },
      { key: "F8", run: nextDiagnostic, preventDefault: true },
      { key: "Mod-Shift-z", run: redo, preventDefault: true },
      { key: "Mod-d", run: copyLineDown, preventDefault: true },
      defaultTabBinding
    ]),
    confinement
  ]
}

export function getCodeDisplayExtensions() {
  return [
    drawSelection(),
    EditorView.editable.of(false),
    EditorView.lineWrapping,
    indentHack,
    confinement
  ]
}
