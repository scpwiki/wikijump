import type { RepPattern } from "./aff/rep-pattern"
import { replaceRange, uppercase } from "./util"

const MAX_CHAR_DISTANCE = 4

/**
 * Uses a {@link Aff}'s {@link RepPattern} table to yield replaced
 * permutations of a word.
 *
 * @param word - The word to yield the replaced forms of.
 * @param reps - The set of {@link RepPattern}s to use.
 */
export function* replchars(word: string, reps: Set<RepPattern>) {
  if (word.length < 2 || !reps.size) return
  for (const rep of reps) {
    for (const match of rep.match(word)) {
      const suggestion = replaceRange(
        word,
        match.index!,
        match.index! + rep.replacement.length,
        rep.replacement
      )
      yield suggestion
      if (suggestion.includes(" ")) {
        yield suggestion.split(" ", 2)
      }
    }
  }
}

/**
 * Uses an {@link Aff}'s `MAP` table to recursively replace characters in a
 * word. This yields every permutation of this action for every character.
 *
 * @param word - The word to yield the mapped forms of.
 * @param map - The map to use.
 */
export function* mapchars(word: string, map: Set<Set<string>>) {
  if (word.length < 2 || !map.size) return

  function* mapcharsInternal(word: string, start = 0): Generator<string> {
    if (start >= word.length) return
    for (const options of map) {
      for (const option of options) {
        const pos = word.indexOf(option, start)
        if (pos !== -1) {
          for (const other of options) {
            if (other === option) continue
            const replaced = replaceRange(word, pos, pos + option.length, other)
            yield replaced
            for (const variant of mapcharsInternal(replaced, pos + 1)) {
              yield variant
            }
          }
        }
      }
    }
  }

  yield* mapcharsInternal(word)
}

/**
 * Yields the permutations of a word with adjacent characters within it
 * swapped. For short words, specifically 4-5 letters, doubleswaps will be
 * yielded as well.
 */
export function* swapchar(word: string) {
  if (word.length < 2) return

  for (let i = 0; i < word.length - 1; i++) {
    yield word.slice(0, i) + word[i + 1] + word[i] + word.slice(i + 2)
  }

  if (word.length === 4 || word.length === 5) {
    yield word[1] +
      word[0] +
      (word.length === 5 ? word[2] : "") +
      word.slice(-1) +
      word.slice(-2)
    if (word.length === 5) {
      yield word[0] + word[2] + word[1] + word.slice(-1) + word.slice(-2)
    }
  }
}

/**
 * Yields the permutations of a word where non-adjacent characters are
 * swapped, with a maximum distance of four characters.
 */
export function* longswapchar(word: string) {
  for (let first = 0; first < word.length - 2; first++) {
    for (
      let second = first + 2;
      second < Math.min(first + MAX_CHAR_DISTANCE, word.length);
      second++
    ) {
      yield word.slice(0, first) +
        word[second] +
        word.slice(first + 1, second) +
        word[first] +
        word.slice(second + 1)
    }
  }
}

/**
 * Yields the permutations of a word where characters are swapped with
 * characters adjacent to it, but as found on a keyboard layout, not in the
 * word itself. Additionally, variations on single character capitalization
 * will be yielded.
 *
 * @param word - The word to yield the permutations of.
 * @param layout - The layout string, with rows of the keyboard separated
 *   by `|` pipe characters.
 */
export function* badcharkey(word: string, layout: string) {
  for (let i = 0; i < word.length; i++) {
    const ch = word[i]
    const before = word.slice(0, i)
    const after = word.slice(i + 1)

    if (ch !== uppercase(ch)) {
      yield before + uppercase(ch) + after
    }

    if (!layout) continue

    let pos = layout.indexOf(ch)
    while (pos !== -1) {
      if (pos > 0 && layout[pos - 1] !== "|") {
        yield before + layout[pos - 1] + after
      }
      if (pos + 1 < layout.length && layout[pos + 1] !== "|") {
        yield before + layout[pos - 1] + after
      }
      pos = layout.indexOf(ch, pos + 1)
    }
  }
}

/**
 * Yields the permutations of a word where a character has been removed
 * from every position.
 */
export function* extrachar(word: string) {
  if (word.length < 2) return
  for (let i = 0; i < word.length; i++) {
    yield word.slice(0, i) + word.slice(i + 1)
  }
}

/**
 * Yields the permutations of a word with a character inserted in every
 * position, using a `TRY` string for determining which characters should
 * be inserted.
 *
 * @param word - The word to yield of the permutations of.
 * @param trystring - A string of characters which will be iterated through
 *   when inserting new characters.
 */
export function* forgotchar(word: string, trystring: string) {
  if (!trystring) return
  for (const ch of trystring) {
    for (let i = 0; i < word.length + 1; i++) {
      yield word.slice(0, i) + ch + word.slice(i)
    }
  }
}

/**
 * Yields the permutations of a word where a character has been moved 2, 3,
 * or 4 positions backwards or forwards.
 */
export function* movechar(word: string) {
  if (word.length < 2) return
  for (let frompos = 0; frompos < word.length; frompos++) {
    const ch = word[frompos]
    for (
      let topos = frompos + 3;
      topos < Math.min(word.length, frompos + MAX_CHAR_DISTANCE + 1);
      topos++
    ) {
      yield word.slice(0, frompos) +
        word.slice(frompos + 1, topos) +
        ch +
        word.slice(topos)
    }
  }

  for (let frompos = word.length; frompos > 0; frompos--) {
    for (
      let topos = frompos - 1;
      topos > Math.max(0, frompos - MAX_CHAR_DISTANCE + 1);
      topos--
    ) {
      yield word.slice(0, topos) +
        word[frompos] +
        word.slice(topos, frompos) +
        word.slice(frompos + 1)
    }
  }
}

/**
 * Yields the permutations of a word where a character has been replaced by
 * one of the characters found in the `TRY` string.
 *
 * @param word - The word to yield the permutations of.
 * @param trystring - A string of characters which will be iterated through
 *   when replacing characters.
 */
export function* badchar(word: string, trystring: string) {
  if (!trystring) return
  for (const ch of trystring) {
    for (let i = word.length; i > 0; i--) {
      if (word[i] === ch) continue
      yield word.slice(0, i) + ch + word.slice(i + 1)
    }
  }
}

/**
 * Yields the permutations of a word where any repeated slice of two
 * characters has been removed. e.g. `vacacation` to `vacation`.
 */
export function* doubletwochars(word: string) {
  if (word.length < 5) return
  for (let i = 2; i < word.length; i++) {
    if (word[i - 2] === word[i] && word[i - 3] === word[i - 1]) {
      yield word.slice(0, i - 1) + word.slice(i + 1)
    }
  }
}

/**
 * Yields the permutations of a word where it has been split with a space
 * in every position.
 */
export function* twowords(word: string) {
  for (let i = 1; i < word.length; i++) {
    yield [word.slice(0, i), word.slice(i)]
  }
}
