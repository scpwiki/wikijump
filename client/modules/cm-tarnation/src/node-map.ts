import { styleTags, tags } from "@codemirror/highlight"
import { NodeProp, NodePropSource, NodeSet, NodeType } from "lezer-tree"
import type { AddNodeSpec, ParserConfiguration } from "./types"

export class NodeMap {
  /** Maps type names to node IDs, or vice versa. */
  private map = new Map<string | number, string | number>()

  /** The list of all node types. */
  types: NodeType[] = []

  /** The `NodeSet` referenced by syntax trees. */
  set = new NodeSet(this.types)

  /** The list of tags that are referenced when adding nodes. */
  // @ts-ignore
  private tags: Record<string, Tag> = { ...tags }

  /** Gets the node ID for a type name. */
  get(name: string): number | undefined
  /** Gets the type name for a node ID. */
  get(id: number): string | undefined
  get(name: string | number): string | number | undefined {
    return this.map.get(name)
  }

  /**
   * Adds a new `NodeType`. Returns the node created. If this couldn't be
   * done for whatever reason, null will be returned instead.
   *
   * @param spec - May be a `NodeType` already, or the specification for
   *   creating one.
   * @param name - Sets the **mapped** name for the node. Required if
   *   providing a `NodeType` directly. *This name is difference from the
   *   token's `spec` name*. This is the name used when retrieving the node
   *   ID from the map. If this isn't provided, the name will default to
   *   the `spec` name.
   */
  add(spec: AddNodeSpec, name?: string): NodeType | null
  add(spec: NodeType, name: string): NodeType | null
  add(spec: AddNodeSpec | NodeType, name?: string): NodeType | null {
    const { map, types, tags } = this

    // adding a node type directly
    if (spec instanceof NodeType && name) {
      map.set(name, spec.id)
      map.set(spec.id, name)
      types.push(spec)
      return spec
    }

    // creating a node type
    if (!(spec instanceof NodeType)) {
      /*
       * There are two ways a node can be interpreted:
       * 1. The node's name is lowercased. That means it is a shortcut for a
       *    CodeMirror highlighting tag.
       * 2. The node's name is capitalized. That means it is a custom name,
       *    and no assumptions will be made about its highlighting or styling.
       */

      const id = map.size / 2 // always increases by two
      const props: (NodePropSource | [NodeProp<any>, any])[] = [...(spec.props ?? [])]

      // handle lowercased names
      if (spec.name && spec.name[0].toUpperCase() !== spec.name[0]) {
        props.push(styleTags({ [`${spec.name}/...`]: tags[spec.name] ?? NodeType.none }))
      }

      // create node now that props and id are resolved
      const node = NodeType.define({ ...spec, name: spec.name || undefined, id, props })

      map.set(name ?? spec.name, id)
      map.set(id, name ?? spec.name)
      types.push(node)

      return node
    }

    return null
  }

  /** Extends the current `NodeSet` with additional configuration. */
  configure(config: ParserConfiguration) {
    if ("props" in config) this.set = this.set.extend(...config.props!)
  }
}
