import type { BlockConfiguration, ModuleConfiguration } from "./types"

// TODO: see if an alias will work here (may not, due to various build tools interacting)
import blocksTOML from "../../../../../ftml/conf/blocks.toml"
export const blocks: BlockConfiguration = blocksTOML as any

import modulesTOML from "../../../../../ftml/conf/modules.toml"
export const modules: ModuleConfiguration = modulesTOML as any
