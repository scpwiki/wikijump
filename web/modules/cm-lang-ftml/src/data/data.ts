import blocksTOML from "@root/ftml/conf/blocks.toml"
import modulesTOML from "@root/ftml/conf/modules.toml"
import Locale, { LOCALE_CMFTML_DOCUMENTATION } from "@wikijump/fluent"
import { Prism } from "@wikijump/prism"
import { BlockData, ModuleData } from "./block"
import {
  type BlockConfiguration,
  type DocumentationData,
  type ModuleConfiguration
} from "./types"

export const Blocks: BlockConfiguration = blocksTOML as any
export const Modules: ModuleConfiguration = modulesTOML as any

export const BlockMap = new Map<string, BlockData>()
export const ModuleMap = new Map<string, ModuleData>()

export const BlockSet = new Set<BlockData>()
export const ModuleSet = new Set<ModuleData>()

export const Documentation: DocumentationData = { blocks: {} }

for (const locale of Locale.supported) {
  if (LOCALE_CMFTML_DOCUMENTATION.has(locale)) {
    const data = await LOCALE_CMFTML_DOCUMENTATION.get(locale)!()
    Object.assign(Documentation, data)
    break
  }
}

for (const name in Blocks) {
  const block = new BlockData(name, Blocks)
  BlockMap.set(name, block)
  BlockSet.add(block)
  for (const alias of block.aliases) {
    BlockMap.set(alias, block)
  }
}

for (const name in Modules) {
  const module = new ModuleData(name, Modules)
  ModuleMap.set(name, module)
  ModuleSet.add(module)
  for (const alias of module.aliases) {
    ModuleMap.set(alias, module)
  }
}

// kind of a hack, but what this does is make it so that the
// code block's `type` argument displays the languages Prism
// has available to highlight.
// the issue with that is that Prism doesn't synchronously load
// the languages, so we would just get a list of no languages
// if we tried this normally.
// so we can use a getter, and on the fly figure out what
// languages can be highlighted. this is slower, but
// the getter won't be called very often so it should be fine.
try {
  const typeArgument = BlockMap.get("code")?.arguments?.get("type")
  Object.defineProperty(typeArgument, "enumCompletions", {
    get: () => {
      return Object.entries(Prism.languages)
        .filter(([, prop]) => typeof prop !== "function")
        .map(([name]) => ({ label: name, type: "enum" }))
    }
  })
} catch {}
