import { decode, encode } from "@wikijump/util"
import * as Comlink from "comlink"

/**
 * Releases a remote proxy. This is required to prevent memory leaks.
 *
 * @param remote - The proxy to release.
 */
export function releaseRemote<T>(remote: Comlink.Remote<T>) {
  if (remote[Comlink.releaseProxy]) remote[Comlink.releaseProxy]()
}

export const transfer = Comlink.transfer

export type {
  Endpoint,
  Local,
  LocalObject,
  ProxyMarked,
  ProxyMethods,
  ProxyOrClone,
  Remote,
  RemoteObject,
  TransferHandler,
  UnproxyOrClone
} from "comlink"
export { Comlink }

export type DerivedWorkerBase<T> = AbstractWorkerBase<T> & Comlink.RemoteObject<T>

export abstract class AbstractWorkerBase<T> {
  /** True if the worker has already been terminated. */
  declare terminated: boolean

  /** Tracks if the worker is still be created. Prevents a race condition. */
  declare starting?: Promise<void>

  /** Required function needed for getting a `Worker` or `Comlink.Remote<T>` instance. */
  protected abstract _baseGetWorker(): Promisable<Worker | Comlink.Remote<T> | false>

  /**
   * An optional function that will be called before each method call. If
   * it returns a boolean, the method will not be called if the value is false.
   */
  protected _baseBeforeMethod?(): Promisable<boolean | void>

  /** An optional function that will be ran whenever a new worker is created. */
  protected _baseInitalize?(): Promisable<void>

  /** The worker instance. */
  declare worker?: Comlink.Remote<T>

  constructor() {
    this.terminated = false
  }

  /**
   * Function intended to be used within an `extends` expression.
   * Constructs a class that wraps around a worker factory function and
   * automatically handles binding methods and worker creation.
   *
   * @param methods - The methods to bind to the class instance, which when
   *   called will be passed the worker instance.
   */
  static of<T>(methods: (keyof T)[]): abstract new () => DerivedWorkerBase<T> {
    // @ts-ignore
    const Derived: new () => DerivedWorkerBase<T> = class extends AbstractWorkerBase<T> {}

    for (const method of methods) {
      Derived.prototype[method] = async function (
        this: DerivedWorkerBase<T>,
        ...args: any[]
      ) {
        if (this.terminated) throw new Error("Worker was already terminated!")
        if (this.starting) await this.starting
        if (!this.worker) await this.start()

        // check one more time - maybe worker couldn't start
        if (!this.worker) throw new Error("Worker could not be started!")

        if (this._baseBeforeMethod) {
          const value = await this._baseBeforeMethod()
          if (typeof value === "boolean" && !value) return
        }

        // @ts-ignore
        const result = await this.worker[method](...args)
        return result
      }
    }

    return Derived
  }

  /** True if the worker has been started. */
  get loaded() {
    return Boolean(this.worker)
  }

  /**
   * Starts the worker.
   *
   * @param force - If true, the worker will be restarted even if it has
   *   already been started.
   */
  async start(force?: boolean) {
    if (!force && this.worker) return
    if (this.starting) {
      await this.starting
      this.starting = undefined
      if (!force) return
    }
    let oldWorker = this.worker
    const result = this._baseGetWorker()
    if (result instanceof Promise) this.starting = result.then()
    const worker: Comlink.Remote<T> | Worker | false = await result
    if (!worker) return
    if (worker instanceof Worker) this.worker = Comlink.wrap<T>(worker)
    else this.worker = worker
    if (this._baseInitalize) await this._baseInitalize()
    if (oldWorker) releaseRemote(oldWorker)
    this.starting = undefined
  }

  /** Stops the worker, but will still allow method calls to restart it. */
  stop() {
    if (this.worker) releaseRemote(this.worker)
    this.worker = undefined
  }

  /** Terminates the worker. */
  terminate() {
    this.stop()
    this.terminated = true
  }
}

export class TransferString {
  readonly buffer: ArrayBuffer

  constructor(str: string | ArrayBufferView | ArrayBufferLike) {
    this.buffer = encode(str)
  }

  toString() {
    return decode(this.buffer)
  }

  static handler: Comlink.TransferHandler<TransferString, ArrayBuffer> = {
    canHandle(value: unknown): value is TransferString {
      return value instanceof TransferString
    },

    serialize(value: TransferString) {
      const serialized = encode(value.buffer)
      return [serialized, [serialized]]
    },

    deserialize(buffer: ArrayBuffer) {
      return new TransferString(buffer)
    }
  }
}

Comlink.transferHandlers.set("TransferString", TransferString.handler)
