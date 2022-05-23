import { HighlightStyle } from "@codemirror/language"
import { tags as t } from "@lezer/highlight"

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
  storage    = "var(--colcode-storage)"   ,
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

// prettier-ignore
export const confinement = HighlightStyle.define([
  // Keywords, Operators, Language Features
  { tag: t.keyword,                                               color: keyword  },
  { tag: t.operator,                                              color: operator },
  { tag: t.labelName,                                             color: string   },
  { tag: t.self,                                                  color: special  },
  { tag: t.atom,                                                  color: type     },
  { tag: [t.controlOperator, t.logicOperator, t.compareOperator], color: logical  },
  { tag: [t.modifier, t.definitionKeyword],                       color: storage  },

  // Names
  { tag: t.name,         color: ident     },
  { tag: t.propertyName, color: property  },
  { tag: t.className,    color: classes   },
  { tag: t.namespace,    color: entity    },

  // Constants, Literals
  { tag: [t.constant(t.name), t.constant(t.variableName)],        color: constant },
  { tag: [t.string, t.special(t.string),t.regexp],                color: string   },
  { tag: [t.literal, t.integer, t.float, t.bool, t.unit, t.null], color: constant },

  // Types
  { tag: [
      t.typeName,
      t.annotation,
      t.special(t.name),
      t.standard(t.name),
      t.standard(t.variableName)
    ],
    color: type
  },

  // Functions
  { tag: [
      t.function(t.name),
      t.function(t.variableName),
      t.function(t.propertyName),
      t.definition(t.function(t.variableName)),
      t.definition(t.function(t.propertyName)),
      t.macroName
    ],
    color: func
  },

  // Changes
  { tag: t.inserted,             color: inserted },
  { tag: t.changed,              color: changed  },
  { tag: [t.deleted, t.invalid], color: invalid  },

  // Punctuation, Comments
  { tag: t.punctuation,               color: punct   },
  { tag: t.processingInstruction,     color: markup  },
  { tag: t.escape,                    color: type    },
  { tag: [t.meta, t.comment],         color: comment },
  { tag: [t.docComment, t.docString], color: doc     },

  // Markup
  { tag: t.tagName,             color: tag                                     },
  { tag: t.special(t.tagName),  color: tag                                     },
  { tag: t.attributeName,       color: attr                                    },
  { tag: t.attributeValue,      color: string                                  },
  { tag: t.link,                color: link                                    },
  { tag: t.monospace,           color: string                                  },
  { tag: t.url,                 color: link,    textDecoration: "underline"    },
  { tag: t.heading,             color: tag,     fontWeight: "bold"             },
  { tag: t.special(t.inserted), color: "black", background: important          },
  { tag: t.strong,                              fontWeight: "bold"             },
  { tag: t.emphasis,                            fontStyle: "italic"            },
  { tag: t.strikethrough,                       textDecoration: "line-through" },
  { tag: t.special(t.emphasis),                 textDecoration: "underline"    },
  { tag: t.contentSeparator,
    fontWeight: "bold",
    color: tag,
    display: "inline-block",
    width: "calc(100% - 1rem)",
    boxShadow: `inset 0 0.125rem 0 ${border}`
  },
])
