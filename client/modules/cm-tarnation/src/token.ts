import type { GrammarToken, ParserAction } from "./types"

/**
 * Creates an `ArrayBuffer` stored token. This is effectively a highly
 * efficient representation of a {@link GrammarToken}.
 *
 * Returns an ArrayBuffer that is, in order:
 *
 * - The token ID, which is a uint8
 * - The token from position, which is a uint32
 * - The token length, which is a uint16
 * - A repeating list of parser actions
 *
 * Parser actions are:
 *
 * - Action type, which is a uint8 (0 is OPEN, 1 is CLOSE)
 * - Node id, which is a uint8
 */
export function create(
  id: number | null,
  from: number,
  to: number,
  open?: ParserAction,
  close?: ParserAction
) {
  let len = 7
  if (open) len += open.length * 2
  if (close) len += close.length * 2

  const arr = new ArrayBuffer(len)
  const view = new DataView(arr)
  view.setUint8(0, id ?? 0)
  view.setUint32(1, from)
  view.setUint16(5, to - from)

  let offset = 7
  if (open) {
    for (let i = 0; i < open.length; i++) {
      view.setUint8(offset, 0)
      view.setUint8(offset + 1, open[i])
      offset += 2
    }
  }
  if (close) {
    for (let i = 0; i < close.length; i++) {
      view.setUint8(offset, 1)
      view.setUint8(offset + 1, close[i])
      offset += 2
    }
  }

  return arr
}

/**
 * Reads an `ArrayBuffer` token.
 *
 * @param token - The token to read.
 * @param offset - The offset to add to the token's position.
 */
export function read(token: ArrayBuffer, offset = 0): GrammarToken {
  const view = new DataView(token)
  const { open, close } = actions(token)
  const id = view.getUint8(0) || null
  const from = view.getUint32(1) + offset
  const to = from + view.getUint16(5)
  return [id, from, to, open, close]
}

/**
 * Returns true if the given token has any parser actions.
 *
 * @param token - The token to check.
 */
export function hasActions(token: ArrayBuffer) {
  return token.byteLength > 7
}

/**
 * Reads the actions from an `ArrayBuffer` token.
 *
 * @param token - The token to read.
 */
export function actions(token: ArrayBuffer) {
  if (token.byteLength <= 7) return { open: undefined, close: undefined }

  const view = new DataView(token)
  const length = token.byteLength - 7
  const open: ParserAction = []
  const close: ParserAction = []

  let i = 0
  while (i < length) {
    const type = view.getUint8(7 + i)
    const id = view.getUint8(8 + i)
    if (type === 0) open.push(id)
    if (type === 1) close.push(id)
    i += 2
  }

  return {
    open: open.length ? open : undefined,
    close: close.length ? close : undefined
  }
}
