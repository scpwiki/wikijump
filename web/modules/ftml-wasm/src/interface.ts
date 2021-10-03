import * as FTML from "../vendor/ftml"
import { freeTracked, ready, trk } from "./base"

// re-export some just to make them easier to import
export type Token = FTML.IToken
export type HTMLMeta = FTML.IHtmlMeta
export type Backlinks = FTML.IBacklinks
export type PageInfo = FTML.IPageInfo
export type PartialInfo = Partial<FTML.IPageInfo>
export type SyntaxTree = FTML.ISyntaxTree
export type Warning = FTML.IParseWarning

export interface RenderedHTML {
  html: string
  meta: HTMLMeta[]
  styles: string[]
  backlinks: Backlinks
}

export interface DetailRenderedHTML extends RenderedHTML {
  tokens: Token[]
  ast: SyntaxTree
  warnings: Warning[]
}

export interface DetailRenderedText {
  tokens: Token[]
  ast: SyntaxTree
  warnings: Warning[]
  text: string
}

export type UTF16IndexMapFunction = {
  (pos: number): number
  free: () => void
}

/** Creates a {@link PageInfo} object. Any properties not provided are mocked. */
export function makeInfo(partial?: PartialInfo): PageInfo {
  return {
    alt_title: null,
    category: null,
    language: "default",
    rating: 0,
    page: "unknown",
    site: "www",
    tags: [],
    title: "",
    ...partial
  }
}

/** Returns FTML's (the crate) version. */
export function version() {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  return FTML.version()
}

/**
 * Preprocesses a string of wikitext. See `ftml/src/preproc/test.rs` for
 * more information.
 *
 * @param str - The wikitext to preprocess.
 */
export function preprocess(str: string) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  return FTML.preprocess(str)
}

/**
 * Tokenizes a string of wikitext.
 *
 * @param str - The wikitext to tokenize.
 */
export function tokenize(str: string) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()

    freeTracked()

    return tokens
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Parses a string of wikitext. This returns an AST and warnings list, not HTML.
 *
 * @param str - The wikitext to parse.
 * @param info - The page info to use.
 */
export function parse(str: string, info?: PartialInfo) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const tokenized = trk(FTML.tokenize(str))
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const parsed = trk(FTML.parse(pageInfo, tokenized))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const warnings = parsed.warnings()

    freeTracked()

    return { ast, warnings }
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Returns the list of warnings emitted when parsing the provided string.
 *
 * @param str - The wikitext to get the warnings of.
 * @param info - The page info to use.
 */
export function warnings(str: string, info?: PartialInfo) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const parsed = trk(FTML.parse(pageInfo, tokenized))
    const warnings = parsed.warnings()

    freeTracked()

    return warnings
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Renders a string of wikitext to HTML.
 *
 * @param str - The wikitext to render.
 * @param info - The page info to use.
 */
export function renderHTML(str: string, info?: PartialInfo): RenderedHTML {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const parsed = trk(FTML.parse(trk(pageInfo.copy()), tokenized))
    const tree = trk(parsed.syntax_tree())
    const rendered = trk(FTML.render_html(pageInfo, tree))

    const html = rendered.body()
    const meta = rendered.html_meta()
    const styles = rendered.styles()
    const backlinks = rendered.backlinks()

    freeTracked()

    return { html, meta, styles, backlinks }
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Renders a string of wikitext like the {@link renderHTML} function, but
 * this function additionally returns the result from every step in the
 * rendering pipeline.
 *
 * @param str - The wikitext to render.
 * @param info - The page info to use.
 */
export function detailRenderHTML(str: string, info?: PartialInfo): DetailRenderedHTML {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()
    const parsed = trk(FTML.parse(trk(pageInfo.copy()), tokenized))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const warnings = parsed.warnings()
    const rendered = trk(FTML.render_html(pageInfo, tree))

    const html = rendered.body()
    const meta = rendered.html_meta()
    const styles = rendered.styles()
    const backlinks = rendered.backlinks()

    freeTracked()

    return { tokens, ast, warnings, html, meta, styles, backlinks }
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Renders a string of wikitext to plaintext.
 *
 * @param str - The wikitext to render.
 * @param info - The page info to use.
 */
export function renderText(str: string, info?: PartialInfo) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const parsed = trk(FTML.parse(trk(pageInfo.copy()), tokenized))
    const tree = trk(parsed.syntax_tree())
    const text = FTML.render_text(pageInfo, tree)

    freeTracked()

    return text
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Renders a string of wikitext like the {@link renderText} function, but
 * this function additionally returns the result from every step in the
 * rendering pipeline.
 *
 * @param str - The wikitext to render.
 * @param info - The page info to use.
 */
export function detailRenderText(str: string, info?: PartialInfo): DetailRenderedText {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()
    const parsed = trk(FTML.parse(trk(pageInfo.copy()), tokenized))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const warnings = parsed.warnings()
    const text = FTML.render_text(pageInfo, tree)

    freeTracked()

    return { tokens, ast, warnings, text }
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Returns a {@link UTF16IndexMapFunction} for the given string. This
 * function will return the UTF16 position for any given UTF8 position.
 * This lets you map between Rust's UTF8 indices and JavaScript's UTF16 indices.
 *
 * Due to a quirk in how Rust-to-WASM works, once you're done using the
 * index map, you should call the `free` method on it.
 *
 * @param str - The string to get the UTF16 index map for.
 */
export function getUTF16IndexMap(str: string): UTF16IndexMapFunction {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  const map = new FTML.Utf16IndexMap(str)
  const mapFunction = (pos: number) => map.get_index(pos)
  mapFunction.free = () => map.free()
  return mapFunction
}
