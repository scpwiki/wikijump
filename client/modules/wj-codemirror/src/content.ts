import { EditorState, Facet } from "../cm"

export type ContentFunction<T extends boolean = boolean> = (
  state: EditorState,
  buffer?: T
) => T extends true ? Promisable<ArrayBuffer> : Promisable<string>

/**
 * Facet that provides a function for extracting the textual content out of
 * a document, e.g. the non-markup ranges of HTML. The
 * `string`/`ArrayBuffer` returned should have the same length as the
 * original document, which should be done by making sure that all removed
 * not-content is replaced with whitespace.
 */
export const ContentFacet = Facet.define<ContentFunction, ContentFunction>({
  combine: arr => arr[arr.length - 1]
})
