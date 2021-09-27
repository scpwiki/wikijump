import { highlight } from "@wikijump/prism"
import { timeout } from "@wikijump/util"
import { defineElement, observe, pauseObservation } from "../../util"

/**
 * FTML `[[code]]` element. Automatically highlights the contents of its
 * `<code>` child with Prism.
 */
export class Code extends HTMLDivElement {
  static tag = "wj-code"

  /** Observer for watching changes to the contents of the code element. */
  declare observer: MutationObserver

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

    // observer for watching for changes to textual content
    this.observer = observe(this, () => this.update())
  }

  /**
   * Extracts the language to highlight with from this elements classes.
   * Specifically, the `wj-language-{name}` class.
   */
  private getLanguageFromClass() {
    const classes = Array.from(this.classList)
    for (const name of classes) {
      // this will always be ASCII lowercased,
      // so we can just use a simple check
      if (name.startsWith("wj-language-")) return name.substr(12)
    }
    return null
  }

  /** Ran whenever highlighting needs to be updated. */
  @pauseObservation
  private update() {
    // get the element every time we update,
    // because it might have been replaced by morphing or something
    const element = this.querySelector("code")
    if (!element) return

    const language = this.getLanguageFromClass() ?? "none"
    const content = element.innerText

    // don't waste resources if we're just doing the same thing
    if (!this.html || this.content !== content || this.language !== language) {
      this.language = language
      this.content = content
      this.html = highlight(content, language)
    }

    element.innerHTML = this.html
  }

  // -- LIFECYCLE

  connectedCallback() {
    if (!this.querySelector("pre")) {
      const defaultElement = document.createElement("pre")
      defaultElement.append(document.createElement("code"))
      this.appendChild(defaultElement)
    }

    this.update()
  }
}

/** Button that, when clicked, copies the contents of a `[[code]]` block. */
export class CodeCopyButton extends HTMLButtonElement {
  static tag = "wj-code-copy"

  constructor() {
    super()

    this.addEventListener("click", () => {
      const component = this.closest(`[is="${Code.tag}"]`)
      if (!component) return

      const code = component.querySelector("code")
      if (!code) return

      const text = code.innerText
      navigator.clipboard.writeText(text).then(() => {
        this.classList.add("wj-code-copy-success")
        timeout(1000, () => this.classList.remove("wj-code-copy-success"))
      })
    })
  }
}

defineElement(Code.tag, Code, { extends: "div" })
defineElement(CodeCopyButton.tag, CodeCopyButton, { extends: "button" })
