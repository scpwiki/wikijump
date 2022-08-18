import * as FTML from "../vendor/ftml"
import { freeTracked, ready, trk } from "./base"

// re-export some just to make them easier to import
export type Token = FTML.IToken
export type HTMLMeta = FTML.IHtmlMeta
export type Backlinks = FTML.IBacklinks
export type PageInfo = FTML.IPageInfo
export type PartialInfo = Partial<FTML.IPageInfo>
export type SyntaxTree = FTML.ISyntaxTree
export type FtmlError = FTML.IParseError
export type WikitextMode = FTML.WikitextMode
export type WikitextSettings = FTML.IWikitextSettings

export type ParseResult = { ast: SyntaxTree; errors: FtmlError[] }

export type RenderSettings = WikitextMode | WikitextSettings

export interface RenderedHTML {
  html: string
  meta: HTMLMeta[]
  backlinks: Backlinks
}

export interface DetailRenderedHTML extends RenderedHTML {
  tokens: Token[]
  ast: SyntaxTree
  errors: FtmlError[]
}

export interface DetailRenderedText {
  tokens: Token[]
  ast: SyntaxTree
  errors: FtmlError[]
  text: string
}

export type UTF16IndexMapFunction = {
  (pos: number): number
  free: () => void
}

function makeSettings(settings: RenderSettings): FTML.WikitextSettings {
  if (typeof settings === "string") {
    return FTML.WikitextSettings.from_mode(settings)
  } else {
    return new FTML.WikitextSettings(settings)
  }
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
 * Parses a string of wikitext. This returns an AST and errors list, not HTML.
 *
 * @param str - The wikitext to parse.
 * @param info - The page info to use.
 * @param mode - The wikitext rendering mode to use.
 */
export function parse(str: string, info?: PartialInfo, mode: RenderSettings = "page") {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const tokenized = trk(FTML.tokenize(str))
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, pageInfo, settings))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const errors = parsed.errors()

    freeTracked()

    return { ast, errors } as ParseResult
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Returns the list of errors emitted when parsing the provided string.
 *
 * @param str - The wikitext to get the errors of.
 * @param info - The page info to use.
 * @param mode - The wikitext rendering mode to use.
 */
export function errors(str: string, info?: PartialInfo, mode: RenderSettings = "page") {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, pageInfo, settings))
    const errors = parsed.errors()

    freeTracked()

    return errors
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
 * @param mode - The wikitext rendering mode to use.
 */
export function renderHTML(
  str: string,
  info?: PartialInfo,
  mode: RenderSettings = "page"
): RenderedHTML {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, trk(pageInfo.copy()), trk(settings.copy())))
    const tree = trk(parsed.syntax_tree())
    const rendered = trk(FTML.render_html(tree, pageInfo, settings))

    const html = rendered.body()
    const meta = rendered.html_meta()
    const backlinks = rendered.backlinks()

    freeTracked()

    return { html, meta, backlinks }
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
 * @param mode - The wikitext rendering mode to use.
 */

export function detailRenderHTML(
  str: string,
  info?: PartialInfo,
  mode: RenderSettings = "page"
): DetailRenderedHTML {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, trk(pageInfo.copy()), trk(settings.copy())))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const errors = parsed.errors()
    const rendered = trk(FTML.render_html(tree, pageInfo, settings))

    const html = rendered.body()
    const meta = rendered.html_meta()
    const backlinks = rendered.backlinks()

    freeTracked()

    return { tokens, ast, errors, html, meta, backlinks }
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
 * @param mode - The wikitext rendering mode to use.
 */
export function renderText(
  str: string,
  info?: PartialInfo,
  mode: RenderSettings = "page"
) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, trk(pageInfo.copy()), trk(settings.copy())))
    const tree = trk(parsed.syntax_tree())
    const text = FTML.render_text(tree, pageInfo, settings)

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
 * @param mode - The wikitext rendering mode to use.
 */
export function detailRenderText(
  str: string,
  info?: PartialInfo,
  mode: RenderSettings = "page"
): DetailRenderedText {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const pageInfo = trk(new FTML.PageInfo(makeInfo(info)))
    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()
    const settings = trk(makeSettings(mode))
    const parsed = trk(FTML.parse(tokenized, trk(pageInfo.copy()), trk(settings.copy())))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const errors = parsed.errors()
    const text = FTML.render_text(tree, pageInfo, settings)

    freeTracked()

    return { tokens, ast, errors, text }
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
