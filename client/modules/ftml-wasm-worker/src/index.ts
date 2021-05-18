import type * as FTML from "ftml-wasm"
import type * as Binding from "ftml-wasm/vendor/ftml"
import wasmRelativeURL from "ftml-wasm/vendor/ftml_bg.wasm?url"
import { BlobWorker, ModuleThread, spawn, Thread, Transfer } from "threads"
import { sleep } from "wj-util"
// imports the worker as a chunk of text
import workerText from "./worker/ftml.worker?bundled-worker"

export * from "./fragment"

const wasmURL = new URL(wasmRelativeURL, import.meta.url).toString()

interface TypedArray extends ArrayBuffer {
  buffer: ArrayBufferLike
}
type TransferInput = string | ArrayBuffer | TypedArray

const decoder = new TextDecoder()
const encoder = new TextEncoder()

const transfer = (buffer: TransferInput) => {
  if (typeof buffer === "string") return Transfer(encoder.encode(buffer).buffer)
  if ("buffer" in buffer) return Transfer(buffer.buffer)
  if (buffer instanceof ArrayBuffer) return Transfer(buffer)
  throw new TypeError("Expected a string, ArrayBuffer, or typed array!")
}

const decode = (buffer: ArrayBuffer) => decoder.decode(buffer)

// -- WORKER MODULE

// TODO: refactor this out into its own module if it gets used enough

interface WorkerModuleOpts {
  persist?: boolean
  timeout?: number
  init?: AnyFunction
}

class WorkerModule {
  name: string
  src: string
  worker!: ModuleThread

  private persist = false
  private timeout = 10000
  private init?: AnyFunction

  constructor(name: string, src: string, opts?: WorkerModuleOpts) {
    this.name = name
    this.src = src
    if (opts) {
      this.persist = opts.persist ?? false
      this.timeout = opts.timeout ?? 10000
      this.init = opts.init
    }
  }

  private async ready() {
    if (!this.worker) {
      this.worker = await spawn<ModuleThread>(
        BlobWorker.fromText(this.src, { name: this.name })
      )
      if (this.init) await this.init()
    }
  }

  private async terminate() {
    if (this.worker) await Thread.terminate(this.worker)
    // @ts-ignore
    this.worker = undefined
  }

  private async restart() {
    await this.terminate()
    await this.ready()
  }

  async invoke<T>(fn: () => Promise<T>) {
    await this.ready()
    const result = this.timeout
      ? await Promise.race([fn(), sleep(this.timeout)])
      : await fn()
    if (result) {
      if (!this.persist) await this.terminate()
      return result
    } else {
      if (this.persist) await this.restart()
      else await this.terminate()
      throw new Error("Worker timed out!")
    }
  }
}

const module = new WorkerModule("ftml-wasm-worker", workerText, {
  persist: true,
  init() {
    module.invoke(() => module.worker.init(wasmURL))
  }
})
const invoke = module.invoke.bind(module)

/** Returns FTML's (the crate) version. */
export async function version() {
  return decode(await invoke<ArrayBuffer>(() => module.worker.version()))
}

/**
 * Preprocesses a string of wikitext.
 * See `ftml/src/preproc/test.rs` for more information.
 */
export async function preprocess(str: string) {
  return decode(await invoke<ArrayBuffer>(() => module.worker.preprocess(transfer(str))))
}

/** Tokenizes a string of wikitext. */
export async function tokenize(str: string) {
  type Return = Binding.IToken[]
  return await invoke<Return>(() => module.worker.tokenize(transfer(str)))
}

/**
 * Parses a string of wikitext. This returns an AST and warnings list, not HTML.
 * @see {@link render}
 */
export async function parse(str: string) {
  type Return = ReturnType<typeof FTML["parse"]>
  return await invoke<Return>(() => module.worker.parse(transfer(str)))
}

/** Renders a string of wikitext to HTML. */
export async function render(str: string, format = false) {
  const [htmlBuffer, styleBuffer] = await invoke<[ArrayBuffer, ArrayBuffer[]]>(() =>
    module.worker.render(transfer(str), format)
  )
  const html = decode(htmlBuffer)
  const styles = styleBuffer.map(buffer => decode(buffer))
  return { html, styles }
}

/** Renders a string of wikitext to text. */
export async function renderText(str: string) {
  return decode(await invoke<ArrayBuffer>(() => module.worker.renderText(transfer(str))))
}

/**
 * Renders a string of wikitext like the {@link renderHTML} function, but this
 * function additionally returns every step in the rendering pipeline.
 */
export async function detailedRender(str: string) {
  type Return = ReturnType<typeof FTML["detailedRender"]>
  return await invoke<Return>(() => module.worker.detailedRender(transfer(str)))
}

/** Returns the list of warnings emitted when parsing the provided string. */
export async function warnings(str: string) {
  type Return = Binding.IParseWarning[]
  return await invoke<Return>(() => module.worker.warnings(transfer(str)))
}

/** Converts a string of wikitext into a pretty-printed list of tokens. */
export async function inspectTokens(str: string) {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.inspectTokens(transfer(str)))
  )
}
