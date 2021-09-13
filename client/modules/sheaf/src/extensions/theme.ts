import { Extension, HighlightStyle, tags as t } from "@wikijump/codemirror/cm"

// prettier-ignore
const
  background = "var(--colcode-background)",
  hover      = "var(--colcode-hover)"     ,
  border     = "var(--colcode-border)"    ,
  accent     = "var(--colcode-accent)"    ,
  selection  = "var(--colcode-selection)" ,
  text       = "var(--colcode-content)"   ,
  comment    = "var(--colcode-comment)"   ,
  doc        = "var(--colcode-commentdoc)",
  punct      = "var(--colcode-punct)"     ,
  operator   = "var(--colcode-operator)"  ,
  keyword    = "var(--colcode-keyword)"   ,
  logical    = "var(--colcode-logical)"   ,
  string     = "var(--colcode-string)"    ,
  entity     = "var(--colcode-entity)"    ,
  type       = "var(--colcode-type)"      ,
  ident      = "var(--colcode-ident)"     ,
  func       = "var(--colcode-function)"  ,
  constant   = "var(--colcode-constant)"  ,
  property   = "var(--colcode-property)"  ,
  tag        = "var(--colcode-tag)"       ,
  classes    = "var(--colcode-class)"     ,
  attr       = "var(--colcode-attribute)" ,
  markup     = "var(--colcode-markup)"    ,
  link       = "var(--colcode-link)"      ,
  invalid    = "var(--colcode-invalid)"   ,
  inserted   = "var(--colcode-inserted)"  ,
  changed    = "var(--colcode-changed)"   ,
  important  = "var(--colcode-important)" ,
  highlight  = "var(--colcode-highlight)" ,
  note       = "var(--colcode-note)"      ,
  special    = "var(--colcode-special)"

const confinementHighlightStyle = HighlightStyle.define([
  // Keywords + Operators
  {
    tag: [t.keyword],
    color: keyword
  },
  {
    tag: [t.controlOperator, t.logicOperator, t.compareOperator],
    color: logical
  },
  {
    tag: [t.regexp, t.operator],
    color: operator
  },
  // Names and Types
  {
    tag: [t.name],
    color: ident
  },
  {
    tag: [t.propertyName],
    color: property
  },
  {
    tag: [t.className],
    color: classes
  },
  {
    tag: [t.typeName, t.escape, t.standard(t.name)],
    color: type
  },
  {
    tag: [t.namespace],
    color: entity
  },
  // Functions
  {
    tag: [t.function(t.name), t.function(t.propertyName), t.macroName],
    color: func
  },
  {
    tag: [t.atom, t.annotation, t.special(t.name), t.special(t.string)],
    color: func
  },
  // Literals
  {
    tag: [t.labelName, t.monospace, t.string],
    color: string
  },
  {
    tag: [t.constant(t.name), t.local(t.name), t.literal, t.unit],
    color: constant
  },
  // Changes
  {
    tag: [t.deleted, t.invalid],
    color: invalid
  },
  {
    tag: [t.inserted],
    color: inserted
  },
  {
    tag: [t.changed],
    color: changed
  },
  // Punctuation, Comments
  {
    tag: [t.punctuation],
    color: punct
  },
  {
    tag: [t.processingInstruction],
    color: markup
  },
  {
    tag: [t.meta, t.comment],
    color: comment
  },
  {
    tag: [t.docComment, t.docString],
    color: doc
  },
  // Misc.
  {
    tag: [t.self],
    color: special
  },
  // Markup
  {
    tag: [t.link],
    color: link
  },
  {
    tag: t.url,
    color: link,
    textDecoration: "underline"
  },
  { tag: t.strong, fontWeight: "bold" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.heading, fontWeight: "bold", color: tag }
])

const confinementMarkupHighlightStyle = HighlightStyle.define([
  {
    tag: t.tagName,
    color: tag
  },
  {
    tag: t.special(t.propertyName),
    color: attr
  },
  {
    tag: t.contentSeparator,
    fontWeight: "bold",
    color: tag,
    display: "inline-block",
    width: "calc(100% - 1rem)",
    boxShadow: `inset 0 0.125rem 0 ${border}`
  },
  { tag: t.special(t.emphasis), textDecoration: "underline" }, // underline
  { tag: t.special(t.deleted), textDecoration: "line-through" }, // strikethrough
  { tag: t.special(t.inserted), background: important, color: "black" }, // mark
  { tag: t.special(t.meta), color: highlight }, // critichighlight
  { tag: t.special(t.comment), color: note } // criticcomment
])

export const confinement: Extension = [
  confinementMarkupHighlightStyle,
  confinementHighlightStyle
]
