import { addElement, BaseButton, observe, pauseObservation } from "@wikijump/dom"
import Prism from "@wikijump/prism"
import { animationFrame, timeout } from "@wikijump/util"

/**
 * FTML `[[code]]` element. Automatically highlights the contents of its
 * `<code>` child with Prism.
 */
export class CodeElement extends HTMLElement {
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
      if (name.startsWith("wj-language-")) return name.substring(12)
    }
    return null
  }

  /** Ran whenever highlighting needs to be updated. */
  @pauseObservation
  private async update() {
    // get the element every time we update,
    // because it might have been replaced by morphing or something
    const element = this.querySelector("code")
    if (!element) return

    const language = this.getLanguageFromClass()

    // jump out early if no language
    if (!language) {
      // replace old highlighting
      if (this.language) {
        this.language = null
        await animationFrame(() => {
          this.content = element.innerText
          this.html = this.content
          element.innerHTML = this.content
        })
      }
      return
    }

    await animationFrame(async () => {
      const content = element.innerText

      // don't waste resources if we're just doing the same thing
      if (!this.html || this.content !== content || this.language !== language) {
        this.language = language
        this.content = content
        this.html = await Prism.highlight(content, language!)
      }

      await animationFrame(() => (element.innerHTML = this.html!))
    })
  }

  // -- LIFECYCLE

  connectedCallback() {
    this.update()
  }
}

/** Button that, when clicked, copies the contents of a `[[code]]` block. */
export class CodeCopyElement extends BaseButton {
  static tag = "wj-code-copy"

  whenClicked() {
    const code = this.closest(".wj-code")?.querySelector("code")
    if (!code) return

    const text = code.innerText
    navigator.clipboard.writeText(text).then(() => {
      this.classList.add("wj-code-copy-success")
      timeout(1000, () => this.classList.remove("wj-code-copy-success"))
    })
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-code": CodeElement
    "wj-code-copy": CodeCopyElement
  }

  interface Window {
    CodeElement: typeof CodeElement
    CodeCopyElement: typeof CodeCopyElement
  }
}

addElement(CodeElement, "CodeElement")
addElement(CodeCopyElement, "CodeCopyElement")
