/* eslint-disable @typescript-eslint/unbound-method */
import { scrollElement } from "./scrolling"

export type Point = [x: number, y: number]
export type Direction = "up" | "down" | "left" | "right"
export type Axis = "vertical" | "horizontal"
export type GestureType = "start" | "move" | "end" | "cancel"

/**
 * Represents the state of an in-progress gesture. This class is intended
 * as immutable, it only represents a gesture as it was during one point in time.
 */
export class Gesture {
  /** The starting position. */
  declare start: Point

  /** The end position. */
  declare end: Point

  /** The `[x, y]` difference. */
  declare diff: Point

  /** The `[x, y]` difference, with both values being absolute (positive). */
  declare diffAbs: Point

  /**
   * The {@link Direction} of the gesture, based on the difference between
   * the start and end points.
   */
  declare direction: Direction

  /** The distance moved in CSS pixels. */
  declare dist: number

  /** The {@link GestureType} for this gesture. */
  declare type: GestureType

  /**
   * @param start - The starting position.
   * @param end - The end position.
   * @param type - The {@link GestureType} for this gesture.
   */
  constructor(start: Point, end: Point, type: GestureType = "move") {
    // prettier-ignore
    const x1 = start[0], y1 = start[1],
          x2 = end[0],   y2 = end[1]

    // with these vars: 0 is vertical, 1 is horizontal
    const diff: Point = [y1 - y2, x1 - x2]
    const diffAbs: Point = [Math.abs(diff[0]), Math.abs(diff[1])]
    const axis = diffAbs[1] > diffAbs[0] ? 1 : 0
    const dist = diffAbs[axis]

    // prettier-ignore
    const direction =
      Gesture.Directions[axis * 2 + +(diff[axis] < 0)]
    //                       ^      ^        ^  dir via sign (+ up|left, - down|right)
    //                       ^      ^  convert boolean to integer
    //                       ^ this is either 0 or 2, as axis is either 0 or 1

    this.start = start
    this.end = end
    this.diff = diff
    this.diffAbs = diffAbs
    this.direction = direction
    this.dist = dist
    this.type = type
  }

  /** All four orthogonal directions. */
  static Directions = ["up", "down", "left", "right"] as const

  // stuff we won't bother precalculating

  /**
   * Returns the angle of the direction in degrees. For reference, `0`
   * faces right, with up (`90`), left (`180`), and down (`-90`) following
   * in the counter-clockwise direction. Be aware of the sign flip that
   * occurs when the angle passes 180 degrees.
   */
  get angle() {
    // prettier-ignore
    const x1 = this.start[0], y1 = this.start[1],
          x2 = this.end[0],   y2 = this.end[1]
    return (Math.atan2(y2 - y1, x2 - x1) * 180) / Math.PI
  }

  /** The {@link Axis}, based on the gesture's direction. */
  get axis(): Axis {
    return this.direction === "up" || this.direction === "down"
      ? "vertical"
      : "horizontal"
  }

  /**
   * The "sign" of the gesture, as in multiplying a pixel offset by this
   * value will move an element in the direction of this gesture (assuming
   * the axis is correct).
   */
  get sign(): -1 | 1 {
    return this.direction === "up" || this.direction === "left" ? -1 : 1
  }

  /** True if the gesture is in the vertical direction. */
  get isVertical(): boolean {
    return this.direction === "up" || this.direction === "down"
  }

  /** True if the gesture is in the horizontal direction. */
  get isHorizontal(): boolean {
    return this.direction === "left" || this.direction === "right"
  }

  /**
   * Checks if a {@link Direction} is one of the directions given.
   *
   * @param other - The direction(s) to check against.
   */
  same(other: Direction | Direction[]) {
    return typeof other === "string"
      ? other === this.direction
      : other.includes(this.direction)
  }

