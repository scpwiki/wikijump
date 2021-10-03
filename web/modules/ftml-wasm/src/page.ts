import {
  DetailRenderedHTML,
  detailRenderHTML,
  makeInfo,
  PageInfo,
  PartialInfo,
  preprocess,
  RenderedHTML,
  renderHTML,
  renderText
} from "./interface"

/**
 * Helper class for wrapping up the `ftml-wasm` functions into a single,
 * lazy-rendering representation of a page.
 */
export class Page {
  /** The current source. */
  private declare source: string

  /** Cached render result. */
  private declare rendered?: RenderedHTML | DetailRenderedHTML

  /** The page's info, such as its title or tags. */
  declare info: PageInfo

  /**
   * @param source - The source to render.
   * @param info - The {@link PageInfo} to use. Any fields not specified
   *   will be mocked.
   * @param preprocessSource - Whether to preprocess the source before rendering.
   */
  constructor(source: string, info?: PartialInfo, preprocessSource = true) {
    this.source = preprocessSource ? preprocess(source) : source
    this.info = makeInfo(info)
  }

  /**
   * Ensures that {@link Page.rendered} is valid, and returns it.
   *
   * @param detailed - Whether to return a detailed render result.
   */
  private ensureRendered(detailed: false): RenderedHTML
  private ensureRendered(detailed: true): DetailRenderedHTML
  private ensureRendered(detailed: boolean): RenderedHTML | DetailRenderedHTML {
    if (!this.rendered || (detailed && !("tokens" in this.rendered))) {
      this.rendered = detailed
        ? detailRenderHTML(this.source, this.info)
        : renderHTML(this.source, this.info)
    }
    return this.rendered
  }

  /**
   * Updates the page's source to a new one.
   *
   * @param source - The new source.
   * @param preprocessSource - Whether to preprocess the source before rendering.
   */
  updateSource(source: string, preprocessSource = true) {
    this.source = preprocessSource ? preprocess(source) : source
    this.rendered = undefined
  }

  /**
   * Updates the page's info. The new info is merged with the page's
   * current info, so you can use this function to e.g. update a single field.
   *
   * @param info - The new info.
   */
  updateInfo(info: PartialInfo) {
    this.info = makeInfo({ ...this.info, ...info })
    this.rendered = undefined
  }

  get html() {
    return this.ensureRendered(false).html
  }

  get meta() {
    return this.ensureRendered(false).meta
  }

  get styles() {
    return this.ensureRendered(false).styles
  }

  get backlinks() {
    return this.ensureRendered(false).backlinks
  }

  get tokens() {
    return this.ensureRendered(true).tokens
  }

  get ast() {
    return this.ensureRendered(true).ast
  }

  get warnings() {
    return this.ensureRendered(true).warnings
  }

  get text() {
    return renderText(this.source, this.info)
  }
}
