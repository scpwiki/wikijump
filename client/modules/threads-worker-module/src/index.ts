import {
  BlobWorker,
  ModuleThread,
  spawn,
  Thread,
  Transfer,
  TransferDescriptor
} from "threads"
import type { WorkerModule as WorkerModuleMethods } from "threads/dist/types/worker"
import { sleep } from "wj-util"

export { Transfer as transferMultiple } from "threads"

const decoder = new TextDecoder()
const encoder = new TextEncoder()

/** Convert a string or generic buffer into an `ArrayBuffer` that can be transferred. */
export function encode(buffer: string | ArrayBufferLike | ArrayBufferView) {
  if (typeof buffer === "string") return encoder.encode(buffer).buffer
  if ("buffer" in buffer) return buffer.buffer
  if (buffer instanceof ArrayBuffer) return buffer
  throw new TypeError("Expected a string, ArrayBuffer, or typed array!")
}

/** Decode an `ArrayBuffer` into a string. */
export function decode(buffer: ArrayBuffer) {
  return decoder.decode(buffer)
}

/**
 * Mark a transferable object as such, so not to be serialized and
 * deserialized on messaging with the main thread, but rather to transfer
 * ownership of it to the receiving thread.
 */
export function transfer(raw: string | ArrayBufferLike | ArrayBufferView) {
  return Transfer(encode(raw)) as TransferDescriptor<ArrayBuffer>
}

interface WorkerModuleOpts {
  /** If true, the worker will persist and be available even after work is complete. */
  persist?: boolean
  /**
   * If true, the worker will be terminated and restarted if it an error
   * occurs for it.
   */
  restartOnError?: boolean
  /** Restarts the worker after this number of milliseconds if it is still working. */
  timeout?: number | false
  /** If provided, this function will be executed any time the worker is started. */
  init?: AnyFunction
}

/**
 * Wrapper around a Threads.js worker module.
 *
 * @typeParam Methods - An interface describing the methods that can be
 *   invoked on the worker. You may need to use Thread's `ModuleProxy` to
 *   get the right interface.
 */
export class WorkerModule<Methods extends WorkerModuleMethods<string> = any> {
  private declare worker?: ModuleThread<Methods>
  private declare workerLoading?: Promise<ModuleThread<Methods>>

  /**
   * @param name - Name of the worker, which appears in the debugger.
   * @param src - The source for a worker, resolving into a string or
   *   worker factory. If the source is given directly, it can only be a
   *   string. Otherwise, a function resolving into a string or a worker
   *   factory must be given.
   * @param workerConfig - Worker lifecycle configuration.
   */
  constructor(
    protected name: string,
    protected src: Promisable<string> | (() => Promisable<string | (new () => Worker)>),
    protected workerConfig: WorkerModuleOpts = {
      persist: false,
      timeout: 10000,
      restartOnError: false
    }
  ) {}

  /**
   * Ensures that the internal worker is started if it isn't already, and
   * then returns it.
   */
  private async ensureWorker() {
    if (this.worker) return this.worker
    if (this.workerLoading) return await this.workerLoading

    this.workerLoading = new Promise<ModuleThread<Methods>>(async resolve => {
      if (!this.worker) {
        const resolved = await this.src
        const src = typeof resolved === "string" ? resolved : await resolved()
        const webworker =
          typeof src === "string"
            ? BlobWorker.fromText(src, { name: this.name })
            : new src()
        this.worker = (await spawn(webworker)) as ModuleThread<Methods>
        if (this.workerConfig.init) await this.workerConfig.init()
      }
      resolve(this.worker!)
      this.workerLoading = undefined
    })

    return await this.workerLoading
  }

  /** Terminates the internal worker. */
  private async terminateWorker() {
    if (this.worker) await Thread.terminate(this.worker)
    this.worker = undefined
  }

  /** Restarts the internal worker. */
  private async restartWorker() {
    await this.terminateWorker()
    return await this.ensureWorker()
  }

  /**
   * Invokes an internal worker method safely.
   *
   * @param method - The method on the worker to invoke.
   * @param args - The arguments to invoke the method with.
   */
  protected async invoke<T extends keyof Methods>(
    method: T,
    ...args: Parameters<Methods[T]>
  ): Promise<ReturnType<Methods[T]>> {
    let result: ReturnType<Methods[T]>

    try {
      const worker = await this.ensureWorker()

      if (this.workerConfig.timeout) {
        let timedout = false
        const timeout = sleep(this.workerConfig.timeout).then(
          () => void (timedout = true)
        )
        const race = (await Promise.race([
          worker[method](...args),
          timeout
        ])) as ReturnType<Methods[T]>

        if (timedout) throw new Error(`Worker timed out! Method: "${method}"`)

        result = race
      } else {
        result = await worker[method](...args)
      }
    } catch (err) {
      if (this.workerConfig.restartOnError && this.workerConfig.persist) {
        await this.restartWorker()
      }
      throw err
    }

    if (!this.workerConfig.persist) await this.terminateWorker()

    return result
  }
}
