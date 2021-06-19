export const CONSTANTS = {
  SYNONYMS: {
    PSEUDOROOT: "NEEDAFFIX",
    COMPOUNDLAST: "COMPOUNDEND"
  } as Record<string, string>,

  FLAG_LONG_REGEX: /../,
  FLAG_NUM_REGEX: /\d+(?=,|$)/,

  // 1. letters, 2. optional, 3. lookahead, 4. flags, 5. priority
  PHONET_RULE_REGEX: /^(\p{L}+)(?:\((\p{L}+)\))?(-+)?([\^$<]*)(\d)?$/u,

  DIC_SKIP_REGEX: /^\d+(\s+|$)|^\/|^\t|^\s*$/,

  // 1. stem, 2. flags, 3. data (not split)
  SPLIT_WORD_REGEX: /^(.+?)(?:\/([\S\t]*?))?(?:(?:\s(?=.*?:.))(.+))?$/,
  // 1. key, 2. value
  SPLIT_DATA_REGEX: /(\S+):(\S+)/,

  MAX_PHONET_SUGGESTIONS: 2,
  MAX_SUGGESTIONS: 15,
  GOOD_EDITS: ["spaceword", "uppercase", "replchars"] as string[],

  NGRAM_MAX_ROOTS: 100,
  NGRAM_MAX_GUESSES: 200,

  PHONET_MAX_ROOTS: 100,

  MAX_CHAR_DISTANCE: 4,

  SPLIT_REGEX_REGEX: /^([^]*)\/([^]+)\/([^]*)$/,

  SPLIT_CONDITION_REGEX: /(\[.+\]|[^\[])/g,

  DEFAULT_BREAK: new Set([/-/g, /^-/g, /-$/g]),

  NUMBER_REGEX: /^\d+(\.\d+)?$/
} as const

export const decoder = new TextDecoder()

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

export enum CompoundPos {
  /** The compound segment is at the beginning of the word. */
  BEGIN,
  /** The compound segment is somewhere in the middle of the word. */
  MIDDLE,
  /** The compound segment is at the end of the word. */
  END
}
