import { hasSigil } from "@wikijump/util"
import type { Block, Module } from "./data/types"

export function aliasesRaw([name, block]: [string, Block | Module]) {
  const aliases = block.aliases ?? []
  // @ts-ignore TS doesn't like a `[]` property access on a union sometimes
  if (block["exclude-name"]) return [...aliases]
  return [name, ...aliases]
}

const EXCLUDE_BLOCKS = ["module", "module654", "include-messy", "include-elements"]

export function aliasesFiltered(blocks: [string, Block | Module]) {
  return aliasesRaw(blocks)
    .filter(str => !EXCLUDE_BLOCKS.includes(str))
    .filter(str => !hasSigil(str, ["=", "<", ">", "f<", "f>"]))
}