  /**
   * Checks if the distance of this gesture exceeds that of a given threshold.
   *
   * @param threshold - The threshold to check against.
   */
  over(threshold?: number) {
    if (threshold === undefined) return false
    return this.dist > threshold
  }

  /**
   * Checks if this gesture's type is one of the types given.
   *
   * @param types - The type(s) to check against.
   */
  is(...types: GestureType[]) {
    return types.includes(this.type)
  }

  /** Returns a new {@link Gesture} with the start, end, or type changed. */
  replace({ start, end, type }: { start?: Point; end?: Point; type?: GestureType }) {
    return new Gesture(start || this.start, end || this.end, type || this.type)
  }

  /**
   * Returns an offset position based on the current direction and distance.
   *
   * @param clamp - Allows for clamping the offset between an optional
   *   minimum and maximum.
   */
  offset(clamp?: { min?: number; max?: number }) {
    let offset = this.sign * this.dist
    if (!clamp) return offset
    if (clamp.min !== undefined) offset = Math.max(offset, clamp.min)
    if (clamp.max !== undefined) offset = Math.min(offset, clamp.max)
    return offset
  }
}

/**
 * Observer for gestures on an element. Fires a callback repeatedly,
 * providing a new {@link Gesture} each time. Use the `destroy` method to
 * end observation.
 */
export class GestureObserver {
  /**
   * The touch ID of the currently running gesture. If `null`, we aren't
   * running a gesture.
   */
  id: number | null = null

  /** The starting point of the gesture. */
  start: Point | null = null

  /**
   * @param target - The target element to observe.
   * @param callback - The callback to fire.
   */
  constructor(public target: HTMLElement, public callback: (gesture: Gesture) => void) {
    // bind handler so we can use it as a callback for events
    this.handler = this.handler.bind(this)

    // start listening
    target.addEventListener("touchstart", this.handler, { passive: true })
  }

  /** Attempts to find the currently running `Touch`. */
  private findTouch(evt: TouchEvent) {
    for (const touch of Array.from(evt.changedTouches)) {
      if (touch.identifier === this.id) return touch
    }
    return null
  }

  /** Cancels the running gesture. */
  private cancel() {
    document.removeEventListener("touchmove", this.handler)
    document.removeEventListener("touchend", this.handler)
    document.removeEventListener("touchcancel", this.handler)
    this.id = null
    this.start = null
  }

  /** Event handler function for touch events. */
  private handler(evt: TouchEvent) {
    let touch: Touch

    // new gesture
    if (evt.type === "touchstart") {
      document.addEventListener("touchmove", this.handler, { passive: true })
      document.addEventListener("touchend", this.handler, { passive: true })
      document.addEventListener("touchcancel", this.handler, { passive: true })
      touch = evt.changedTouches[0]
      this.id = touch.identifier
      this.start = [touch.clientX, touch.clientY]
    }
    // running gesture
    else if (this.id !== null) {
      const found = this.findTouch(evt)
      if (!found) return
      touch = found
    }
    // unrelated touch event
    else return

    // update gesture

    let type: GestureType
    // prettier-ignore
    switch (evt.type) {
      case 'touchstart':  type = 'start';  break
      case 'touchmove':   type = 'move';   break
      case 'touchend':    type = 'end';    break
      case 'touchcancel': type = 'cancel'; break
      default: throw new Error(`Unknown event type: ${evt.type}`)
    }

    const gesture = new Gesture(this.start!, [touch.clientX, touch.clientY], type)

    // check if we're performing a "scrolling" gesture
    // on an element that is scrollable
    if (type !== "start") {
      const target = evt.target as HTMLElement
      // if we found a scrollable element in the direction of our gesture, cancel
      if (scrollElement(target, gesture.axis)) this.cancel()
    }

    this.callback(gesture)

    if (gesture.is("end", "cancel")) this.cancel()
  }

  /** Ends observation. */
  destroy() {
    this.target.removeEventListener("touchstart", this.handler)
  }
}
