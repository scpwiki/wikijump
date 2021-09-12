import type { PartialInfo } from "ftml-wasm"
import wasmRelativeURL from "ftml-wasm/vendor/ftml_bg.wasm?url"
import { decode, transfer, WorkerModule } from "threads-worker-module"
import type { FTMLWorkerInterface } from "./worker/ftml.worker"

const wasmURL = new URL(wasmRelativeURL, import.meta.url).toString()

async function importFTML() {
  return (await import("./worker/ftml.worker?worker")).default
}

class FTMLWorker extends WorkerModule<FTMLWorkerInterface> {
  constructor() {
    super("ftml-wasm-worker", importFTML, {
      persist: true,
      init: async () => {
        await this.invoke("init", wasmURL)
      }
    })
  }

  /** Returns FTML's (the crate) version. */
  async version() {
    return decode(await this.invoke("version"))
  }

  async preprocess(str: string) {
    return decode(await this.invoke("preprocess", transfer(str)))
  }

  async tokenize(str: string) {
    return await this.invoke("tokenize", transfer(str))
  }

  async parse(str: string, info?: PartialInfo) {
    return await this.invoke("parse", transfer(str), info)
  }

  async renderHTML(str: string, info?: PartialInfo, format = false) {
    const [html, styles] = await this.invoke("renderHTML", transfer(str), info, format)
    return { html: decode(html), styles: styles.map(buffer => decode(buffer)) }
  }

  async detailRenderHTML(str: string, info?: PartialInfo) {
    return await this.invoke("detailRenderHTML", transfer(str), info)
  }

  async renderText(str: string, info?: PartialInfo) {
    return decode(await this.invoke("renderText", transfer(str), info))
  }

  async detailRenderText(str: string, info?: PartialInfo) {
    return await this.invoke("detailRenderText", transfer(str), info)
  }

  async warnings(str: string, info?: PartialInfo) {
    return await this.invoke("warnings", transfer(str), info)
  }

  async inspectTokens(str: string) {
    return decode(await this.invoke("inspectTokens", transfer(str)))
  }
}

export default new FTMLWorker()
