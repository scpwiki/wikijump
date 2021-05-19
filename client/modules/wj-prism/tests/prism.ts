import * as uvu from "uvu"
import * as assert from "uvu/assert"
import * as lib from "../src/index"

const Prism = uvu.suite("wj-prism")

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
  const snapshot = `<span class="code-token code-console code-class-name">console</span><span class="code-token code-punctuation">.</span><span class="code-token code-method code-function code-property-access">log</span><span class="code-token code-punctuation">(</span><span class="code-token code-string">"foo"</span><span class="code-token code-punctuation">)</span>`
  assert.snapshot(html, snapshot)
})

Prism("highlight with raw text", () => {
  const html = lib.highlight('console.log("foo")', "raw")
  assert.snapshot(html, 'console.log("foo")')
})

Prism("highlight with invalid language", () => {
  const html = lib.highlight('console.log("foo")', "bad-language-that-doesn't-exist")
  assert.snapshot(html, 'console.log("foo")')
})

Prism.run()
