import wasmRelativeURL from "ftml-wasm/vendor/ftml_bg.wasm?url"
import { decode, transfer, WorkerModule } from "threads-worker-module"
import type { FTMLWorkerInterface } from "./worker/ftml.worker"

const wasmURL = new URL(wasmRelativeURL, import.meta.url).toString()

async function importFTML() {
  return (await import("./worker/ftml.worker?bundled-worker")).default
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

  /**
   * Preprocesses a string of wikitext. See `ftml/src/preproc/test.rs` for
   * more information.
   */
  async preprocess(str: string) {
    return decode(await this.invoke("preprocess", transfer(str)))
  }

  /** Tokenizes a string of wikitext. */
  async tokenize(str: string) {
    return await this.invoke("tokenize", transfer(str))
  }

  /**
   * Parses a string of wikitext. This returns an AST and warnings list, not HTML.
   *
   * @see {@link FTMLWorker#render}
   */
  async parse(str: string) {
    return await this.invoke("parse", transfer(str))
  }

  /**
   * Renders a string of wikitext to HTML.
   *
   * @param format - Pretty-prints the HTML if true. Does not preserve whitespace.
   */
  async render(str: string, format = false) {
    const [htmlBuffer, stylesBuffer] = await this.invoke("render", transfer(str), format)
    const html = decode(htmlBuffer)
    const styles = stylesBuffer.map(buffer => decode(buffer))
    return { html, styles }
  }

  /** Renders a string of wikitext to text. */
  async renderText(str: string) {
    return decode(await this.invoke("renderText", transfer(str)))
  }

  /**
   * Renders a string of wikitext like the {@link FTMLWorker#render}
   * function, but this function additionally returns every step in the
   * rendering pipeline.
   */
  async detailedRender(str: string) {
    return await this.invoke("detailedRender", transfer(str))
  }

  /** Returns the list of warnings emitted when parsing the provided string. */
  async warnings(str: string) {
    return await this.invoke("warnings", transfer(str))
  }

  /** Converts a string of wikitext into a pretty-printed list of tokens. */
  async inspectTokens(str: string) {
    return decode(await this.invoke("inspectTokens", transfer(str)))
  }
}

export default new FTMLWorker()
