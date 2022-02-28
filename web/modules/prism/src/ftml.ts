import type { Prism as PrismType } from "./index"

/** Adds FTML syntax highlighting to a Prism instance. */
export function prismFTML(Prism: PrismType) {
  function generateEmbedded(embed: string, start: string, end = start) {
    const pattern = new RegExp(
      `(\\[\\[\\s*${start}[^]*?\\]\\])([^]*?(?=\\[\\[\\/\\s*${end}\\s*\\]\\]))`,
      "im"
    )
    return {
      pattern,
      lookbehind: true,
      // use a getter so that the language doesn't have to exist right away
      // this is so that we can do recursive highlighting (see below)
      get inside() {
        return Prism.languages[embed]
      }
    }
  }

  const codePatterns: Record<string, ReturnType<typeof generateEmbedded>> = {}
  // languages that we'll add [[code]] embedded highlighting for
  const highlightLanguages = [
    "ftml",
    "wikidot",
    "wikijump",
    "wikitext",
    ...Object.keys(Prism.languages)
  ]
  // make a embedded highlighting rule for every language from the above list
  for (const language of highlightLanguages) {
    codePatterns[`code-${language}`] = generateEmbedded(
      language,
      `code[^]*?type\\s*=\\s*"\\s*${language}\\s*"`,
      "code"
    )
  }

  Prism.languages.ftml = {
    "comment": /\[!--[^]*?--\]/im,

    "escape-nl": {
      pattern: / (_|\\)$/im,
      alias: "escaped"
    },

    "escape-at": {
      pattern: /@@[^]+?@@/i,
      alias: "escaped"
    },

    "escape-bracket": {
      pattern: /@<[^]+?>@/im,
      alias: "escaped"
    },

    "link-triple": {
      pattern: /(\[{3}(?!\[))([^\n\[\]]+)((?!\]{4})\]{3})/,
      inside: {
        "punctuation": /\[\[\[|\]\]\]|\|/,
        "url": /[^\[\]\|]+/
      }
    },

    "embedded-css": generateEmbedded("css", "css"),
    "embedded-css-module": generateEmbedded("css", "module\\s*css", "module"),
    "embedded-html": generateEmbedded("html", "html"),
    "embedded-math": generateEmbedded("tex", "math"),

    ...codePatterns,
    "code": generateEmbedded("plaintext", "code"),

    "block": {
      // this horrifying pattern is actually what the CM parser uses (mostly, anyways)
      // however unlike in Tarnation we can't use regexp variables easily so...
      // just accept this as black magic and move on - if it needs to be edited,
      // use the Tarnation parser as reference and don't try to hand make this
      pattern:
        /((\[{2}(?!\[)\s*(?!\/))|(\[{2}\/\s*))(((?:[*=><](?![*=><])|f>|f<)(?![^\S\r\n]|(\s*(?!\]{3})\]{2})))?)([^\\#*\s\]]+?)(_?(?=[^\S\r\n]|\s*(?!\]{3})\]{2}|$))([^]*?)(\s*(?!\]{3})\]{2})/im,
      inside: {
        "block-name": {
          pattern: /(^\[\[\/?)((?:[*=><](?![*=><])|f>|f<)*)([^\s\]]+)/i,
          lookbehind: true,
          inside: {
            "keyword": /(^([*=><]|f>|f<))|_$/i,
            "tag": /[^\s*=><_]+/i
          }
        },
        "argument": {
          pattern: /(\S+?)(\s*=\s*)/i,
          inside: {
            "attr-name": /[^\s=]/i,
            "operator": /=/i
          }
        },
        "string": /"[^"]*"/i,
        "punctuation": /\[\/|[\[\]]/i,
        "block-label": {
          pattern: /([^\s\]=](?![="]))+/i,
          alias: "string"
        }
      }
    },

    "table-mark": {
      pattern: /(\|{2,})([~=]?)/i,
      alias: "punctuation"
    },

    "blockquote": {
      pattern: /^\s*>(?:[\t ]*>)*/im,
      alias: "keyword"
    },

    "list-hash": {
      pattern: /^\s*#(?!#)(?:[\t ]*#(?!#))*/im,
      alias: "keyword"
    },

    "list-star": {
      pattern: /^\s*\*(?!\*)(?:[\t ]*\*(?!\*))*/im,
      alias: "keyword"
    },

    "hr": {
      pattern: /(^(?:\s*|>*|\|\|[~=]?))(?:-{3,}|={3,})\s*$/im,
      lookbehind: true,
      alias: "keyword"
    },

    "heading": {
      pattern: /(^(?:\s*|>*|\|\|[~=]?))(?:\++\*?)\s+(?!$)/im,
      lookbehind: true,
      alias: "keyword"
    },

    "colored-text": {
      pattern: /##\w+\|/i,
      inside: {
        "punctuation": /##|\|/i,
        "constant": /\w+/i
      }
    },

    "colored-text-end": {
      pattern: /##/i,
      alias: "punctuation"
    },

    "formatting": {
      pattern: /\*\*|\/\/|__|--|,,|\^\^|\{\{|\}\}/i,
      alias: "operator"
    }
  }

  Prism.languages.wikidot = Prism.languages.ftml
  Prism.languages.wikijump = Prism.languages.ftml
  Prism.languages.wikitext = Prism.languages.ftml
}
