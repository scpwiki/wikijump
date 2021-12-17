export interface GuardOpts<T> {
  /** When true, the `use` function will be applied. */
  when: boolean

  /** The `use` function to guard. */
  use: [
    (node: HTMLElement, opts: T) => { update: (opts: T) => void; destroy: () => void },
    T
  ]
}

/**
 * Helper that allows you to use a `use` function with a guard condition.
 *
 * @param node - The element to use the guard on.
 * @param opts - Options for the guard.
 */
export function guard<T>(node: HTMLElement, opts: GuardOpts<T>) {
  let applied: null | ReturnType<typeof opts["use"][0]> = null

  const update = (opts: GuardOpts<T>) => {
    if (opts.when) {
      if (!applied) {
        applied = opts.use[0](node, opts.use[1])
      } else {
        applied.update(opts.use[1])
      }
    } else if (applied) {
      applied.destroy()
      applied = null
    }
  }

  update(opts)

  return {
    update,
    destroy() {
      if (applied) applied.destroy()
      applied = null
    }
  }
}
