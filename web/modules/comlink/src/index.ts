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
  TransferHandler,
  UnproxyOrClone
} from "comlink"
export { Comlink }

export type Remote<T> = Comlink.Remote<T>
export type RemoteObject<T> = Comlink.RemoteObject<T>
export type AbstractRemoteWorker<T> = Worker | Remote<T>

export type DerivedWorkerBase<T> = AbstractWorkerBase<T> & RemoteObject<T>

export abstract class AbstractWorkerBase<T> {
  /** Tracks if the worker is still be created. Prevents a race condition. */
  declare starting?: Promise<void>

  /** Required function needed for getting a `Worker` or `Comlink.Remote<T>` instance. */
  protected abstract _baseGetWorker(): Promisable<AbstractRemoteWorker<T> | false>

  /**
   * An optional function that will be called before each method call. If
   * it returns a boolean, the method will not be called if the value is false.
   */
  protected _baseBeforeMethod?(): Promisable<boolean | void>

  /** An optional function that will be ran whenever a new worker is created. */
  protected _baseInitalize?(): Promisable<void>

  /**
   * Object that allows for setting a default return value function when a
   * worker couldn't be started, or if a `_baseBeforeMethod` check failed.
   */
  protected _baseDefaults?: {
    [P in keyof T]?: RemoteObject<T>[P] extends (...args: infer A) => infer R
      ? (this: this, ...args: A) => R
      : (this: this) => RemoteObject<T>
  }

  /** The worker instance. */
  protected declare worker?: Remote<T>

  /**
   * Function intended to be used within an `extends` expression.
   * Constructs a class with a prototype that has all the methods and
   * properties of the worker proxy.
   *
   * @param props - The properties to bind. These can't be figured out automatically.
   */
  static of<T>(props: (keyof T)[]): abstract new () => DerivedWorkerBase<T> {
    // @ts-ignore
    const Derived: new () => DerivedWorkerBase<T> = class extends AbstractWorkerBase<T> {}

    for (const prop of props) {
      Derived.prototype[prop] = async function (
        this: DerivedWorkerBase<T>,
        ...args: any[]
      ) {
        if (this.starting) await this.starting
        if (!this.worker) await this.start()

        // check one more time - maybe worker couldn't start
        if (!this.worker) await this._tryToGetDefault(prop, ...args)

        const workerProperty = this.worker![prop] as unknown

        if (typeof workerProperty === "function") {
          if (this._baseBeforeMethod) {
            const value = await this._baseBeforeMethod()
            if (value === false) await this._tryToGetDefault(prop, ...args)
          }

          // @ts-ignore
          return await workerProperty.call(this.worker!, ...args)
        } else {
          return workerProperty
        }
      }
    }

    return Derived
  }

  /** Tries to run a default method if the worker couldn't be started. */
  private async _tryToGetDefault(method: keyof T, ...args: any[]) {
    if (!this._baseDefaults || !this._baseDefaults[method]) {
      if (!this.worker) throw new Error(`Worker could not be started!`)
      else throw new Error(`Method "${method}" could not be called!`)
    }

    return await this._baseDefaults[method]!.call(this, ...args)
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
    const worker: AbstractRemoteWorker<T> | false = await result
    if (!worker) return
    if (worker instanceof Worker) this.worker = Comlink.wrap<T>(worker)
    else this.worker = worker
    if (this._baseInitalize) await this._baseInitalize()
    if (oldWorker) releaseRemote(oldWorker)
    this.starting = undefined
  }

  /** Stops the worker. Needed for garbage collection. */
  stop() {
    if (this.worker) releaseRemote(this.worker)
    this.worker = undefined
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
