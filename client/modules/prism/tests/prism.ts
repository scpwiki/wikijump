import { expect } from "@esm-bundle/chai"
import * as lib from "../src/index"

describe("@wikijump/prism", () => {
  it("needs manual to be true", () => {
    // @ts-ignore
    expect(lib.Prism.manual).to.be.true
  })

  it("needs disableWorkerMessageHandler to be true", () => {
    // @ts-ignore
    expect(lib.Prism.disableWorkerMessageHandler).to.be.true
  })

  it("should highlight", async () => {
    await lib.languagesReady
    const html = lib.highlight('console.log("foo")', "javascript")
    const snapshot = `<span class="code-token code-console code-class-name">console</span><span class="code-token code-punctuation">.</span><span class="code-token code-method code-function code-property-access">log</span><span class="code-token code-punctuation">(</span><span class="code-token code-string">"foo"</span><span class="code-token code-punctuation">)</span>`
    expect(html).to.equal(snapshot)
  })

  it("should highlight with raw text", () => {
    const html = lib.highlight('console.log("foo")', "raw")
    expect(html).to.equal('console.log("foo")')
  })

  it("should highlight even with a bad language", () => {
    const html = lib.highlight('console.log("foo")', "bad-language-that-doesn't-exist")
    expect(html).to.equal('console.log("foo")')
  })
})
