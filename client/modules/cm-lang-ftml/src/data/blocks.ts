// TODO: see if an alias will work here (may not, due to various build tools interacting)
import blocksTOML from "../../../../../ftml/conf/blocks.toml"
import modulesTOML from "../../../../../ftml/conf/modules.toml"
import type { BlockConfiguration, ModuleConfiguration } from "./types"

export const blocks: BlockConfiguration = blocksTOML as any
export const modules: ModuleConfiguration = modulesTOML as any
