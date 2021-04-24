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
