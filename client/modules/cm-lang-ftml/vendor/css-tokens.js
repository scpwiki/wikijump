// FULL CREDIT TO: https://github.com/lezer-parser/css/blob/master/src/tokens.js
// THIS IS SIMPLY A FORK OF THE CSS GRAMMAR SO IT WORKS IN STYLE STRINGS.

import {ExternalTokenizer} from "@lezer/lr"
import {callee, identifier, Unit} from "./css-attribute.terms.js"

const space = [9, 10, 11, 12, 13, 32, 133, 160, 5760, 8192, 8193, 8194, 8195, 8196, 8197,
               8198, 8199, 8200, 8201, 8202, 8232, 8233, 8239, 8287, 12288]
const colon = 58, parenL = 40, underscore = 95, bracketL = 91, dash = 45, period = 46,
      hash = 35, percent = 37

function isAlpha(ch) { return ch >= 65 && ch <= 90 || ch >= 97 && ch <= 122 || ch >= 161 }

function isDigit(ch) { return ch >= 48 && ch <= 57 }

export const identifiers = new ExternalTokenizer(input => {
  for (let inside = false, i = 0;; i++) {
    let {next} = input
    if (isAlpha(next) || next == dash || next == underscore || (inside && isDigit(next))) {
      if (!inside && (next != dash || i > 0)) inside = true
      input.advance()
    } else {
      if (inside)
        input.acceptToken(next == parenL ? callee : identifier)
      break
    }
  }
})

export const unitToken = new ExternalTokenizer(input => {
  if (!space.includes(input.peek(-1))) {
    let {next} = input
    if (next == percent) { input.advance(); input.acceptToken(Unit) }
    if (isAlpha(next)) {
      do { input.advance() } while (isAlpha(input.next))
      input.acceptToken(Unit)
    }
  }
})
