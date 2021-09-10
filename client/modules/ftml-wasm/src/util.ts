import * as FTML from "../vendor/ftml"
import { freeTracked, ready, trk } from "./base"

/** Converts a string of wikitext into a pretty-printed list of tokens. */
export function inspectTokens(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    str = preprocess ? FTML.preprocess(str) : str

    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()

    freeTracked()

    let out = ""
    for (const {
      slice,
      span: { start, end },
      token
    } of tokens) {
      const tokenStr = String(token.padEnd(16))
      const startStr = String(start).padStart(4, "0")
      const endStr = String(end).padStart(4, "0")
      const sliceStr = slice.slice(0, 40).replaceAll("\n", "\\n")
      out += `[${startStr} <-> ${endStr}]: ${tokenStr} => '${sliceStr}'\n`
    }

    return out
  } catch (err) {
    freeTracked()
    throw err
  }
}
