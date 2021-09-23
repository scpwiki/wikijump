import { highlight } from "@wikijump/prism"
import { defineElement } from "../util"

/**
 * FTML `[[code]]` element. Automatically highlights the contents of its
 * `<code>` child with Prism.
 */
export class Code extends HTMLPreElement {
  static tag = "wj-code"

  /** The language highlighting is being done with. */
  declare language: string | null

  /** The current textual contents of this element. */
  declare content: string

  /** The compiled/highlighted HTML. */
  declare html?: string

  constructor() {
    super()

    this.language = null
    this.content = ""

    // watch for changes to textual content
    const mutationObserver = new MutationObserver(() => this.update())
    mutationObserver.observe(this, { characterData: true, subtree: true })
  }

  /**
   * Extracts the language to highlight with from this elements classes.
   * Specifically, the `wj-language-{name}` class.
   */
  private getLanguageFromClass() {
    const classes = Array.from(this.classList)
    for (const name of classes) {
      if (name.startsWith("wj-language-")) return name.substr(12)
    }
    return null
  }

  /** Ran whenever highlighting needs to be updated. */
  private update() {
    // get the element every time we update,
    // because it might have been replaced by morphing or something
    const element = this.querySelector("code")
    if (!element) return

    const language = this.getLanguageFromClass() ?? "none"
    const content = element.innerText

    // don't waste resources if we're just doing the same thing
    if (this.html && this.content === content && this.language === language) {
      element.innerHTML = this.html
      return
    }

    this.language = language
    this.content = content
    this.html = highlight(content, language)

    element.innerHTML = this.html
  }

  // -- LIFECYCLE

  connectedCallback() {
    const element = this.querySelector("code")

    if (!element) {
      const defaultElement = document.createElement("code")
      this.appendChild(defaultElement)
    }

    this.update()
  }

  adoptedCallback() {
    this.update()
  }

  attributeChangedCallback() {
    this.update()
  }
}

defineElement(Code.tag, Code, { extends: "pre" })
