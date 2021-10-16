import { timeout } from "@wikijump/util"
import { writable } from "svelte/store"
import Toasts from "../Toasts.svelte"

/** Represents a currently active toast. */
interface Toast {
  /** The type of toast. */
  type: "success" | "danger" | "warning" | "info"
  /** The message being displayed. */
  message: string
  /** A function, that when called, will remove the toast. */
  remove: () => void
}

/** A stored immutable `Set` containing the currently visible toasts. */
export const toasts = writable<Set<Toast>>(new Set())

/**
 * Displays a 'toast' notification to the user.
 *
 * @param type - The type of notification to display.
 * @param message - The message to display.
 * @param time - The time in milliseconds to display the notification. Pass
 *   `0` to prevent the notification from closing.
 */
export function toast(
  type: "success" | "danger" | "warning" | "info",
  message: string,
  time = 5000
) {
  const remove = () => {
    toasts.update(cur => {
      cur.delete(toastData)
      return new Set(cur)
    })
  }
  const toastData = { type, message, remove }
  toasts.update(cur => new Set(cur.add(toastData)))
  if (time) timeout(time, remove)
}

// load the toasts handler component into the DOM
window.addEventListener("DOMContentLoaded", async () => {
  const container = document.querySelector("#toasts")
  if (!container) return
  new Toasts({ target: container })
})
