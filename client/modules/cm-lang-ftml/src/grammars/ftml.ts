import { tags as t } from "@codemirror/highlight"
import { cssCompletion } from "@codemirror/lang-css"
import { htmlCompletion } from "@codemirror/lang-html"
import { foldNodeProp } from "@codemirror/language"
import { languages } from "@codemirror/language-data"
import { lb, lkup, re, TarnationLanguage } from "cm-tarnation"
import type { Grammar } from "cm-tarnation/src/grammar/definition"
import { completeFTML } from "../autocomplete/autocomplete"
import { blocks, modules } from "../data/blocks"
import { FTMLLinter } from "../lint"
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
    autocomplete: completeFTML
  },

  supportExtensions: [FTMLLinter, htmlCompletion, cssCompletion],

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

      bs: /\[{2}(?!\[)\s*(?!\/)/, // block node start
      bsc: /\[{2}\/\s*/,          // block closing node start
      be: /\s*(?!\]{3})\]{2}/,    // block node end
      bsf: /_?(?=@ws|@be|$)/,     // block name suffix
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
        { include: "#special" },
        { include: "#typography" },
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
        [/(\*?)((?:\w+:\/\/)?(?:[-\w@:%.+~#=]{2,256}\.(?!\.{3}))+?[a-z]{2,6}\b(?:[-\w@:%+.~#?&/=]*))/,
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
          BlockNameSpecial: t.keyword,
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

        [/(@tls)([^\n\[\]]+)(@tle)/, "LinkTriple", ["@BR:li", { rules: [
          // [[[link | text]]]
          [/^([*#]?)([^|]*)(@ws*\|@ws*)(.*)$/,
            ["t.keyword", "t.link", "t.separator", { strict: false, rules: "#inline" }]
          ],
          // [[[link]]]
          [/^([*#]?)([^|]+)$/, ["t.keyword", "t.link"]]
        ] }, "@BR:li"]],

        // -- EMBEDDED

        // unofficial
        // inline math block [[$...$]]
        [/(@bs)(\$)(.*?)(\$)(@be)/, "BlockInlineMath",
          ["@BR", "t.keyword", { embedded: "wikimath!" }, "t.keyword", "@BR"]
        ],

        // [[math]]
        { begin:
          { begin: [[/(@bs)(@bm?)(math)(@bsf)/],
              ["@BR", "BlockPrefix", "BlockName", "BlockModifier"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end:   [/(@bsc)(math)(@be)/, "BlockNode", ["@BR", "BlockName", "@BR"]],
          type: "BlockNested",
          embedded: "wikimath!"
        },

        // [[module css]]
        { begin:
          { begin: [/(@bs)(@bm?)(module)(@bsf)(\s*)(css)/,
              ["@BR", "BlockPrefix", "BlockNameModule", "BlockModifier", "", "ModuleName"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end:   [/(@bsc)(module)(@be)/, "BlockNode", ["@BR", "BlockNameModule", "@BR"]],
          type: "BlockNested",
          embedded: "css!"
        },

        // [[css]]
        { begin:
          { begin: [[/(@bs)(@bm?)(css)(@bsf)/],
              ["@BR", "BlockPrefix", "BlockNameSpecial", "BlockModifier"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end:   [/(@bsc)(css)(@be)/, "BlockNode", ["@BR", "BlockNameSpecial", "@BR"]],
          type: "BlockNested",
          embedded: "css!"
        },

        // [[html]]
        { begin:
          { begin: [[/(@bs)(@bm?)(html)(@bsf)/],
              ["@BR", "BlockPrefix", "BlockNameSpecial", "BlockModifier"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end:   [/(@bsc)(html)(@be)/, "BlockNode", ["@BR", "BlockNameSpecial", "@BR"]],
          type: "BlockNested",
          embedded: "html!"
        },

        // [[code]]
        { begin:
          { begin: [[/(@bs)(@bm?)/, "code", /(@bsf)/],
              ["@BR", "BlockPrefix", "BlockNameSpecial", "BlockModifier"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: [
              [/(type)(\s*=\s*)(")((?:[^"]|\\")*?)(")/, "BlockNodeArgument", [
                "BlockNodeArgumentName",
                "t.definitionOperator",
                "@BR/O:arg",
                ["BlockNodeArgumentValue", { context: { lang: "$4" } }],
                "@BR/C:arg"
              ]],
              { include: "#block_node_map" }
            ]
          },
          end: [/(@bsc)(code)(@be)/, "BlockNode",
            ["@BR", "BlockNameSpecial", "@BR"], { context: { lang: null } }
          ],
          type: "BlockNested",
          embedded: "::lang!"
        },

        // -- BLOCKS

        // block (map)
        { begin: [[/(@bs)(@bm?)/, "@blk_map", /(@bsf)/],
            ["@BR", "BlockPrefix", "BlockName", "BlockModifier"]
          ],
          end: [/@be/, "@BR"],
          type: "BlockNode",
          rules: "#block_node_map"
        },

        // block (value)
        [[/(@bs)(@bm?)/, "@blk_val", /(@bsf)(\s*)([^\s]*?)(@be)/], "BlockNode",
          ["@BR", "BlockPrefix", "BlockName", "BlockModifier", "", "BlockValue", "@BR"]
        ],

        // block (valmap)
        { begin: [[/(@bs)(@bm?)/, "@blk_valmap", /(@bsf)(\s*)([^\]\s]*)/],
            ["@BR", "BlockPrefix", "BlockName", "BlockModifier", "", "BlockValue"]
          ],
          end: [/@be/, "@BR"],
          type: "BlockNode",
          rules: "#block_node_map"
        },

        // block modules
        { begin: [/(@bs)(@bm?)(module)(@bsf)(\s*)([^\s\]]+)/,
            ["@BR", "BlockPrefix", "BlockNameModule", "BlockModifier", "", { rules: [
              ["@mods", "ModuleName"],
              ["@DEFAULT", "ModuleNameUnknown"]
            ] }]
          ],
          end: [/@be/, "@BR"],
          type: "BlockNode",
          rules: "#block_node_map"
        },

        // -- BLOCK CONTAINERS

        // block containers (map, elements)
        { begin:
          { begin: [[/(@bs)(@bm?)/, "@blk_map_el", /(@bsf)/],
              ["@BR", "BlockPrefix", "BlockName", "BlockModifier"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end: [[/@bsc/, "@blk_map_el", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockName", "BlockModifier", "@BR"]
          ],
          type: "BlockContainer"
        },

        // block containers (value, elements)
        { begin:
          { begin: [[/(@bs)(@bm?)/, "@blk_val_el", /(@bsf)(\s*)([^\]\s]*)/],
              ["@BR", "BlockPrefix", "BlockName", "BlockModifier", "", "BlockValue"]
            ],
            end: [/@be/, "@BR"],
            type: "BlockNode",
            rules: "#block_node_map"
          },
          end: [[/@bsc/, "@blk_val_el", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockName", "BlockModifier", "@BR"]
          ],
          type: "BlockContainer"
        },

        // block containers (elements)
        { begin: [[/(@bs)(@bm?)/, "@blk_el", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockPrefix", "BlockName", "BlockModifier", "@BR"]
          ],
          end: [[/@bsc/, "@blk_el", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockName", "BlockModifier", "@BR"]
          ],
          type: "BlockContainer"
        },

        // block containers (alignment)
        { begin: [[/(@bs)(@bm?)/, "@blk_align", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockPrefix", "BlockNameAlign", "BlockModifier", "@BR"]
          ],
          end: [[/@bsc/, "@blk_align", /(@bsf)(@be)/], "BlockNode",
            ["@BR", "BlockNameAlign", "BlockModifier", "@BR"]
          ],
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

        [/(\[)([^\n\[\]]+)(\])/, "LinkSingle", ["@BR:li", { rules: [
          // [link text]
          [/^@lslug(@ws+|\|)(.*)$/,
            ["t.keyword", "t.link", "t.separator", { strict: false, rules: "#inline" }]
          ],
          // [link]
          [/^@lslug$/, ["t.keyword", "t.link"]],
          // [# anchortext]
          [/^(#)(@ws.+)$/,
            ["t.keyword", { strict: false, rules: "#inline" }]
          ]
        ] }, "@BR:li"]]
      ],

      block_node_map: [
        { style: {
          BlockLabel: t.invalid,
          BlockNodeArgumentName: t.propertyName,
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
          IncludeParameterProperty: t.propertyName
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
