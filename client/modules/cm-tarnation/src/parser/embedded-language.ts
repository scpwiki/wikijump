import { Input, PartialParse, Tree } from "@lezer/common"
import { EditorParseContext, LanguageDescription } from "wj-codemirror/cm"
import type { TarnationLanguage } from "../language"

/**
 * An `EmbeddedLanguage` is effectively a very safe wrapper around a
 * CodeMirror/Lezer parser. This is because it does not accept a parser
 * directly - instead, you must give it a language to find (e.g.
 * "javascript"). It then does the rest itself - finding the language,
 * advancing the parser, etc.
 *
 * If the language given cannot be found, a safe fallback parser will be
 * returned instead, which ultimately results in an empty `Tree`. If an
 * `EditorParserContext` is available, a skipping parser will be used which
 * tells CodeMirror that the region of text parsed is still incomplete and
 * needs to be parsed again once the desired language is loaded. Otherwise,
 * a "fake" parser will be used that immediately returns an empty tree.
 */
export class EmbeddedLanguage {
  // fun fact: you can leave off the type in a class property declaration.
  // if you assign something to the property in the constructor, it will assume the type.
  // that's done here because these types are long and annoying.

  /** A promise that resolves when the language has finished loading. */
  private declare loading

  /** The loaded language's parser, if any. */
  private declare parser

  /** The fully loaded and resolved language, if available. */
  declare lang?: LanguageDescription | null

  /**
   * @param host - The host language.
   * @param language - The language to load.
   */
  constructor(host: TarnationLanguage, public language: string) {
    if (host.nestLanguages.length) {
      this.lang = LanguageDescription.matchLanguageName(host.nestLanguages, language)
      if (this.lang?.support) this.parser = this.bindParser()
      else this.loading = this.init(this.lang)
    }
  }

  /** Whether or not the language and parser is ready. */
  get ready() {
    return Boolean(this.parser)
  }

  /** Returns a resolved and bound parser constructor. */
  private bindParser() {
    if (!this.lang?.support) throw new Error("Could not bind unloaded language!")
    const parser = this.lang.support.language.parser
    return parser.startParse.bind(parser)
  }

  /**
   * Starts the loading process for the language.
   *
   * @param language - The language description to load. Can be null - if
   *   it is, a dummy parser will be returned instead.
   */
  private async init(language: LanguageDescription | null) {
    if (!language) this.parser = input => new FakeParse(input, Tree.empty)
    else {
      await language.load()
      return (this.parser = this.bindParser())
    }
  }

  /**
   * Gets a fallback skipping parser using the given context. If the
   * context is undefined, a dummy parser will be returned instead.
   */
  private getFallbackParser(context?: EditorParseContext) {
    return !context
      ? (input: Input) => new FakeParse(input, Tree.empty)
      : // eslint-disable-next-line @typescript-eslint/unbound-method
        EditorParseContext.getSkippingParser(this.loading).startParse
  }

  /**
   * Starts and returns the language's parser. If the language isn't
   * loaded, a fallback parser will be returned instead that fully
   * satisfies the interface.
   *
   * @param input - The document to parse.
   * @param start - The start position for the parser.
   * @param context - The CodeMirror editor context to give to the parser, if any.
   */
  startParse(input: Input, start: number, context?: EditorParseContext) {
    return (this.parser ?? this.getFallbackParser(context))(input, start, context ?? {})
  }
}

/**
 * Fake parser that implements CodeMirror's parse interface. Simply returns
 * a provided tree when advanced.
 */
export class FakeParse implements PartialParse {
  /**
   * @param input - The document to parse.
   * @param tree - The tree to return when the parser is advanced.
   */
  constructor(private input: Input, private tree: Tree) {}

  get pos() {
    return this.input.length
  }

  advance() {
    return this.tree
  }

  forceFinish() {
    return this.tree
  }
}
