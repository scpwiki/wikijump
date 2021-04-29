import type { BlockConfiguration, Block, Attribute } from "./types"

// TODO: see if an alias will work here (may not, due to various build tools interacting)
import blocksTOML from "../../../../../ftml/conf/blocks.toml"
export const blocks = (blocksTOML as unknown) as BlockConfiguration
