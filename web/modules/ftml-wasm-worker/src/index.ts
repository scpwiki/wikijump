import { AbstractWorkerBase } from "@wikijump/comlink"
import FTMLRemoteWorker from "./worker?worker"

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

export class FTMLWorker extends AbstractWorkerBase.of([
  "detailRenderHTML",
  "detailRenderText",
  "formatHTML",
  "getUTF16IndexMap",
  "inspectTokens",
  "makeInfo",
  "parse",
  "preprocess",
  "renderHTML",
  "renderText",
  "tokenize",
  "version",
  "waitUntilReady",
  "warnings"
]) {
  protected createWorker() {
    return new FTMLRemoteWorker()
  }

  async methodCondition() {
    await this.worker!.waitUntilReady()
  }
}

export const FTML = new FTMLWorker()

export default FTML
