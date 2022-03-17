import { AbstractWorkerBase } from "@wikijump/comlink"
import wasmURL from "@wikijump/ftml-wasm/vendor/ftml_bg.wasm?url"
import type { FTMLModule } from "./worker"

export type {
  Backlinks,
  DetailRenderedHTML,
  DetailRenderedText,
  HTMLMeta,
  Page,
  PageInfo,
  ParseResult,
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
  "init",
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
  "wordCount",
  "Page"
]) {
  protected _baseGetWorker() {
    return new Worker(new URL("./worker", import.meta.url), { type: "classic" })
  }

  protected async _baseInitalize() {
    await this.worker!.init(wasmURL)
    await this.worker!.waitUntilReady()
  }
}

export const FTML = new FTMLWorker()

export default FTML
