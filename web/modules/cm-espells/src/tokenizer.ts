import { EditorView, syntaxTree } from "@wikijump/codemirror/cm"
import type { Locale, SpellcheckFilter, Word } from "./types"

/**
 * Extracts the "visible words" of an editor. All words found will be
 * passed to the relevant language's spellchecking filter function, if it
 * exists. If it doesn't, the word will be excluded.
 *
 * @param view - The view to extract the words out of.
 * @param locale - The {@link Locale} to use.
 */
export function visibleWords(view: EditorView, locale: Locale) {
  const ranges = view.visibleRanges
  const total = { from: ranges[0].from, to: ranges[ranges.length - 1].to }

  let pos = total.from
  const words: Word[] = []
  const iter = view.state.doc.iterRange(total.from, total.to)
  const tree = syntaxTree(view.state)

  do {
    if (iter.done) break
    const matches = iter.value.matchAll(locale.pattern)
    match_loop: for (const match of matches) {
      if (match.index === undefined) continue

      // check locale filters

      for (let idx = 0; idx < locale.filters.length; idx++) {
        const filter = locale.filters[idx]
        if (typeof filter === "function" ? filter(match[0]) : filter.test(match[0])) {
          continue match_loop
        }
      }

      // check language filter

      const wordPos = pos + match.index

      const filter = view.state.languageDataAt<SpellcheckFilter>("spellcheck", wordPos)[0]
      if (!filter) continue // spellcheck only if a filter is present

      const word = {
        from: wordPos,
        to: wordPos + match[0].length,
        word: match[0]
      }

      if (!filter(view.state, tree, word)) continue

      // word passed all filters

      words.push(word)
    }
    pos += iter.value.length
  } while (iter.next())

  return words
}
