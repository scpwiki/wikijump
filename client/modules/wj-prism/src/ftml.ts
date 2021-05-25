/** Adds FTML syntax highlighting to a Prism instance. */
export function prismFTML(Prism: typeof import("Prismjs")) {
  function generateEmbedded(embed: string, start: string, end = start) {
    const pattern = new RegExp(
      `(\\[\\[\\s*${start}[^]*?\\]\\])([^]*?(?=\\[\\[\\/\\s*${end}\\s*\\]\\]))`,
      "im"
    )
    return {
      pattern,
      lookbehind: true,
      inside: Prism.languages[embed]
    }
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
