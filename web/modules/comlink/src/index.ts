/* eslint-disable @typescript-eslint/ban-types */
import { timedout, TIMED_OUT_SYMBOL } from "@wikijump/util"
import * as Comlink from "comlink"

const DEFAULT_TIMEOUT = 5000

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
export type RemoteWorker<T> = Worker | Remote<T>

export type BoundWorker<T> = AbstractWorkerBase<T> & Methods<RemoteObject<T>>

export abstract class AbstractWorkerBase<T> {
  /**
   * Tracks if the worker is still being created. Will be undefined if the
   * worker is stopped or fully started.
   */
  declare starting?: Promise<void>

  /** The worker instance. */
  declare worker?: Remote<T>

  /** Required function needed for getting a `Worker` or `Comlink.Remote<T>` instance. */
  protected abstract _baseGetWorker(): Promisable<RemoteWorker<T> | false>

  /** An optional function that will be ran whenever a new worker is created. */
  protected _baseInitalize?(): Promisable<void>

  /**
   * Object that allows for setting a default return value function when a
   * worker couldn't be started, or if a `_baseBeforeMethod` check failed.
   */
  protected _baseDefaults?: {
    [P in keyof Methods<T>]?: RemoteObject<T>[P] extends (...args: infer A) => infer R
      ? Functionable<Promisable<Awaited<R>>, A, this>
      : Functionable<Promisable<Awaited<RemoteObject<T>[P]>>, void, this>
  }

  /**
   * Number of milliseconds a function can run before an error is thrown
   * and the worker is stopped. Defaults to 5000. Set to 0 to disable.
   */
  protected _baseMethodTimeout?: number

  /**
   * If a worker was provided, its instance will be kept here so that it
   * can be forcefully terminated.
   */
  private _workerInstance?: Worker

  /**
   * Function intended to be used within an `extends` expression.
   * Constructs a class with a prototype that binds the given methods of
   * the worker proxy. Constructors (classes) count as "methods" too.
   *
   * This function doesn't have to be used - it's just a convenience.
   * However, you'll have to create your own methods that call out to the
   * worker. Note that calling worker methods directly will not have any
   * protections.
   *
   * @param methods - The methods to bind. These can't be figured out automatically.
   */
  static of<T>(methods: (keyof Methods<T>)[]): AbstractClass<BoundWorker<T>> {
    // @ts-ignore
    const Derived: AbstractClass<BoundWorker<T>> = class extends AbstractWorkerBase<T> {}

    for (const method of methods) {
      Derived.prototype[method] = async function (this: BoundWorker<T>, ...args: any[]) {
        if (this.starting) await this.starting
        if (!this.worker) await this.start()

        // check one more time - maybe worker couldn't start
        if (!this.worker) return await this._baseTryToGetDefault(method, args)

        if (this._baseMethodTimeout !== 0) {
          const result = await timedout(
            // @ts-ignore
            this.worker![method](...args),
            this._baseMethodTimeout ?? DEFAULT_TIMEOUT
          )

          if (result !== TIMED_OUT_SYMBOL) return result

          // worker is timing out, have to stop it
          this.stop()
          throw new Error(`Method "${method}" timed out!`)
        }
      }
    }

    return Derived
  }

  /** Tries to run a default method if the worker couldn't be started. */
  private async _baseTryToGetDefault(method: keyof Methods<T>, args: any[]) {
    if (!this._baseDefaults || !this._baseDefaults.hasOwnProperty(method)) {
      if (!this.worker) throw new Error(`Worker could not be started!`)
      else throw new Error(`Method "${method}" could not be called!`)
    }

    const def = this._baseDefaults[method]

    if (typeof def === "function") {
      return await def.apply(this, args)
    } else {
      return def
    }
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
    const old = [this.worker, this._workerInstance] as const

    const result = this._baseGetWorker()
    if (result instanceof Promise) this.starting = result.then()
    const worker: RemoteWorker<T> | false = await result

    if (worker) {
      if (worker instanceof Worker) {
        this.worker = Comlink.wrap<T>(worker)
        this._workerInstance = worker
      } else {
        this.worker = worker
        this._workerInstance = undefined
      }

      if (this._baseInitalize) await this._baseInitalize()

      if (old[0]) releaseRemote(old[0])
      if (old[1]) old[1].terminate()

      this.starting = undefined
    }
  }

  /** Stops the worker. Needed for garbage collection. */
  stop() {
    if (this.worker) releaseRemote(this.worker)
    if (this._workerInstance) this._workerInstance.terminate()
    this.worker = undefined
    this._workerInstance = undefined
  }
}
