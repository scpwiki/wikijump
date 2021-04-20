import { spawn, Thread, Worker, Transfer, ModuleThread } from "threads"
import { sleep } from "wj-util"
import type * as FTML from "ftml-wasm"
import type * as Binding from "ftml-wasm/vendor/ftml"

import url from "./worker/ftml.worker.ts"

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
}

class WorkerModule {
  name: string
  persist = false
  timeout = 10000
  url: URL
  worker!: ModuleThread

  constructor(name: string, url: string, opts?: WorkerModuleOpts) {
    this.name = name
    this.url = new URL(url, import.meta.url)
    if (opts) {
      if (opts.persist) this.persist = true
      this.timeout = opts.timeout ?? 10000
    }
  }

  private async _ready() {
    if (!this.worker) {
      this.worker = await spawn<ModuleThread>(
        new Worker(this.url as any, {
          name: this.name,
          credentials: "same-origin",
          type: "classic"
        })
      )
    }
  }

  private async _terminate() {
    if (this.worker) await Thread.terminate(this.worker)
    this.worker = undefined as any
  }

  private async _restart() {
    await this._terminate()
    await this._ready()
  }

  async invoke<T>(fn: () => Promise<T>) {
    await this._ready()
    const result = this.timeout
      ? await Promise.race([fn(), sleep(this.timeout)])
      : await fn()
    if (result) {
      if (!this.persist) await this._terminate()
      return result
    } else {
      if (this.persist) await this._restart()
      else await this._terminate()
      throw new Error("Worker timed out!")
    }
  }
}

const module = new WorkerModule("ftml-wasm-worker", url, { persist: true })
const invoke = module.invoke.bind(module)

export async function version() {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.version())
  )
}

export async function tokenize(str: string) {
  type Return = Binding.IToken[]
  return await invoke<Return>(() => module.worker.tokenize(transfer(str)))
}

export async function parse(str: string) {
  type Return = ReturnType<typeof FTML["parse"]>
  return await invoke<Return>(() => module.worker.parse(transfer(str)))
}

export async function renderHTML(str: string) {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.renderHTML(transfer(str)))
  )
}

export async function renderStyle(str: string) {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.renderStyle(transfer(str)))
  )
}

export async function renderText(str: string) {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.renderText(transfer(str)))
  )
}

export async function detailedRender(str: string) {
  type Return = ReturnType<typeof FTML["detailedRender"]>
  return await invoke<Return>(() => module.worker.detailedRender(transfer(str)))
}

export async function warnings(str: string) {
  type Return = Binding.IParseWarning[]
  return await invoke<Return>(() => module.worker.warnings(transfer(str)))
}

export async function inspectTokens(str: string) {
  return decode(
    await invoke<ArrayBuffer>(() => module.worker.inspectTokens(transfer(str)))
  )
}
