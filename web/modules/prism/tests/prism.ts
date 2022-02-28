import { describe, expect, it } from "vitest"
import * as lib from "../src/index"

// Vitest doesn't support web workers yet
describe.skip("@wikijump/prism", () => {
  const worker = lib.default

  it("needs manual to be true", async () => {
    expect(await worker.manual()).to.be.true
  })

  it("needs disableWorkerMessageHandler to be true", async () => {
    expect(await worker.disableWorkerMessageHandler()).to.be.true
  })

  it("should highlight", async () => {
    const html = await worker.highlight('console.log("foo")', "javascript")
    const snapshot = `<span class="wj-code-token wj-code-console wj-code-class-name">console</span><span class="wj-code-token wj-code-punctuation">.</span><span class="wj-code-token wj-code-method wj-code-function wj-code-property-access">log</span><span class="wj-code-token wj-code-punctuation">(</span><span class="wj-code-token wj-code-string">"foo"</span><span class="wj-code-token wj-code-punctuation">)</span>`
    expect(html).to.equal(snapshot)
  })

  it("should highlight with raw text", async () => {
    const html = await worker.highlight('console.log("foo")', "raw")
    expect(html).to.equal('console.log("foo")')
  })

  it("should highlight even with a bad language", async () => {
    const html = await worker.highlight(
      'console.log("foo")',
      "bad-language-that-doesn't-exist"
    )
    expect(html).to.equal('console.log("foo")')
  })
})
