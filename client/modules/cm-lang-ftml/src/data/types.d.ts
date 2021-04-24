export interface Block {
  name: string
  aliases: string[]
  type: "none" | "name" | "map" | "namemap"
  attrs: Attribute[]
  body: boolean
  deprecated?: string
  info?: string
}

export interface Attribute {
  name: string
  boolean?: boolean
  values?: string[]
  deprecated?: boolean
  info?: string
}
