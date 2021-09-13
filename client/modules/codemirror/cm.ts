// organize-imports-ignore

// CodeMirror exports

export {
  CompletionContext,
  acceptCompletion,
  autocompletion,
  clearSnippet,
  closeCompletion,
  completeAnyWord,
  completeFromList,
  completionStatus,
  completionKeymap,
  currentCompletions,
  ifNotIn,
  moveCompletionSelection,
  nextSnippetField,
  prevSnippetField,
  snippet,
  snippetCompletion,
  snippetKeymap,
  startCompletion
} from "@codemirror/autocomplete"

export type {
  Completion,
  CompletionResult,
  CompletionSource
} from "@codemirror/autocomplete"

export {
  closeBrackets,
  closeBracketsKeymap,
  deleteBracketPair,
  insertBracket
} from "@codemirror/closebrackets"

export type { CloseBracketConfig } from "@codemirror/closebrackets"

export {
  copyLineDown,
  copyLineUp,
  cursorCharBackward,
  cursorCharForward,
  cursorCharLeft,
  cursorCharRight,
  cursorDocEnd,
  cursorDocStart,
  cursorGroupBackward,
  cursorGroupForward,
  cursorGroupLeft,
  cursorGroupRight,
  cursorLineBoundaryBackward,
  cursorLineBoundaryForward,
  cursorLineDown,
  cursorLineEnd,
  cursorLineStart,
  cursorLineUp,
  cursorMatchingBracket,
  cursorPageDown,
  cursorPageUp,
  cursorSyntaxLeft,
  cursorSyntaxRight,
  defaultKeymap,
  deleteCharBackward,
  deleteCharForward,
  deleteGroupBackward,
  deleteGroupForward,
  deleteLine,
  deleteToLineEnd,
  deleteToLineStart,
  deleteTrailingWhitespace,
  emacsStyleKeymap,
  indentLess,
  indentMore,
  indentSelection,
  insertNewline,
  insertNewlineAndIndent,
  insertTab,
  moveLineDown,
  moveLineUp,
  selectAll,
  selectCharBackward,
  selectCharForward,
  selectCharLeft,
  selectCharRight,
  selectDocEnd,
  selectDocStart,
  selectGroupBackward,
  selectGroupForward,
  selectGroupLeft,
  selectGroupRight,
  selectLine,
  selectLineBoundaryBackward,
  selectLineBoundaryForward,
  selectLineDown,
  selectLineEnd,
  selectLineStart,
  selectLineUp,
  selectMatchingBracket,
  selectPageDown,
  selectPageUp,
  selectParentSyntax,
  selectSyntaxLeft,
  selectSyntaxRight,
  simplifySelection,
  splitLine,
  standardKeymap,
  transposeChars,
  cursorSubwordBackward,
  cursorSubwordForward,
  indentWithTab,
  selectSubwordBackward,
  selectSubwordForward
} from "@codemirror/commands"

export {
  blockComment,
  blockUncomment,
  commentKeymap,
  lineComment,
  lineUncomment,
  toggleBlockComment,
  toggleComment,
  toggleLineComment
} from "@codemirror/comment"

export type { CommentTokens } from "@codemirror/comment"

export {
  codeFolding,
  foldAll,
  foldCode,
  foldEffect,
  foldGutter,
  foldKeymap,
  foldedRanges,
  unfoldAll,
  unfoldCode,
  unfoldEffect
} from "@codemirror/fold"

export {
  GutterMarker,
  gutter,
  gutterLineClass,
  gutters,
  highlightActiveLineGutter,
  lineNumberMarkers,
  lineNumbers
} from "@codemirror/gutter"

export {
  HighlightStyle,
  Tag,
  classHighlightStyle,
  defaultHighlightStyle,
  highlightTree,
  styleTags,
  tags
} from "@codemirror/highlight"

export type { TagStyle } from "@codemirror/highlight"

export {
  history,
  historyField,
  historyKeymap,
  invertedEffects,
  isolateHistory,
  redo,
  redoDepth,
  redoSelection,
  undo,
  undoDepth,
  undoSelection
} from "@codemirror/history"

export {
  IndentContext,
  Language,
  LanguageDescription,
  LanguageSupport,
  TreeIndentContext,
  continuedIndent,
  defineLanguageFacet,
  delimitedIndent,
  ensureSyntaxTree,
  flatIndent,
  foldInside,
  foldNodeProp,
  foldService,
  foldable,
  getIndentUnit,
  getIndentation,
  indentNodeProp,
  indentOnInput,
  indentService,
  indentString,
  indentUnit,
  language,
  languageDataProp,
  syntaxTree,
  LRLanguage,
  ParseContext
} from "@codemirror/language"

export { languages } from "@codemirror/language-data"

export {
  closeLintPanel,
  lintKeymap,
  linter,
  nextDiagnostic,
  openLintPanel,
  setDiagnostics,
  forceLinting
} from "@codemirror/lint"

export type { Action, Diagnostic } from "@codemirror/lint"

export { bracketMatching, matchBrackets } from "@codemirror/matchbrackets"

export type { Config, MatchResult } from "@codemirror/matchbrackets"

export { getPanel, panels, showPanel } from "@codemirror/panel"

export type { Panel, PanelConstructor } from "@codemirror/panel"

export { rectangularSelection } from "@codemirror/rectangular-selection"

export {
  RegExpCursor,
  SearchCursor,
  closeSearchPanel,
  findNext,
  findPrevious,
  gotoLine,
  highlightSelectionMatches,
  openSearchPanel,
  replaceAll,
  replaceNext,
  searchConfig,
  searchKeymap,
  selectMatches,
  selectNextOccurrence,
  selectSelectionMatches
} from "@codemirror/search"

export {
  Annotation,
  AnnotationType,
  ChangeDesc,
  ChangeSet,
  Compartment,
  EditorSelection,
  EditorState,
  Facet,
  Prec,
  SelectionRange,
  MapMode,
  CharCategory,
  StateEffect,
  StateEffectType,
  StateField,
  Text,
  Transaction,
  combineConfig
} from "@codemirror/state"

export type {
  ChangeSpec,
  EditorStateConfig,
  TransactionSpec,
  Extension,
  StateCommand
} from "@codemirror/state"

export {
  Line,
  codePointAt,
  codePointSize,
  countColumn,
  findClusterBreak,
  findColumn,
  fromCodePoint
} from "@codemirror/text"

export type { TextIterator } from "@codemirror/text"

export { hoverTooltip, showTooltip } from "@codemirror/tooltip"

export type { Tooltip, TooltipView } from "@codemirror/tooltip"

export {
  BidiSpan,
  BlockInfo,
  BlockType,
  Decoration,
  EditorView,
  Direction,
  MatchDecorator,
  PluginField,
  PluginFieldProvider,
  Range,
  ViewPlugin,
  ViewUpdate,
  WidgetType,
  drawSelection,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  logException,
  placeholder,
  runScopeHandlers,
  scrollPastEnd
} from "@codemirror/view"

export type {
  Command,
  DOMEventHandlers,
  DOMEventMap,
  DecorationSet,
  KeyBinding,
  MouseSelectionStyle,
  PluginSpec,
  PluginValue,
  Rect
} from "@codemirror/view"

export { RangeSet, RangeSetBuilder, RangeValue } from "@codemirror/rangeset"

export type { RangeComparator, RangeCursor, SpanIterator } from "@codemirror/rangeset"

export { css, cssCompletion, cssLanguage } from "@codemirror/lang-css"

export { html, htmlCompletion, htmlLanguage } from "@codemirror/lang-html"
