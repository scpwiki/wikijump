import {
  EditorSvelteComponent,
  type EditorSvelteComponentInstance
} from "@wikijump/codemirror"
import { type Completion } from "@wikijump/codemirror/cm"
import Locale, { LOCALE_CMFTML_BLOCKS } from "@wikijump/fluent"
import { FTMLFragment } from "@wikijump/ftml-wasm-worker"
import BlockTip from "../tips/BlockTip.svelte"
import ModuleTip from "../tips/ModuleTip.svelte"
import { aliasesFiltered, aliasesRaw } from "../util"
import { htmlArgumentCompletions } from "./html-attributes"
import { type Argument, type BlockConfiguration, type ModuleConfiguration } from "./types"

let blockDocData: Record<string, { TITLE: string; INFO: string; EXAMPLE: string }> = {}

for (const locale of Locale.supported) {
  if (LOCALE_CMFTML_BLOCKS.has(locale)) {
    blockDocData = await LOCALE_CMFTML_BLOCKS.get(locale)!()
    break
  }
}

const BlockTipFactory = new EditorSvelteComponent(BlockTip)
const ModuleTipFactory = new EditorSvelteComponent(ModuleTip)

export class ArgumentData {
  declare name: string

  declare type: `${"string" | "int" | "float" | "bool"}${"[]" | ""}`
  declare enum?: (string | number)[]
  declare minValue?: number
  declare maxValue?: number
  declare default?: string | number | boolean

  declare isList: boolean

  declare completion: Completion
  declare enumCompletions?: Completion[]

  constructor(name: string, argument: Argument) {
    this.name = name

    assign(argument, this, ["type", "enum", ["min-value", "minValue"], "default"])

    if (this.type.endsWith("[]")) this.isList = true

    this.completion = {
      label: name,
      type: "property",
      apply: `${name}=""`
    }

    if (this.enum) {
      this.enumCompletions = []
      for (const val of this.enum) {
        const completion: Completion = { label: String(val), type: "enum" }
        if (this.default === val) completion.detail = "default"
        this.enumCompletions.push(completion)
      }
    }
    // treat booleans as ["true", "false"] enums
    else if (this.type === "bool") {
      this.enumCompletions = [
        { label: "true", type: "keyword" },
        { label: "false", type: "keyword" }
      ]

      if (this.default !== undefined) {
        this.enumCompletions[this.default ? 0 : 1].detail = "default"
      }
    }
  }
}

export class BlockData {
  declare name: string

  excludeName = false
  deprecated = false
  acceptsStar = false
  acceptsScore = false
  acceptsNewlines = false
  htmlAttributes = false

  declare aliases: string[]
  declare aliasesRaw: string[]
  declare head: "none" | "value" | "map" | "value+map"
  declare body: "none" | "raw" | "elements" | "other"
  declare htmlOutput: "css" | "other" | `html,${string}` | `html,${string},${string}`
  declare special?: "include-elements" | "include-special"

  declare arguments?: Map<string, ArgumentData>

  declare outputType: "css" | "html" | "other"
  declare outputTag?: string
  declare outputClass?: string

  declare docs?: {
    title: string
    info: FTMLFragment
    example: string
  }

  declare tip: EditorSvelteComponentInstance

  completions: Completion[] = []

  declare argumentCompletions?: Completion[]

  constructor(name: string, config: BlockConfiguration) {
    if (!config.hasOwnProperty(name)) throw new Error(`Unknown block: ${name}`)

    this.name = name

    const block = config[name]

    assign(block, this, [
      ["exclude-name", "excludeName"],
      "deprecated",
      ["accepts-star", "acceptsStar"],
      ["accepts-score", "acceptsScore"],
      ["accepts-newlines", "acceptsNewlines"],
      "head",
      "body",
      ["html-attributes", "htmlAttributes"],
      ["html-output", "htmlOutput"],
      "special"
    ])

    this.aliases = aliasesFiltered([name, block])
    this.aliasesRaw = aliasesRaw([name, block])

    if (block.arguments) {
      this.arguments = new Map()
      this.argumentCompletions = []
      for (const arg in block.arguments) {
        const argument = new ArgumentData(arg, block.arguments[arg])
        this.arguments.set(arg, argument)
        this.argumentCompletions.push(argument.completion)
      }
    }

    if (this.htmlAttributes) {
      this.argumentCompletions ??= []
      this.argumentCompletions.push(...htmlArgumentCompletions)
    }

    const [type, tag, cls] = this.htmlOutput.split(",")

    // @ts-ignore
    if (type) this.outputType = type
    if (tag) this.outputTag = tag
    if (cls) this.outputClass = cls

    if (blockDocData[name]) {
      const data = blockDocData[name]
      const fragment = new FTMLFragment(data.INFO)
      this.docs = { title: data.TITLE, info: fragment, example: data.EXAMPLE }
    }

    this.tip = BlockTipFactory.create(undefined, { pass: { block: this } })

    for (const alias of this.aliases) {
      this.completions.push({
        label: alias,
        type: "type",
        info: () => this.tip.dom
      })
    }
  }
}

export class ModuleData {
  declare name: string

  deprecated = false
  htmlAttributes = false

  declare aliases: string[]
  declare aliasesRaw: string[]
  declare body: "none" | "raw" | "elements" | "other"

  declare arguments?: Map<string, ArgumentData>

  declare tip: EditorSvelteComponentInstance

  completions: Completion[] = []

  declare argumentCompletions?: Completion[]

  constructor(name: string, config: ModuleConfiguration) {
    if (!config.hasOwnProperty(name)) throw new Error(`Unknown module: ${name}`)

    this.name = name

    const module = config[name]

    assign(module, this, ["deprecated", "body", ["html-attributes", "htmlAttributes"]])

    this.aliases = aliasesFiltered([name, module])

    if (module.arguments) {
      this.arguments = new Map()
      this.argumentCompletions = []
      for (const arg in module.arguments) {
        const argument = new ArgumentData(arg, module.arguments[arg])
        this.arguments.set(arg, argument)
        this.argumentCompletions.push(argument.completion)
      }
    }

    if (this.htmlAttributes) {
      this.argumentCompletions ??= []
      this.argumentCompletions.push(...htmlArgumentCompletions)
    }

    this.tip = ModuleTipFactory.create(undefined, { pass: { module: this } })

    for (const alias of this.aliases) {
      this.completions.push({
        label: alias,
        type: "class",
        info: () => this.tip.dom
      })
    }
  }
}

function assign(from: any, to: any, fields: (string | [string, string])[]) {
  for (const field of fields) {
    const [fromField, toField] = typeof field === "string" ? [field, field] : field
    if (from.hasOwnProperty(fromField) && from[fromField] !== undefined) {
      to[toField] = from[fromField]
    }
  }
}
