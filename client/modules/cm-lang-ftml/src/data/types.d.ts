export interface Block {
  name: string
  aliases: string[]
  type: "none" | "value" | "map" | "value+map"
  attrs: Attribute[]
  body: boolean
  globals?: boolean
  deprecated?: string
  info?: string
}

export interface Attribute {
  name: string
  boolean?: boolean
  enum?: string[]
  deprecated?: boolean
  info?: string
}

export type BlockConfiguration = Record<string, Block2>

export interface Block2 {
  "deprecated"?: boolean
  "aliases"?: string[]
  "accepts-star"?: boolean
  "accepts-score"?: boolean
  "accepts-newlines"?: boolean
  "head": "none" | "value" | "map" | "value+map"
  "body": "none" | "raw" | "elements" | "other"
  "html-attributes"?: boolean
  "html-output": "css" | "other" | `html,${string}` | `html,${string},${string}`
  "special"?: "" | "module"
  "arguments"?: Record<string, Attribute2>
}

export interface Attribute2 {
  "type": `${"string" | "int" | "float"}${"[]" | ""}`
  "enum"?: (string | number)[]
  "min-value"?: number
  "max-value"?: number
  "default"?: string | number
}
