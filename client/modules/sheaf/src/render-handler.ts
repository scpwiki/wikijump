import type { Text } from "@codemirror/state"
import * as FTML from "ftml-wasm-worker"
import { Memoize } from "typescript-memoize"
import { toFragment } from "wj-util"

/**
 * Heavily memoized FTML-render-emit handler. Designed to invoke the FTML
 * renderer as little as possible.
 */
export class RenderHandler {
  private declare fragmentNode?: DocumentFragment

  /** @param doc - The CodeMirror `state.doc` to render with. */
  constructor(public doc?: Text) {}

  /**
   * Raw source of the document.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  get src() {
    if (!this.doc) return ""
    return this.doc.toString()
  }

  /**
   * Renders the document to CSS and HTML.
   *
   * @param format - Whether or not to pretty-print/format the HTML.
   * @decorator `@Memoize()`
   */
  @Memoize()
  async result(format = false) {
    return await FTML.render(this.src, format)
  }

  /**
   * Renders the document to HTML.
   *
   * @param format - Whether or not to pretty-print/format the HTML.
   * @decorator `@Memoize()`
   */
  @Memoize()
  async html(format = false) {
    const { html } = await this.result(format)
    return html
  }

  /**
   * Renders the document's stylesheets.
   *
   * @decorator `@Memoize`
   */
  @Memoize()
  async styles() {
    const { styles } = await this.result()
    return styles
  }

  /**
   * Renders the document's combined stylesheet. This should only be
   * used for preview/display purposes, as it's likely the combined
   * stylesheet will contain invalid imports.
   *
   * @decorator `@Memoize`
   */
  @Memoize()
  async style() {
    const { styles } = await this.result()
    return styles
      .map((style, idx) => `/* stylesheet ${idx + 1} */\n\n${style}\n\n`)
      .join("\n")
  }

  /**
   * Renders the document to formatted text.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  async text() {
    const text = await FTML.renderText(this.src)
    return text
  }

  /**
   * Parses the document.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  async parse() {
    return await FTML.parse(this.src)
  }

  /**
   * Parses the document and returns its AST.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  async ast() {
    const { ast } = await this.parse()
    return ast
  }

  /**
   * Returns the warnings that would be emitted by parsing the document.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  async warnings() {
    const { warnings } = await this.parse()
    return warnings
  }

  /** Renders the document into a {@link DocumentFragment}. */
  async fragment() {
    if (!this.fragmentNode) {
      this.fragmentNode = toFragment(await this.html())
    }
    return this.fragmentNode.cloneNode(true) as DocumentFragment
  }

  /**
   * Retrieves the document's AST and then formats it into a pretty-printed string.
   *
   * @decorator `@Memoize()`
   */
  @Memoize()
  async stringifiedAST() {
    const ast = await this.ast()
    return JSON.stringify(ast, undefined, 2)
  }
}
