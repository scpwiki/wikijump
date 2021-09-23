import { highlight } from "@wikijump/prism"
import { defineElement } from "../util"

export class Code extends HTMLPreElement {
  static tag = "wj-code"

  declare language: string | null
  declare content: string

  constructor() {
    super()

    this.language = null
    this.content = ""

    // watch for changes to textual content
    const mutationObserver = new MutationObserver(() => this.update())
    mutationObserver.observe(this, { characterData: true, subtree: true })
  }

  private getLanguageFromClass() {
    const classes = Array.from(this.classList)
    for (const name of classes) {
      if (name.startsWith("wj-language-")) return name.substr(12)
    }
    return null
  }

  private update() {
    // get the element every time we update,
    // because it might have been replaced by morphing or something
    const element = this.querySelector("code")
    if (!element) return

    const language = this.getLanguageFromClass() ?? "none"
    const content = element.innerText

    // don't waste resources if we're just doing the same thing
    if (this.content === content && this.language === language) return

    this.language = language
    this.content = content

    element.innerHTML = highlight(content, language)
  }

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
