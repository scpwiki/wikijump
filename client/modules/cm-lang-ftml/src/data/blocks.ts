import type { Block, Attribute } from "./types"

export const blocks: Record<string, Block> = {}
export const blockNames = new Set<string>()

// TODO: handle translation
// currently, all of the info properties are turned into getters that can be manipulated
// this does nothing as it stands, but the infrastructure is there to handle some
// translation support

addTag("Anchor", ["a", "anchor"], {
  info: "Creates a hyperlink. Use with the {{href}} attribute."
})

addTag("Blockquote", ["blockquote", "quote"], {
  info: "Indicates that the block's contents are an extended quotation."
})

addTag("Bold", ["b", "bold", "strong"], {
  info: "Bolds (increases font weight of) a span of enclosed text."
})

addBlock("Char", {
  info: "Renders the HTML entity specified.",
  aliases: ["char", "character"],
  type: "value"
})

addTag("Checkbox", ["checkbox"], {
  info: "Creates an interactive checkbox input."
})

addBlock("Code", {
  info: "Displays the element's contents as a chunk of formatted code.",
  aliases: ["code"],
  type: "map",
  body: true,
  attrs: [
    // TODO: suggest supported languages?
    { name: "type", info: "Enables syntax highlighting using the specified language." }
  ]
})

addBlock("Collapsible", {
  info: "Creates an interactive widget that can display or hide the element's contents.",
  aliases: ["collapsible"],
  type: "map",
  body: true,
  attrs: [
    { name: "show", boolean: true },
    { name: "hide", boolean: true },
    { name: "folded", boolean: true },
    { name: "hideLocation", enum: ["top", "bottom", "both", "neither"] }
  ]
})

addBlock("CSS", {
  info: "Includes the element's contents as a stylesheet when the page is rendered.",
  aliases: ["css"],
  body: true
})

addTag("Deletion", ["del", "deletion"], {
  info: "Indicates a range of deleted text. This does not literally delete the text."
})

addTag("Div", ["div"], {
  info: "Creates a generic block container element."
})

addTag("Hidden", ["hidden"], {
  info: "Makes a block of enclosed text invisible."
})

addBlock("HTML", {
  info: "Creates an {{iframe}} which wraps around the element's contents.",
  aliases: ["html"],
  body: true
})

addBlock("iframe", {
  info: "Creates an {{iframe}} element.",
  aliases: ["iframe"],
  body: true
})

addTag("Insertion", ["ins", "insertion"], {
  info: "Indicates a range of inserted text."
})

addTag("Invisible", ["invisible"], {
  info: "Makes an enclosed span of text invisible."
})

addTag("Italics", ["i", "italics", "em", "emphasis"], {
  info: "Italicizes an enclosed span of text."
})

addBlock("Lines", {
  info: "Adds a specified number of newlines to the page.",
  aliases: ["lines", "newlines"],
  type: "value"
})

addTag("Mark", ["mark", "highlight"], {
  info: "Highlights a span of enclosed text."
})

addTag("Monospace", ["tt", "mono", "monospace"], {
  info: "Renders a span of enclosed text in a monospace font."
})

addTag("radio", ["radio", "radio-button"], {
  info: "Creates an interactive radio button input.",
  type: "value+map"
})

addBlock("Size", {
  info: "Changes the font size of an enclosed span of text.",
  aliases: ["size"],
  type: "value"
})

addTag("Span", ["span"], {
  info: "Creates a generic span (range of text) element."
})

addTag("Strikethrough", ["s", "strikethrough"], {
  info: "Renders an enclosed span of text with a strikethrough."
})

addTag("Subscript", ["sub", "subscript"], {
  info: "Renders an enclosed span of text as subscript (small text below)."
})

addTag("Superscript", ["sup", "super", "superscript"], {
  info: "Renders an enclosed span of text as superscript (small text above)."
})

addTag("Underline", ["u", "underline"], {
  info: "Renders an enclosed span of text with an underline."
})

function addTag(name: string, aliases: string[], data?: Partial<Block>) {
  addBlock(name, {
    aliases,
    type: "map",
    body: true,
    globals: true,
    ...data
  })
}

function addBlock(name: string, data: Partial<Block>) {
  const {
    aliases = [],
    type = "none",
    attrs = [],
    body = false,
    globals = false,
    info
  } = data

  const block: Block = {
    name,
    aliases,
    type,
    attrs,
    body,
    globals,
    get info() {
      // TODO: use this getter to handle translations
      return info
    }
  }

  // do a pass over `attrs` so we can add info getters
  const processedAttrs: Exclude<Attribute, string>[] = []
  for (const attr of block.attrs) {
    const info = attr.info
    processedAttrs.push({
      ...attr,
      get info() {
        // TODO: use this getter to handle translations
        return info
      }
    })
  }

  block.attrs = processedAttrs

  for (const name of block.aliases) {
    blocks[name] = block
    blockNames.add(name)
  }
}
