export const CONSTANTS = {
  /**
   * A record of deprecated names that map to their proper name. Used in
   * the {@link Aff} `.aff` parser.
   */
  SYNONYMS: {
    PSEUDOROOT: "NEEDAFFIX",
    COMPOUNDLAST: "COMPOUNDEND"
  } as Record<string, string>,

  /**
   * `RegExp` used to split a string of flags in "long" format, i.e. each
   * flag is two characters.
   */
  FLAG_LONG_REGEX: /../,

  /**
   * `RegExp` used to split a string of flags in "numeric" format, i.e.
   * each flag is a number, separated from other numbers with commas.
   */
  FLAG_NUM_REGEX: /\d+(?=,|$)/,

  /**
   * `RegExp` used to parse phoneme table rules.
   *
   * Groups:
   *
   * 1. Letters
   * 2. Optional
   * 3. Lookahead
   * 4. Flags
   * 5. Priority
   */
  PHONET_RULE_REGEX: /^(\p{L}+)(?:\((\p{L}+)\))?(-+)?([\^$<]*)(\d)?$/u,

  /**
   * `RegExp` used by the {@link Dic} `.dic` parser to determine if a line
   * should be skipped.
   */
  DIC_SKIP_REGEX: /^\d+(\s+|$)|^\/|^\t|^\s*$/,

  /**
   * `RegExp` used to split a `.dic` "word" into its various components. Groups:
   *
   * 1. Stem
   * 2. Flags
   * 3. Data (not split here, see {@link CONSTANTS.SPLIT_DATA_REGEX})
   */
  SPLIT_WORD_REGEX: /^(.+?)(?:\/([\S\t]*?))?(?:(?:\s(?=.*?:.))(.+))?$/,

  /**
   * `RegExp` used to split a `.dic` word data key-value. Groups:
   *
   * 1. Key
   * 2. Value
   */
  SPLIT_DATA_REGEX: /(\S+):(\S+)/,

  /** Maximum number of {@link PhonetTable} suggestions per list of suggestions. */
  MAX_PHONET_SUGGESTIONS: 2,

  /**
   * Maximum number of suggestions generated when yielding permutations of
   * a misspelling.
   */
  MAX_SUGGESTIONS: 15,

  /**
   * Types of "edits" to a misspelling that, if they result in a correct
   * word, mean that their resultant suggestion is almost certainly what
   * the misspelling was supposed to be and thus further suggestions
   * shouldn't be generated.
   */
  GOOD_EDITS: ["spaceword", "uppercase", "replchars"] as string[],

  /** Maximum number of ngram "roots" (most similar words to a misspelling). */
  NGRAM_MAX_ROOTS: 100,

  /** Maximum number of ngram guesses that can be processed. */
  NGRAM_MAX_GUESSES: 200,

  /**
   * Maximum number of {@link PhonetTable} "roots" (most similar words to a
   * misspelling).
   */
  PHONET_MAX_ROOTS: 100,

  /**
   * Maximum distance a character can be moved from its original position
   * when making permutations of a word.
   */
  MAX_CHAR_DISTANCE: 4,

  /** `RegExp` used to split a string `RegExp`. Used in the {@link re} function. */
  SPLIT_REGEX_REGEX: /^([^]*)\/([^]+)\/([^]*)$/,

  /** `RegExp` used to split an {@link Affix} condition. */
  SPLIT_CONDITION_REGEX: /(\[.+\]|[^\[])/g,

  /**
   * The default set of `RegExp`s used when breaking apart multiple words
   * from a single string with {@link breakWord}.
   */
  DEFAULT_BREAK: new Set([/-/g, /^-/g, /-$/g]),

  /** `RegExp` used to match a line that is just a number. Used in the `.dic` parser. */
  NUMBER_REGEX: /^\d+(\.\d+)?$/,

  /** `RegExp` used to split a line based on whitespace. */
  SPLIT_LINE_REGEX: /\s+/u
} as const

export const decoder = new TextDecoder()

/** The various capitalization types a word can have. */
export enum CapType {
  /** All lowercase. */
  NO,
  /** Titlecase. */
  INIT,
  /** All uppercase. */
  ALL,
  /** Mixed capitalization. */
  HUH,
  /** Mixed capitalization, first letter is capitalized. */
  HUHINIT
}

/** The various positions a word in a compound word could be in. */
export enum CompoundPos {
  /** The compound segment is at the beginning of the word. */
  BEGIN,
  /** The compound segment is somewhere in the middle of the word. */
  MIDDLE,
  /** The compound segment is at the end of the word. */
  END
}
