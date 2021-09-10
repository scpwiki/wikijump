import { assert } from "@esm-bundle/chai"
import * as lib from "../src/grammar/helpers"

// TODO: flesh out, this is mostly just placeholder

describe("helpers", () => {
  it("safe regex (re)", () => {
    const { re } = lib
    assert.instanceOf(re`/foo\w+/`, RegExp)
    assert.isNull(re`/[\]\w+/`) // bad regexp
  })
})
