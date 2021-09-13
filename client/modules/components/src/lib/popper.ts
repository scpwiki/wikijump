import * as Popper from "@popperjs/core"

export interface PopoverOpts {
  /** When true, the element will be placed in the desired position. Defaults to true. */
  when?: boolean
  /** Offset position to place the element. Defaults to `"auto"`. */
  placement?: Popper.Placement
  /** The target element to position against. Defaults to the parent element. */
  target?: Element | null
}

/**
 * Svelte use function that uses PopperJS to place an element relative to
 * another element.
 */
export function popover(elem: Element, opts: PopoverOpts) {
  let instance: Popper.Instance | undefined

  const destroy = () => {
    if (instance) {
      instance.destroy()
      instance = undefined
    }
  }

  const update = ({
    when = true,
    placement = "auto",
    target = elem.parentElement
  }: PopoverOpts) => {
    if ((!when && instance) || (instance && !target)) {
      destroy()
    } else if (when && target) {
      if (instance) destroy()
      if (target) {
        instance = Popper.createPopper(target, elem as HTMLElement, {
          placement: placement
        })
      }
    }
  }

  update(opts)

  return { update, destroy }
}
