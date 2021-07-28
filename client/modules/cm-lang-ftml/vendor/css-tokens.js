/* eslint-disable eqeqeq */
// FULL CREDIT TO: https://github.com/lezer-parser/css/blob/master/src/tokens.js
// THIS IS SIMPLY A FORK OF THE CSS GRAMMAR SO IT WORKS IN STYLE STRINGS.

import { ExternalTokenizer } from 'lezer'
import { callee, identifier, Unit } from './css-attribute.terms.js'

const space = [9, 10, 11, 12, 13, 32, 133, 160, 5760, 8192, 8193, 8194, 8195, 8196, 8197,
               8198, 8199, 8200, 8201, 8202, 8232, 8233, 8239, 8287, 12288]
const parenL = 40, underscore = 95, dash = 45, percent = 37

function isAlpha(ch) { return ch >= 65 && ch <= 90 || ch >= 97 && ch <= 122 || ch >= 161 }

function isDigit(ch) { return ch >= 48 && ch <= 57 }

export const identifiers = new ExternalTokenizer((input, token) => {
  let start = token.start, pos = start, inside = false
  for (;;) {
    let next = input.get(pos)
    if (isAlpha(next) || next == dash || next == underscore || (inside && isDigit(next))) {
      if (!inside && (next != dash || pos > start)) inside = true
      pos++
      continue
    }
    if (inside)
      token.accept(next == parenL ? callee : identifier, pos)
    break
  }
})

export const unitToken = new ExternalTokenizer((input, token) => {
  let { start } = token
  if (!space.includes(input.get(start - 1))) {
    let next = input.get(start)
    if (next == percent) token.accept(Unit, start + 1)
    if (isAlpha(next)) {
      let pos = start + 1
      while (isAlpha(input.get(pos))) pos++
      token.accept(Unit, pos)
    }
  }
})
