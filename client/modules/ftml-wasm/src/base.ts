import initFTML, * as Binding from "../vendor/ftml"
// import wasmURL from "../vendor/ftml_bg.wasm?url"

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
export async function init(path: Binding.InitInput) {
  wasm = await initFTML(path)
  ready = true
  resolveLoading()
}

/** Safely frees any WASM objects provided. */
export function free(...objs: any) {
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
  free(...tracked)
  tracked.clear()
}
