import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

import {
  checkRedirectDeviation,
  REDIRECT_DEVIATION_NOTE,
  REDIRECT_DEVIATION_SECTIONS,
  REDIRECT_SCANNER_SYMBOLS,
  redirectDeviationProblems
} from "../scripts/check-ftml-boundary-deviations.mjs"

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..")
const note = readFileSync(path.join(root, REDIRECT_DEVIATION_NOTE), "utf8")

test("Redirect syntax debt has the required FTML boundary deviation note", () => {
  assert.doesNotThrow(() => checkRedirectDeviation(root))
  assert.deepEqual(redirectDeviationProblems(note), [])
})

test("Redirect deviation validation rejects a missing note", () => {
  assert.deepEqual(redirectDeviationProblems(null), [`missing required note ${REDIRECT_DEVIATION_NOTE}`])
})

test("Redirect deviation validation rejects every missing scanner symbol", () => {
  for (const symbol of REDIRECT_SCANNER_SYMBOLS) {
    const problems = redirectDeviationProblems(note.replaceAll(symbol, "removed_scanner_symbol"))
    assert.ok(problems.includes(`missing scanner symbol ${symbol}`), symbol)
  }
})

test("Redirect deviation validation rejects every missing required section", () => {
  for (const section of REDIRECT_DEVIATION_SECTIONS) {
    const problems = redirectDeviationProblems(note.replace(`## ${section}`, `## Removed ${section}`))
    assert.ok(problems.includes(`missing required section ${section}`), section)
  }
})
