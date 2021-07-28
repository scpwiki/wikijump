import { animationFrame, idleCallback } from "wj-util"
import type { EditorState, Text } from "../cm"

const encoder = new TextEncoder()

/** Asynchronously compiles a `Text` object into a single string. */
export async function textValue(doc: Text) {
  let out = ""
  let last = 0
  for (const str of doc) {
    out += str
    // throttle on 32k chunks
    if (out.length - last > 32768) {
      last = out.length
      await animationFrame()
    }
  }

  return out
}

/** Asynchronously compiles a `Text` object into a `Uint8Array` buffer. */
export async function textBuffer(doc: Text) {
  let len = 0
  let last = 0
  const buffers: Uint8Array[] = []
  for (const str of doc) {
    const buffer = encoder.encode(str)
    buffers.push(buffer)
    len += buffer.length
    // throttle on 32k chunks
    if (len - last > 32768) {
      last = len
      await animationFrame()
    }
  }

  let pos = 0
  const out = new Uint8Array(len)
  await idleCallback(() => {
    for (const buffer of buffers) {
      out.set(buffer, pos)
      pos += buffer.length
    }
  })

  return out
}

/**
 * Gets the "active lines" of a state. This includes any lines the user has
 * a cursor on and all lines touching their selection box, if any.
 */
export function getActiveLines(state: EditorState) {
  const activeLines: Set<number> = new Set()
  for (const range of state.selection.ranges) {
    const lnStart = state.doc.lineAt(range.from).number
    const lnEnd = state.doc.lineAt(range.to).number
    if (lnStart === lnEnd) activeLines.add(lnStart - 1)
    else {
      const diff = lnEnd - lnStart
      for (let lineNo = 0; lineNo <= diff; lineNo++) {
        activeLines.add(lnStart + lineNo - 1)
      }
    }
  }
  return activeLines
}

// this is apparently how CodeMirror does underlines,
// I figured it was just a text underline, but no, it's actually this
// kind of interesting
/** Returns a `background-image` inlined SVG string for decorations. */
export function underline(color: string) {
  if (typeof btoa !== "function") return "none"
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="6" height="3">
    <path d="m0 3 l2 -2 l1 0 l2 2 l1 0" stroke="${color}" fill="none" stroke-width=".7"/>
  </svg>`
  return `url('data:image/svg+xml;base64,${btoa(svg)}')`
}
