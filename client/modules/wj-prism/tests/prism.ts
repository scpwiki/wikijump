import * as uvu from "uvu"
import * as assert from "uvu/assert"

import * as lib from "../src/index"

const Prism = uvu.suite("PrismWrapper")

Prism("manual is true", () => {
  // @ts-ignore
  assert.is(lib.Prism.manual, true)
})

Prism("disable worker messsage is true", () => {
  // @ts-ignore
  assert.is(lib.Prism.disableWorkerMessageHandler, true)
})

Prism("highlight", () => {
  const html = lib.highlight('console.log("foo")', "javascript")
  const snapshot = `<span class="token console class-name">console</span><span class="token punctuation">.</span><span class="token method function property-access">log</span><span class="token punctuation">(</span><span class="token string">"foo"</span><span class="token punctuation">)</span>`
  assert.snapshot(html, snapshot)
})

Prism.run()
