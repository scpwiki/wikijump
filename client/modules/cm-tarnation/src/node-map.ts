import { styleTags, tags } from "@codemirror/highlight"
import { NodeProp, NodePropSource, NodeSet, NodeType } from "lezer-tree"
import type { AddNodeSpec, ParserConfiguration } from "./types"

export class NodeMap {
  map = new Map<string, number>()
  types: NodeType[] = []
  set = new NodeSet(this.types)

  // @ts-ignore
  private tags: Record<string, Tag> = { ...tags }

  get(name: string) {
    return this.map.get(name)
  }

  add(spec: AddNodeSpec, name?: string): NodeType | null
  add(spec: NodeType, name: string): NodeType | null
  add(spec: AddNodeSpec | NodeType, name?: string): NodeType | null {
    const { map, types, tags } = this
    if (spec instanceof NodeType && name) {
      map.set(name, spec.id)
      types.push(spec)
      return spec
    }
    if (!(spec instanceof NodeType)) {
      /*
       * There is two ways a node can be interpreted:
       * 1. The node's name is lowercased. That means it is a shortcut for a
       *    CodeMirror highlighting tag.
       * 2. The node's name is capitalized. That means it is a custom name,
       *    and no assumptions will be made about its highlighting or styling.
       */
      const id = map.size
      const props: (NodePropSource | [NodeProp<any>, any])[] = [...(spec.props ?? [])]
      if (spec.name && spec.name[0].toUpperCase() !== spec.name[0]) {
        props.push(styleTags({ [`${spec.name}/...`]: tags[spec.name] ?? NodeType.none }))
      }
      const node = NodeType.define({ ...spec, name: spec.name || undefined, id, props })
      map.set(name ?? spec.name, id)
      types.push(node)
      return node
    }
    return null
  }

  configure(config: ParserConfiguration) {
    if ("props" in config) this.set = this.set.extend(...config.props!)
  }
}
