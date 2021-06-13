import { EditorView, syntaxTree } from "wj-codemirror/cm"
import type { SpellcheckFilter, Word } from "./types"

const WORDS_REGEX = /\p{L}(?![\p{L}'’])|\p{L}[\p{L}'’]*\p{L}/gu

/**
 * Extracts the "visible words" of an editor. All words found will be
 * passed to the relevant language's spellchecking filter function, if it
 * exists. If it doesn't, the word will be excluded.
 *
 * @param view - The view to extract the words out of.
 * @param regex - The regex used to match the words of the document. Uses a
 *   default that should be adequate for scripts which separate words with
 *   whitespace.
 */
export function visibleWords(view: EditorView, regex = WORDS_REGEX) {
  const ranges = view.visibleRanges
  const total = { from: ranges[0].from, to: ranges[ranges.length - 1].to }

  let pos = total.from
  const words: Word[] = []
  const iter = view.state.doc.iterRange(total.from, total.to)
  const tree = syntaxTree(view.state)

  do {
    if (iter.done) break
    const matches = iter.value.matchAll(regex)
    for (const match of matches) {
      if (match.index === undefined) continue

      const wordPos = pos + match.index

      const filter = view.state.languageDataAt<SpellcheckFilter>("spellcheck", wordPos)[0]
      if (!filter) continue // spellcheck only if a filter is present

      const word = {
        from: wordPos,
        to: wordPos + match[0].length,
        word: match[0]
      }

      if (!filter(view.state, tree, word)) continue

      words.push(word)
    }
    pos += iter.value.length
  } while (iter.next())

  return words
}
