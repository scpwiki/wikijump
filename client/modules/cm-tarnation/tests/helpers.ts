import * as uvu from "uvu"
import * as assert from "uvu/assert"
import * as lib from "../src/grammar/helpers"

// TODO: flesh out, this is mostly just placeholder

const Helpers = uvu.suite("Helpers")

Helpers("safe regex (re)", () => {
  const { re } = lib
  assert.instance(re`/foo\w+/`, RegExp)
  assert.is(re`/[\]\w+/`, null) // bad regexp
})

Helpers.run()
