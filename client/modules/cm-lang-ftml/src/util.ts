import { hasSigil } from "wj-util"
import type { Block, Module } from "./data/types"

export function aliasesRaw([name, block]: [string, Block | Module]) {
  return [name, ...(block.aliases ?? [])]
}

const EXCLUDE_BLOCKS = ["module", "module654", "include"]

export function aliasesFiltered(blocks: [string, Block | Module]) {
  return aliasesRaw(blocks)
    .filter(str => !EXCLUDE_BLOCKS.includes(str))
    .filter(str => !hasSigil(str, ["=", "<", ">", "f<", "f>"]))
}
