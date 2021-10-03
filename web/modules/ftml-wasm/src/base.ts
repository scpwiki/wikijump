import initFTML, * as Binding from "../vendor/ftml"
// TODO: get rid of this dumb import
// stupid vite bug means that I can't dynamically import in a worker
// so I have to do this
import wasmURL from "../vendor/ftml_bg.wasm?url"

/** Indicates if the WASM binding is loaded. */
export let ready = false

let resolveLoading: (value?: unknown) => void
/** Promise that resolves when the WASM binding has loaded. */
export const loading = new Promise(resolve => {
  resolveLoading = resolve
})

/** Actual output of the WASM instantiation. */
export let wasm: Binding.InitOutput | null = null

/** Loads the WASM required for the FTML library. */
export async function init(path?: Binding.InitInput) {
  // TODO: uncomment this as soon as Vite stops being bad
  // see TODO above this one
  // if (!path) path = (await import("../vendor/ftml_bg.wasm?url")).default
  if (!path) path = new URL(wasmURL, location.href)
  wasm = await initFTML(path)
  ready = true
  resolveLoading()
}

/** Safely frees any WASM objects provided. */
export function free(objs: Set<any>) {
  for (const obj of objs) {
    if (typeof obj !== "object" || !("ptr" in obj)) continue
    if (obj.ptr !== 0) obj.free()
  }
}

/**
 * This set contains unfreed WASM objects. It is separate from any
 * particular function so that error recovery can still clear memory.
 */
const tracked = new Set<any>()

/** Adds a WASM object to the list of tracked objects. */
export function trk<T>(obj: T): T {
  tracked.add(obj)
  return obj
}

/** Frees all objects being {@link tracked}, and clears the set. */
export function freeTracked() {
  // use setTimeout so that we don't stall a function clearing memory
  // this speeds up returning objects, especially in a worker
  setTimeout(() => {
    free(tracked)
    tracked.clear()
  })
}
