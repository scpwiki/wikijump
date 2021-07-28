/* Exports an easy to use wrapper around the FTML (WASM) library. */

import initFTML, * as Binding from "../vendor/ftml"

/** Indicates if the WASM binding is loaded. */
export let ready = false

let resolveLoading: (value?: unknown) => void
/** Promise that resolves when the WASM binding has loaded. */
export const loading = new Promise(resolve => {
  resolveLoading = resolve
})

/** Actual output of the WASM instantiation. */
export let wasm: Binding.InitOutput | null = null

/** Loads the WASM required for the FTML library. */
export async function init(path?: Binding.InitInput) {
  wasm = await initFTML(path)
  ready = true
  resolveLoading()
}

/** Safely frees any WASM objects provided. */
function free(...objs: any) {
  for (const obj of objs) {
    if (typeof obj !== "object" || !("ptr" in obj)) continue
    if (obj.ptr !== 0) obj.free()
  }
}

/**
 * This set contains unfreed WASM objects. It is separate from any
 * particular function so that error recovery can still clear memory.
 */
const tracked = new Set<any>()

/** Adds a WASM object to the list of tracked objects. */
function trk<T>(obj: T): T {
  tracked.add(obj)
  return obj
}

/** Frees all objects being {@link tracked}, and clears the set. */
function freeTracked() {
  free(...tracked)
  tracked.clear()
}

export type PageInfo = Partial<Binding.IPageInfo>

/**
 * Creates a {@link Binding.PageInfo | PageInfo} object. Any properties not
 * provided are mocked.
 */
function makeInfo({
  alt_title = null,
  category = null,
  language = "default",
  rating = 0,
  page = "unknown",
  site = "www",
  tags = [],
  title = ""
}: PageInfo = {}) {
  return new Binding.PageInfo({
    alt_title,
    category,
    language,
    rating,
    page,
    site,
    tags,
    title
  })
}

/** Returns FTML's (the crate) version. */
export function version() {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  return Binding.version()
}

/**
 * Preprocesses a string of wikitext. See `ftml/src/preproc/test.rs` for
 * more information.
 */
export function preprocess(str: string) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  return Binding.preprocess(str)
}

/** Tokenizes a string of wikitext. */
export function tokenize(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    str = preprocess ? Binding.preprocess(str) : str

    const tokenized = trk(Binding.tokenize(str))
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
 * @see {@link render}
 */
export function parse(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    str = preprocess ? Binding.preprocess(str) : str

    const tokenized = trk(Binding.tokenize(str))
    const parsed = trk(Binding.parse(tokenized))
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

export interface RenderOptions {
  /** Return HTML data or just text? */
  mode?: "html" | "text"
  /**
   * Contextual information about the wikitext being rendered. Unspecified
   * properties will be mocked.
   */
  info?: PageInfo
  /** Preprocess input before rendering? */
  preprocess?: boolean
}

type RenderHTML = { html: string; meta: Binding.IHtmlMeta[]; styles: string[] }

/** Renders a string of wikitext. */
export function render(str: string, opts?: { mode?: "html" } & RenderOptions): RenderHTML
export function render(str: string, opts?: { mode: "text" } & RenderOptions): string
export function render(str: string, opts?: RenderOptions) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  const { mode = "html", info, preprocess = true } = opts ?? {}
  try {
    str = preprocess ? Binding.preprocess(str) : str

    const tokenized = trk(Binding.tokenize(str))
    const parsed = trk(Binding.parse(tokenized))
    const tree = trk(parsed.syntax_tree())
    const pageInfo = trk(makeInfo(info))

    const rendered =
      mode === "html"
        ? trk(Binding.render_html(pageInfo, tree))
        : trk(Binding.render_text(pageInfo, tree))

    if (typeof rendered === "object") {
      const html = rendered.body()
      const meta = rendered.html_meta()
      const styles = rendered.styles()
      freeTracked()
      return { html, meta, styles }
    }

    freeTracked()
    return rendered
  } catch (err) {
    freeTracked()
    throw err
  }
}

export interface DetailedRenderOptions {
  mode?: "html" | "text"
  info?: PageInfo
}

export interface DetailedRenderHTML {
  preprocessed: string
  tokens: Binding.IToken[]
  ast: Binding.ISyntaxTree
  warnings: Binding.IParseWarning[]
  html: string
  meta: Binding.IHtmlMeta[]
  styles: string[]
}

export interface DetailedRenderText {
  preprocessed: string
  tokens: Binding.IToken[]
  ast: Binding.ISyntaxTree
  warnings: Binding.IParseWarning[]
  text: string
}

// yeah the overloads are awful, sorry lol
// means that setting the mode returns the correct object
/**
 * Renders a string of wikitext like the {@link render} function, but this
 * function additionally returns every step in the rendering pipeline.
 */
export function detailedRender(
  str: string,
  opts?: { mode?: "html" } & DetailedRenderOptions
): DetailedRenderHTML
export function detailedRender(
  str: string,
  opts?: { mode: "text" } & DetailedRenderOptions
): DetailedRenderText
export function detailedRender(
  str: string,
  opts?: DetailedRenderOptions
): DetailedRenderHTML | DetailedRenderText {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  const { mode = "html", info } = opts ?? {}
  try {
    const preprocessed = Binding.preprocess(str)
    const tokenized = trk(Binding.tokenize(preprocessed))
    const tokens = tokenized.tokens()
    const parsed = trk(Binding.parse(tokenized))
    const tree = trk(parsed.syntax_tree())
    const ast = tree.data()
    const warnings = parsed.warnings()
    const pageInfo = trk(makeInfo(info))

    const rendered =
      mode === "html"
        ? trk(Binding.render_html(pageInfo, tree))
        : trk(Binding.render_text(pageInfo, tree))

    if (typeof rendered === "object") {
      const html = rendered.body()
      const meta = rendered.html_meta()
      const styles = rendered.styles()
      freeTracked()
      return { preprocessed, tokens, ast, warnings, html, meta, styles }
    }

    freeTracked()
    return { preprocessed, tokens, ast, warnings, text: rendered }
  } catch (err) {
    freeTracked()
    throw err
  }
}

/** Returns the list of warnings emitted when parsing the provided string. */
export function warnings(str: string) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    const tokenized = trk(Binding.tokenize(str))
    const parsed = trk(Binding.parse(tokenized))

    const warnings = parsed.warnings()

    freeTracked()

    return warnings
  } catch (err) {
    freeTracked()
    throw err
  }
}

/** Converts a string of wikitext into a pretty-printed list of tokens. */
export function inspectTokens(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    str = preprocess ? Binding.preprocess(str) : str

    const tokenized = trk(Binding.tokenize(str))
    const tokens = tokenized.tokens()

    freeTracked()

    let out = ""
    for (const {
      slice,
      span: { start, end },
      token
    } of tokens) {
      const tokenStr = String(token.padEnd(16))
      const startStr = String(start).padStart(4, "0")
      const endStr = String(end).padStart(4, "0")
      const sliceStr = slice.slice(0, 40).replaceAll("\n", "\\n")
      out += `[${startStr} <-> ${endStr}]: ${tokenStr} => '${sliceStr}'\n`
    }

    return out
  } catch (err) {
    freeTracked()
    throw err
  }
}
