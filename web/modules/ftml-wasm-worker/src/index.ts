import { bindMethods, Comlink, type Remote, type RemoteObject } from "@wikijump/comlink"
import type { FTMLModule } from "./worker"

export type {
  Backlinks,
  HTMLMeta,
  PageInfo,
  PartialInfo,
  SyntaxTree,
  Token,
  Warning
} from "@wikijump/ftml-wasm"
export * from "./fragment"

type RemoteFTML = Remote<FTMLModule>

export class FTMLWorker implements RemoteObject<FTMLModule> {
  declare makeInfo: RemoteFTML["makeInfo"]
  declare version: RemoteFTML["version"]
  declare preprocess: RemoteFTML["preprocess"]
  declare tokenize: RemoteFTML["tokenize"]
  declare parse: RemoteFTML["parse"]
  declare renderHTML: RemoteFTML["renderHTML"]
  declare detailRenderHTML: RemoteFTML["detailRenderHTML"]
  declare renderText: RemoteFTML["renderText"]
  declare detailRenderText: RemoteFTML["detailRenderText"]
  declare warnings: RemoteFTML["warnings"]
  declare getUTF16IndexMap: RemoteFTML["getUTF16IndexMap"]
  declare inspectTokens: RemoteFTML["inspectTokens"]
  declare formatHTML: RemoteFTML["formatHTML"]
  declare waitUntilReady: RemoteFTML["waitUntilReady"]

  declare worker: RemoteFTML

  constructor() {
    this.init()
  }

  private async init() {
    this.worker = Comlink.wrap<FTMLModule>(
      new (await import("./worker?worker")).default()
    )

    bindMethods({
      target: this,
      worker: this.worker,
      methods: [
        "makeInfo",
        "version",
        "preprocess",
        "tokenize",
        "parse",
        "renderHTML",
        "detailRenderHTML",
        "renderText",
        "detailRenderText",
        "warnings",
        "getUTF16IndexMap",
        "inspectTokens",
        "formatHTML",
        "waitUntilReady"
      ],
      check: async () => {
        await this.worker.waitUntilReady()
      }
    })
  }
}

export const FTML = new FTMLWorker()

export default FTML
