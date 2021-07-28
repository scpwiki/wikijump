import blocksTOML from "@root/ftml/conf/blocks.toml"
import modulesTOML from "@root/ftml/conf/modules.toml"
import { FTMLFragment } from "ftml-wasm-worker"
import { EditorSvelteComponent, EditorSvelteComponentInstance } from "wj-codemirror"
import { format } from "wj-state"
import BlockTip from "../tips/BlockTip.svelte"
import { aliasesFiltered } from "../util"
import type { BlockConfiguration, ModuleConfiguration } from "./types"

export const blocks: BlockConfiguration = blocksTOML as any
export const modules: ModuleConfiguration = modulesTOML as any

export const blockTips: Record<string, EditorSvelteComponentInstance> = {}
const blockTipHandler = new EditorSvelteComponent(BlockTip)
for (const [name, block] of Object.entries(blocks)) {
  const aliases = aliasesFiltered([name, block])
  if (aliases.length) {
    const docPath = `cmftml.blocks.${name.toLowerCase()}`

    const title = format(`${docPath}.TITLE`, { default: name })

    // create a reusable FTML fragment so that the documentation is
    // both lazily rendered and memoized
    const info = new FTMLFragment(
      format(`${docPath}.INFO`, { default: format("cmftml.blocks.BLOCK_UNDOCUMENTED") })
    )

    // providing `default: ""` doesn't appear to work, probably because it's falsy
    // so we use a special value here
    const example = format(`${docPath}.EXAMPLE`, { default: "_no_example_" })

    const docs = { title, info, example }
    const instance = blockTipHandler.create(undefined, { pass: { name, block, docs } })

    for (const alias of aliases) {
      blockTips[alias] = instance
    }
  }
}
