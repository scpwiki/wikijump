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
  startCompletion,
  type Completion,
  type CompletionResult,
  type CompletionSource,
  ifIn,
  pickedCompletion,
  selectedCompletion
} from "@codemirror/autocomplete"

export {
  closeBrackets,
  closeBracketsKeymap,
  deleteBracketPair,
  insertBracket,
  type CloseBracketConfig
} from "@codemirror/closebrackets"

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
  selectSubwordForward,
  insertBlankLine
} from "@codemirror/commands"

export {
  blockComment,
  blockUncomment,
  commentKeymap,
  lineComment,
  lineUncomment,
  toggleBlockComment,
  toggleComment,
  toggleLineComment,
  type CommentTokens,
  toggleBlockCommentByLine
} from "@codemirror/comment"

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
  tags,
  type TagStyle
} from "@codemirror/highlight"

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
  ParseContext,
  syntaxParserRunning,
  syntaxTreeAvailable
} from "@codemirror/language"

export { languages } from "@codemirror/language-data"

export {
  closeLintPanel,
  lintKeymap,
  linter,
  nextDiagnostic,
  openLintPanel,
  setDiagnostics,
  forceLinting,
  type Action,
  type Diagnostic,
  diagnosticCount,
  lintGutter,
  setDiagnosticsEffect
} from "@codemirror/lint"

export {
  bracketMatching,
  matchBrackets,
  type Config,
  type MatchResult
} from "@codemirror/matchbrackets"

export {
  getPanel,
  panels,
  showPanel,
  type Panel,
  type PanelConstructor
} from "@codemirror/panel"

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
  searchKeymap,
  selectMatches,
  selectNextOccurrence,
  selectSelectionMatches,
  SearchQuery,
  getSearchQuery,
  setSearchQuery,
  search
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
  combineConfig,
  type ChangeSpec,
  type EditorStateConfig,
  type Extension,
  type StateCommand,
  type TransactionSpec
} from "@codemirror/state"

export {
  Line,
  codePointAt,
  codePointSize,
  countColumn,
  findClusterBreak,
  findColumn,
  fromCodePoint,
  type TextIterator
} from "@codemirror/text"

export {
  hoverTooltip,
  showTooltip,
  type Tooltip,
  type TooltipView,
  closeHoverTooltips,
  getTooltip,
  hasHoverTooltips,
  repositionTooltips,
  tooltips
} from "@codemirror/tooltip"

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
  scrollPastEnd,
  type Command,
  type DOMEventHandlers,
  type DOMEventMap,
  type DecorationSet,
  type KeyBinding,
  type MouseSelectionStyle,
  type PluginSpec,
  type PluginValue,
  type Rect,
  dropCursor
} from "@codemirror/view"

export {
  RangeSet,
  RangeSetBuilder,
  RangeValue,
  type RangeComparator,
  type RangeCursor,
  type SpanIterator
} from "@codemirror/rangeset"

export { css, cssCompletion, cssLanguage } from "@codemirror/lang-css"

export { html, htmlCompletion, htmlLanguage } from "@codemirror/lang-html"

export { type Tree } from "@lezer/common"
