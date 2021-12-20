export type BlockConfiguration = Record<string, Block>
export type ModuleConfiguration = Record<string, Module>

export interface Block {
  "exclude-name"?: boolean
  "deprecated"?: boolean
  "aliases"?: string[]
  "accepts-star"?: boolean
  "accepts-score"?: boolean
  "accepts-newlines"?: boolean
  "head": "none" | "value" | "map" | "value+map"
  "body": "none" | "raw" | "elements" | "other"
  "html-attributes"?: boolean
  "html-output": "css" | "other" | `html,${string}` | `html,${string},${string}`
  "special"?: "include-elements" | "include-special"
  "arguments"?: Record<string, Argument>
}

export interface Module {
  "deprecated"?: boolean
  "aliases"?: string[]
  "body": "none" | "raw" | "elements" | "other"
  "html-attributes"?: boolean
  "arguments"?: Record<string, Argument>
}

export interface Argument {
  "type": `${"string" | "int" | "float" | "bool"}${"[]" | ""}`
  "enum"?: (string | number)[]
  "min-value"?: number
  "max-value"?: number
  "default"?: string | number | boolean
}
