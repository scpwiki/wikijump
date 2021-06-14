import { lb, lkup, re, TarnationLanguage } from "cm-tarnation"
import type * as DF from "cm-tarnation/src/grammar/definition"
import type { Grammar } from "cm-tarnation/src/grammar/definition"
import { ContentFacet, textBuffer } from "wj-codemirror"
import {
  cssCompletion,
  foldNodeProp,
  htmlCompletion,
  languages,
  tags as t
} from "wj-codemirror/cm"
import { completeFTML } from "../autocomplete"
import Content from "../content"
import { blocks, modules } from "../data/blocks"
import { ftmlHoverTooltips } from "../hover"
import { ftmlLinter } from "../lint"
import { spellcheckFTML } from "../spellcheck"
import { aliasesFiltered, aliasesRaw } from "../util"
import { StyleAttributeGrammar } from "./css-attributes"
import { TexLanguage } from "./tex"

const blockEntries = Object.entries(blocks)
const moduleEntries = Object.entries(modules)

const data = {
  blk_map: lkup(
    blockEntries
      .filter(([, { head, body }]) => head === "map" && body === "none")
      .flatMap(aliasesFiltered),
    { ignoreCase: true }
  ),

  blk_val: lkup(
    blockEntries
      .filter(([, { head, body }]) => head === "value" && body === "none")
      .flatMap(aliasesFiltered),
    { ignoreCase: true }
  ),

  blk_valmap: lkup(
    blockEntries
      .filter(([, { head, body }]) => head === "value+map" && body === "none")
      .flatMap(aliasesFiltered),
    { ignoreCase: true }
  ),

  // currently empty
  // blk_el: lkup(
  //   blockEntries
  //     .filter(([, { head, body }]) => head === "none" && body === "elements")
  //     .flatMap(aliases),
  //  { ignoreCase: true }
  // ),

  blk_map_el: lkup(
    blockEntries
      .filter(([, { head, body }]) => head === "map" && body === "elements")
      .flatMap(aliasesFiltered),
    { ignoreCase: true }
  ),

  blk_val_el: lkup(
    blockEntries
      .filter(([, { head, body }]) => head === "value" && body === "elements")
      .flatMap(aliasesFiltered),
    { ignoreCase: true }
  ),

  // currently empty
  // blk_valmap_el: lkup(
  //   blockEntries
  //     .filter(([, { head, body }]) => head === "value+map" && body === "elements")
  //     .flatMap(aliases),
  //  { ignoreCase: true }
  // ),

  mods: lkup(moduleEntries.flatMap(aliasesRaw), { ignoreCase: true }),

  blk_align: lkup(["=", "==", "<", ">"])
}

