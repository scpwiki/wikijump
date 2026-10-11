import { strict as assert } from "node:assert"
import test from "node:test"

import {
  addLocalePreference,
  moveLocalePreference,
  removeLocalePreference
} from "../src/lib/user-locale-order.js"

test("display language ordering preserves primary and fallback positions", () => {
  assert.deepEqual(moveLocalePreference(["ja", "en", "ko"], "ko", -1), ["ja", "ko", "en"])
  assert.deepEqual(moveLocalePreference(["ja", "ko", "en"], "en", -1), ["ja", "en", "ko"])
  assert.deepEqual(moveLocalePreference(["ja", "en"], "ja", -1), ["ja", "en"])
  assert.deepEqual(moveLocalePreference(["ja", "en"], "en", 1), ["ja", "en"])
})

test("adding and removing display languages never creates duplicate preferences", () => {
  assert.deepEqual(addLocalePreference(["ja", "en"], "ko"), ["ja", "en", "ko"])
  assert.deepEqual(addLocalePreference(["ja", "en"], "ja"), ["ja", "en"])
  assert.deepEqual(removeLocalePreference(["ja", "en", "ja"], "ja"), ["en"])
})
