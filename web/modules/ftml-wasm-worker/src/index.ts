import { AbstractWorkerBase } from "@wikijump/comlink"
import type { FTMLModule } from "./worker"
import FTMLRemoteWorker from "./worker?worker"

export type {
  Backlinks,
  DetailRenderedHTML,
  DetailRenderedText,
  HTMLMeta,
  PageInfo,
  PartialInfo,
  RenderedHTML,
  RenderSettings,
  SyntaxTree,
  Token,
  UTF16IndexMapFunction,
  Warning,
  WikitextMode,
  WikitextSettings
} from "@wikijump/ftml-wasm"
export * from "./fragment"

export class FTMLWorker extends AbstractWorkerBase.of<FTMLModule>([
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
  "warnings",
  "wordCount"
]) {
  protected _baseGetWorker() {
    return new FTMLRemoteWorker()
  }

  protected async _baseInitalize() {
    await this.worker!.waitUntilReady()
  }
}

export const FTML = new FTMLWorker()

export default FTML