export const FTMLLanguage = new TarnationLanguage({
  name: "FTML",

  nestLanguages: [
    ...languages,
    TexLanguage.description,
    StyleAttributeGrammar.description
  ],

  languageData: {
    commentTokens: { block: { open: "[!--", close: "--]" } },
    autocomplete: completeFTML,
    spellcheck: spellcheckFTML
  },

  supportExtensions: [
    ContentFacet.of(async state => Content.extract(await textBuffer(state.doc))),
    ftmlLinter,
    ftmlHoverTooltips,
    htmlCompletion,
    cssCompletion
  ],

  configure: {
    props: [
      foldNodeProp.add({
        "BlockComment": tree => ({ from: tree.from + 3, to: tree.to - 3 }),
        "Container Table": (tree, state) => ({
          from: Math.min(tree.from + 20, state.doc.lineAt(tree.from).to),
          to: tree.to - 1
        }),
        "BlockNested BlockContainer": (tree, state) => {
          const from = state.doc.lineAt(tree.from).to
          const to = tree?.lastChild?.from
          if (from && to) return { from, to }
          return null
        }
      })
    ]
  },

  /*
   * Due to the weirdness of defining a grammar like this,
   * we don't want Prettier to mangle all the formatting.
   *
   * Finally, don't get too hung up on the accuracy of this grammar.
   * It's still very incomplete and will need work. However, it's good enough
   * for now, and can be used to develop extensions for FTML.
   */
  // prettier-ignore
  grammar: (): Grammar => ({

    ignoreCase: true,

    start: "root",

    variables: {

      esc: /@ws(?:_|\\)$/, // escape next line
      ws: /[^\S\r\n]/,     // whitespace, no newlines
      s: /^(?!@esc)@ws*/,  // starting whitespace
      enl: /^@ws*$/,       // empty new line

      // control characters, aka anything used to maybe signify something
      control:    /[\s!"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~\xA1\u2010-\u2027]/,
      nocontrol: /[^\s!"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~\xA1\u2010-\u2027]/,
      escapes: /\\@control/,

      bs: /\[{2}(?![\[\/])\s*/, // block node start
      bsc: /\[{2}\/\s*/,        // block closing node start
      be: /\s*(?!\]{3})\]{2}/,  // block node end
      bsf: /_?(?=@ws|@be|$)/,   // block name suffix
      // block prefix modifiers
      bm: /(?:[*=><](?![*=><])|f>|f<)(?!@ws|@be)/,

    },

    brackets: [
      { name: "BlockComment", pair: ["[!--", "--]"], tag: "t.blockComment" },

      { name: "t.paren",         pair: ["(", ")"] },
      { name: "t.brace",         pair: ["{", "}"] },
      { name: "t.squareBracket", pair: ["[", "]"] }
    ],

    global: [

      { style: {
        BlockComment: t.blockComment,
        EscapedNewline: t.escape,
        EscapedCharacter: t.escape
      } },

      [/@esc/, "EscapedNewline"],
      [/@escapes/, "EscapedCharacter"],
      [/(\[!--)([^]+?)(--\])/, "BlockComment", ["@BR", "", "@BR"]],
      [/@nocontrol+/]
    ],

    states: {

      root: [
        { include: "#block_markup" },
        { include: "#inline" },
        { include: "#include" },
        { include: "#block" }
      ],

      inline: [
        { include: "#typography" },
        { include: "#special" },
        { include: "#markup" },
        { include: "#include" },
        { include: "#block" }
      ],

      block_markup: [

        { style: {
          "HeadingMark": t.heading,
          "CenterMark": t.heading,
          "BlockquoteMark ListBulletedMark ListNumberedMark": t.keyword
        } },

        { variables: {
          // symbols that interrupt a paragraph on line start
          interrupt:/(?:\++\*?@ws)|(?:[\[\]])|(?:[*#]@ws)|(?:[-=]{4,})|(?::)|(?:>+@ws)|(?:>$)|(?:\|{2})/,
          hr: /(?:-{3,}|={3,})@ws*$/,          // horizontal rules
          heading: /(?:\++\*?)@ws+(?!$)/,      // headings
          cs: /@s(?:(?:>+@ws|>|[*#]@ws)@ws*)+/ // container start
        } },

        // horizontal rules
        [/@s@hr/, "t.contentSeparator"],
        // headings
        [/(@s@heading)(.+?)$/, "Heading",
          ["HeadingMark", { strict: false, rules: "#inline" }]
        ],
        // center rule
        [/(@s=@ws+)(.+?)$/, "Center",
          ["CenterMark", { strict: false, rules: "#inline" }]
        ],

        // tables
        { begin: [/@s\|{2}/, "@RE"],
          end:
            // fallback if we can't use the lookbehind
            [re`/(?<!@esc\s*)(@enl|^((?!\|{2}).)+$)/` ?? /@enl|^((?!\|{2}).)+$/, "@RE"],
          type: "Table",
          rules: [
            [/(\|{2,})([~=]?)/, "TableMark", ["t.separator", "t.operator"]],
            { include: "#inline" }
          ]
        },

        // containers
        { begin: [/@cs/, "@RE"],
          end: [re`/(?<!@esc\s*)(@enl|(?!@cs)@s)/` ?? /@enl|(?!@cs)@s/, "@RE"],
          type: "Container",
          rules: [
            // horizontal rules
            [/(@s)(>+|[*#])(@ws*@hr)/,
              ["", { rules: "#container_mark_type" }, "t.contentSeparator"]
            ],

            // headings
            [/(@s)(>+|[*#])(@ws*@heading.+?$)/,
              ["", { rules: "#container_mark_type" }, { rules: [
                [/(@ws*@heading)(.+)/, "Heading",
                  ["HeadingMark", { strict: false, rules: "#inline" }]
                ]
              ]}]
            ],

            // normal container start
            [/(@s)(>+|[*#])/, ["", { rules: "#container_mark_type" }]],

            { include: "#inline" }
          ]
        },

        // paragraphs
        { begin: [/@s(?!@interrupt)\S/, "@RE"],
          end: [/@s(?:@interrupt)|@enl/, "@RE"],
          type: "Paragraph",
          rules: "#inline"
        }
      ],

      container_mark_type: [
        [/>+/, "BlockquoteMark"],
        ["*",  "ListBulletedMark"],
        ["#",  "ListNumberedMark"]
      ],

      special: [
        // auto-detect links (huge frickin' regex)
        [/(\*?)((?:\w+:\/\/)?(?:[-\w@:%.+~#=]{2,256}\.(?!\.{2,3}))+?[a-z]{2,6}\b(?:[-\w@:%+.~#?&/=]*))/,
          "LinkInline", ["t.keyword", "t.link"]
        ],

        { brackets: [
          { name: "IncludeVariable", pair: ["{$", "}"], hint: "vi", tag: "t.bracket" },
          { name: "PageVariable",    pair: "%%",        hint: "vp", tag: "t.bracket" }
        ] },

        // include variables
        [/(\{\$)(.*?)(\})/, "IncludeVariable", ["@BR:vi", "t.variableName", "@BR:vi"]],

        // page variables
        [/(%%)(.*?)(%%)/, "PageVariable", [
          "@BR/O:vp",
          { strict: false, rules: [
            [/^[^{}]+/, "t.variableName"],
            [/(\{)(.*?)(\})$/, "PageVariableAccessor", ["@BR", "t.string", "@BR"]]
          ] },
          "@BR/C:vp"
        ]]
      ],

      typography: [
        { style: { Typography: t.processingInstruction } },

        // some of these require lookbehinds
        // so they're compiled with the `re` safe regex function
        // blame safari

        // ``quotation''
        [re`/\`\`(?=(?!\`\`).+?'')/`, "Typography"],
        [re`/(?<=\`\`(?!'').+?)''/`, "Typography"],

        // `quotation'
        [re`/\`(?=(?!\`).+?')/`, "Typography"],
        [re`/(?<=\`(?!').+?)'/`, "Typography"],

        // ,,quotation'' (this one is so damn stupid)
        [re`/,,(?=(?!,,).+?'')/`, "Typography"],
        [re`/(?<=,,(?!,,).+?)''/`, "Typography"],

        // <<, >>
        [/<<|>>/, "Typography"],
        // ...
        [/\.{3}/, "Typography"],
        // --
        [re`/(?<=\s)--(?=\s)/`, "Typography"]
      ],

      markup: [

        { style: {
          "Escaped EscapedBlock": t.escape,
          "EntityReference": t.character
        } },

        { brackets: [
          { name: "EscapedBlock", pair: ["@<", ">@"], tag: "t.processingInstruction" },
          { name: "Escaped",      pair: "@@",         tag: "t.processingInstruction" },
          { name: "ColorText",    pair: "##",         tag: "t.processingInstruction" }
        ] },

        // raw escape block
        [/(@<)(.*?)(>@)/, "EscapedBlock",
          ["@BR", { strict: false, rules: [[/&[\w#]+;/, "EntityReference"]] }, "@BR"]
        ],

        // @@ escape formatting (the WORST thing ever)
        [/(@@)(@@)(@@)/, "Escaped", ["@BR/O", "", "@BR/C"]],
        [/(@@)(@)(@@)/, "Escaped", ["@BR/O", "", "@BR/C"]],
        [/(@@)(@@(?!@))/, "Escaped", ["@BR/O", "@BR/C"]],
        [/(@@)(.*)(@@)/, "Escaped", ["@BR/O", "", "@BR/C"]],

        // colored text
        [/(##)(\w+)(\|)/,
          ["@BR/O", "t.color", "t.separator"], { parser: ">>ColorText" }
        ],
        ["##", "@BR/C", { parser: ">>/ColorText" }],

        // -- FORMATTING

        { style: {
          "Strong/...":        t.strong,
          "Emphasis/...":      t.emphasis,
          "Underline/...":     t.special(t.emphasis),
          "Strikethrough/...": t.special(t.deleted),
          "Mark/...":          t.special(t.inserted),
          "Subscript/...":     t.character, // TODO: style as superscript in editor
          "Superscript/...":   t.character, // TODO: style as subscript in editor
          "Monospace/...":     t.monospace
        } },


        { brackets: [
          { name: "Strong",        tag: "t.processingInstruction" },
          { name: "Emphasis",      tag: "t.processingInstruction" },
          { name: "Underline",     tag: "t.processingInstruction" },
          { name: "Strikethrough", tag: "t.processingInstruction" },
          { name: "Subscript",     tag: "t.processingInstruction" },
          { name: "Superscript",   tag: "t.processingInstruction" },
          { name: "Monospace",     tag: "t.processingInstruction" }
        ] },

        { variables: {
          formatting: ["**", "//", "__", "--", ",,", "^^", "{{", "}}"]
        } },

        // closing formatting
        [[lb`1/\S/`, /(@formatting)(?![^\W\d_])/], { rules: [
          ["**", "StrongClose",        { parser: ">>/Strong"        }],
          ["//", "EmphasisClose",      { parser: ">>/Emphasis"      }],
          ["__", "UnderlineClose",     { parser: ">>/Underline"     }],
          ["--", "StrikethroughClose", { parser: ">>/Strikethrough" }],
          [",,", "SubscriptClose",     { parser: ">>/Subscript"     }],
          ["^^", "SuperscriptClose",   { parser: ">>/Superscript"   }],
          ["}}", "MonospaceClose",     { parser: ">>/Monospace"     }]
        ] }],

        // opening formatting
        [[lb`!1/\\|\w/`, /(@formatting)/], { rules: [
          ["**", "StrongOpen",        { parser: ">>Strong"        }],
          ["//", "EmphasisOpen",      { parser: ">>Emphasis"      }],
          ["__", "UnderlineOpen",     { parser: ">>Underline"     }],
          ["--", "StrikethroughOpen", { parser: ">>Strikethrough" }],
          [",,", "SubscriptOpen",     { parser: ">>Subscript"     }],
          ["^^", "SuperscriptOpen",   { parser: ">>Superscript"   }],
          ["{{", "MonospaceOpen",     { parser: ">>Monospace"     }]
        ] }]
      ],

      block: [

        { style: {
          BlockName: t.tagName,
          BlockNameAlign: t.function(t.name),
          BlockNameModule: t.keyword,
          BlockNameUnknown: t.invalid,

          ModuleName: t.className,
          ModuleNameUnknown: t.invalid,

          BlockPrefix: t.keyword,
          BlockModifier: t.function(t.name),
          BlockValue: t.string,

          IncludeValue: t.link,
          IncludeParameterProperty: t.propertyName
        } },

        { variables: {
          // has capturing groups
          lslug: /([:#*]|(?=\/)|(?=[^#*\s]+?[@:][^#*\s]))([^#*\s]+)/,

          tls: /\[{3}(?!\[)/,    // triple link start
          tle: /(?!\]{4})\]{3}/, // triple link end

          ...data
        } },

        { brackets: [
          { name: "LinkTriple", pair: ["[[[","]]]"], hint: "li", tag: "t.squareBracket" },
          { name: "LinkSingle", pair: ["[", "]"],    hint: "li", tag: "t.squareBracket" },
          { name: "BlockNode",  pair: ["[[/", "]]"],             tag: "t.squareBracket" },
          { name: "BlockNode",  pair: ["[[", "]]"],              tag: "t.squareBracket" }
        ] },

        // -- TRIPLE LINK

        [/(@tls)([^\n\[\]]+)(@tle)/, "LinkTriple", [
          "@BR:li",
          { rules: [
            // [[[link | text]]]
            [/^([*#]?)([^|]*)(@ws*\|@ws*)(.*)$/,
              ["t.keyword", "t.link", "t.separator", ""]
            ],
            // [[[link]]]
            [/^([*#]?)([^|]+)$/, ["t.keyword", "t.link"]]
            ]
          },
          "@BR:li"
        ]],

        // botched/incomplete triple link
        [/(@tls)([^\s\[\]]*)/, "LinkTriple", ["@BR:li", "t.link"]],

        // -- EMBEDDED

        // unofficial
        // inline math block [[$...$]]
        [/(@bs)(\$)(.*?)(\$)(@be)/, "BlockInlineMath",
          ["@BR", "t.keyword", { embedded: "wikimath!" }, "t.keyword", "@BR"]
        ],

        // [[math]]
        { begin: blkStart("math", "map"),
          end: blkEnd("math"),
          type: "BlockNested",
          embedded: "wikimath!"
        },

        // [[module css]]
        { begin: blkStart("css", "module"),
          end: blkEnd("module", "BlockNameModule"),
          type: "BlockNested",
          embedded: "css!"
        },

        // [[css]]
        { begin: blkStart("css", "none"),
          end: blkEnd("css"),
          type: "BlockNested",
          embedded: "css!"
        },

        // [[html]]
        { begin: blkStart("html", "none"),
          end: blkEnd("html"),
          type: "BlockNested",
          embedded: "html!"
        },

        // [[code]]
        { begin: blkStart("code", "map", undefined, [
            [/(type)(\s*=\s*)(")((?:[^"]|\\")*?)(")/, "BlockNodeArgument", [
              "BlockNodeArgumentName",
              "t.definitionOperator",
              "@BR/O:arg",
              ["BlockNodeArgumentValue", { context: { lang: "$4" } }],
              "@BR/C:arg"
            ]],
          ]),
          end: blkEnd("code", undefined, { lang: null }),
          type: "BlockNested",
          embedded: "::lang!"
        },

        // -- BLOCKS

        // block (map)
        blkStart("@blk_map", "map"),

        // block (value)
        blkStart("@blk_val", "value"),

        // block (valmap)
        blkStart("@blk_valmap", "value+map"),

        // block modules
        blkStart("@mods", "module_loose"),

        // -- BLOCK CONTAINERS

        // block containers (map, elements)
        { begin: blkStart("@blk_map_el", "map"),
          end: blkEnd("@blk_map_el"),
          type: "BlockContainer"
        },

        // block containers (value, elements)
        { begin: blkStart("@blk_val_el", "value"),
          end: blkEnd("@blk_val_el"),
          type: "BlockContainer"
        },

        // block containers (elements)
        { begin: blkStart("@blk_el", "none"),
          end: blkEnd("@blk_el"),
          type: "BlockContainer"
        },

        // block containers (alignment)
        { begin: blkStart("@blk_align", "none", "BlockNameAlign"),
          end: blkEnd("@blk_align", "BlockNameAlign"),
          type: "BlockContainer"
        },

        // -- UNKNOWN

        { begin: [/(@bs|@bsc)(@bm?)([^\\#*\s\]]+?)(@bsf)/,
            ["@BR", "BlockPrefix", "BlockNameUnknown", "BlockModifier"]
          ],
          end: [/@be/, "@BR"],
          type: "BlockNode",
          rules: "#block_node_map"
        },

        // -- SINGLE LINKS

        [/(\[)([^\n\[\]]+)(\])/, "LinkSingle", [
          "@BR:li",
          { rules: [
            // [link text]
            [/^@lslug(@ws+|\|)(.*)$/, ["t.keyword", "t.link", "t.separator", ""]],
            // [link]
            [/^@lslug$/, ["t.keyword", "t.link"]],
            // [# anchortext]
            [/^(#)(@ws.+)$/, ["t.keyword", ""]]
          ] },
          "@BR:li"
        ]]
      ],

      block_node_map: [
        { style: {
          BlockLabel: t.invalid,
          BlockNodeArgumentName: t.special(t.propertyName),
          BlockNodeArgumentValue: t.string
        } },

        { brackets: [
          { name: "BlockNodeArgumentMark", pair: '"', hint: "arg", tag: "t.string" }
        ] },

        [/(\S+?)(\s*=\s*)(")((?:[^"]|\\")*?)(")/, "BlockNodeArgument", [
          "BlockNodeArgumentName",
          "t.definitionOperator",
          "@BR/O:arg",
          { type: "BlockNodeArgumentValue", rules: [
            ["$1", "style", { embedded: "style-attribute!" }],
            ["@DEFAULT"]
          ] },
          "@BR/C:arg"
        ]],

        [/(\S+?)(?=$|\s|@be)/, "BlockLabel"]
      ],

      include: [

        { style: {
          BlockNameInclude: t.keyword,
          IncludeValue: t.link,
          IncludeParameterProperty: t.special(t.propertyName)
        } },

        { begin: [/(@bs)(include)(@bsf)((?:@ws*)[^\s\]]+)/,
            ["@BR", "BlockNameInclude", "BlockModifier", "IncludeValue"]
          ],
          end: [/@be/, "@BR"],
          type: "IncludeNode",
          rules: [
            ["|", "t.separator"],
            { begin: [/([^\s=]+)(\s*=\s*)/,[
                "IncludeParameterProperty",
                ["t.operator", { parser: "<<IncludeParameterValue" }]
              ]],
              end: [/@be|\|/, "@RE", { parser: "<</IncludeParameterValue" }],
              type: "IncludeParameter",
              rules: "#root"
            }
          ]
        }
      ]
    }
  })
})

type Head = "none" | "map" | "value" | "value+map" | "module" | "module_loose"

function blkStart(
  name: string,
  head: Head,
  special?: DF.Type,
  rules?: DF.RuleState["rules"]
): DF.Rule | DF.RuleState {
  const blockName = special ? special : "BlockName"
  const stateRules = (
    !rules ? "#block_node_map" : [...rules, { include: "#block_node_map" }]
  ) as DF.RuleState["rules"]

  // prettier-ignore
  switch (head) {
    case "none": {
      return [
        [/(@bs)(@bm?)/, name, /(@bsf)([^]*?)(@be)/], "BlockNode",
        ["@BR", "BlockPrefix", blockName, "BlockModifier", "t.invalid", "@BR"],
        { predicate: "[[" }
      ]
    }

    case "map": {
      return {
        begin: [[/(@bs)(@bm?)/, name, /(@bsf)/],
          ["@BR", "BlockPrefix", blockName, "BlockModifier"],
          { predicate: "[[" }
        ],
        end: [/@be/, "@BR"],
        type: "BlockNode",
        rules: stateRules
      }
    }

    case "value": {
      return [
        [/(@bs)(@bm?)/, name, /(@bsf)(\s*)([^]*?)(@be)/], "BlockNode",
        ["@BR", "BlockPrefix", blockName, "BlockModifier", "", "BlockValue", "@BR"],
        { predicate: "[[" }
      ]
    }

    case "value+map": {
      return {
        begin: [[/(@bs)(@bm?)/, name, /(@bsf)(\s*)([^\s\]]*)/],
          ["@BR", "BlockPrefix", blockName, "BlockModifier", "", "BlockValue"],
          { predicate: "[[" }
        ],
        end: [/@be/, "@BR"],
        type: "BlockNode",
        rules: stateRules
      }
    }

    case "module": {
      return {
        begin: [[/(@bs)(@bm?)(module)(@bsf)(\s*)/, name],
          ["@BR", "BlockPrefix", "BlockNameModule", "BlockModifier", "", "ModuleName"],
          { predicate: "[[" }
        ],
        end: [/@be/, "@BR"],
        type: "BlockNode",
        rules: stateRules
      }
    }

    case "module_loose": {
      return {
        begin: [/(@bs)(@bm?)(module)(@bsf)(\s*)([^\s\]]+)/,
          ["@BR", "BlockPrefix", "BlockNameModule", "BlockModifier", "", { rules: [
            [name, "ModuleName"],
            ["@DEFAULT", "ModuleNameUnknown"]
          ] }],
          { predicate: "[[" }
        ],
        end: [/@be/, "@BR"],
        type: "BlockNode",
        rules: stateRules
      }
    }
  }
}

function blkEnd(name: string, special?: DF.Type, context?: DF.Context): DF.Rule {
  const blockName = special ? special : "BlockName"
  // prettier-ignore
  return [
    [/@bsc/, name, /(@bsf)(@be)/], "BlockNode",
    ["@BR", blockName, "BlockModifier", "@BR"],
    { context, predicate: "[[" }
  ]
}
