import type { Trie } from "../trie"
import type { Affix, Prefix, Suffix } from "./affix"
import type { CompoundPattern } from "./compound-pattern"
import type { CompoundRule } from "./compound-rule"
import type { ConvTable } from "./conv-table"
import type { PhonetTable } from "./phonet-table"
import type { RepPattern } from "./rep-pattern"

/**
 * A flag is just a string label for some special property. This may mark a
 * word as being valid for some {@link Prefix}/{@link Suffix}, as an example.
 */
export type Flag = string

/** A set of {@link Flag}s. */
export type Flags = Set<Flag>

/**
 * A set, of a set, of {@link Flag}s. This nesting is often needed when
 * representing a word *as* its flags, e.g. when checking if a compound is valid.
 */
export type FlagSet = Set<Flags>

/**
 * A mapping of {@link Flag}s to {@link Prefix}es. A flag may correspond to
 * multiple prefixes, so they're stored in a set.
 */
export type PrefixMap = Map<Flag, Set<Prefix>>

/**
 * A mapping of {@link Flag}s to {@link Suffix}es. A flag may correspond to
 * multiple suffixes, so they're stored in a set.
 */
export type SuffixMap = Map<Flag, Set<Suffix>>

/** An index, more specifically a {@link Trie}, of {@link Affix}es. */
export type AffixIndex = Trie<Set<Affix>>

/** An index, more specifically a {@link Trie}, of {@link Prefix}es. */
export type PrefixIndex = Trie<Set<Prefix>>

/** An index, more specifically a {@link Trie}, of {@link Suffix}es. */
export type SuffixIndex = Trie<Set<Suffix>>

/**
 * A set of sets representing similar characters to try when doing
 * suggestions, e.g. `aáã`. Characters found in this map are here because
 * they're a frequent source of typos.
 */
export type CharacterMap = Set<Set<string>>

export interface AffData {
  /** Encoding format. Unused in Espells. */
  SET: string // unused

  /**
   * Flag format type, as in determines how they are parsed from a string.
   * Can be one of four values:
   *
   * - `short` (default): Each flag is one character.
   * - `long`: Each flag is two characters.
   * - `numeric`: Each flag is a number, separated with `,` commas.
   * - `UTF-8`: Each flag is a single UTF-8 character.
   */
  FLAG: "short" | "long" | "numeric" | "UTF-8"

  /**
   * ISO language code. Only effect this currently has in Espells is
   * determining when to use {@link TurkicCasing} rules.
   */
  LANG?: string

  /**
   * Informs Hunspell's tokenizer how to parse out words from a string by
   * providing additional characters to consider as "part of words".
   * Espells doesn't have a tokenizer so this is unusued.
   */
  WORDCHARS?: string // unused

  /** A set of characters to ignore when doing lookups. Useful for diacritics. */
  IGNORE?: Set<string>

  /**
   * If true, the spellchecker will make special considerations to the
   * "sharp S", i.e. the "ß" character in German. This enables certain
   * checks littered throughout Espells, but ultimately the main effect of
   * this is enabling the {@link GermanCasing} casing handler.
   */
  CHECKSHARPS: boolean

  /**
   * Flag that marks a word as forbidden. Primarily, this flag is used to
   * constrain word compounding, as in to mark certain "technically valid"
   * compounded words as actually invalid. An example could be "decreated",
   * which isn't really considered a real word but is a otherwise perfectly
   * acceptable compound word in English.
   */
  FORBIDDENWORD?: Flag

  /**
   * A string of characters separated by `|` pipe characters, representing
   * rows of a keyboard. This is used to try and find spelling mistakes
   * caused by hitting the wrong key on the keyboard.
   */
  KEY: string

  /**
   * A general list of characters that can be found in words for this
   * language, sorted in order of likelihood, most probable first. Affects
   * how certain suggestions are sorted.
   */
  TRY: string

  /**
   * A flag that marks a word as "not to be suggested", but is otherwise
   * considered correctly spelled word. Mostly used for obscenities or
   * unusual words.
   */
  NOSUGGEST?: Flag

  /**
   * A flag that informs the spellchecker that a word cannot be considered
   * valid unless cased *exactly* how it was given in the dictionary file.
   */
  KEEPCASE?: Flag

  /**
   * A list of {@link RepPattern} objects to use when trying to find good
   * suggestions. Specifically, a {@link RepPattern} represents a common
   * typo (usually not an entire word, just a segment) and how to fix it.
   */
  REP: Set<RepPattern>

  /**
   * Sets of similar characters to try when doing suggestions, e.g. `aáã`.
   * Characters found in this map are here because they're a frequent
   * source of typos.
   */
  MAP: CharacterMap

  /**
   * If true, suggestions that are a word split into two will never be
   * given. Apparently a must-have for Swedish, if going by LibreOffice's
   * `sv_SE` dictionary.
   */
  NOSPLITSUGS: boolean

  /**
   * Table of metaphone transformations. These transformations may provide
   * superior suggestions because they describe similar sounding (as in
   * spoken) syllables, which will be used by the suggestion engine to find
   * words that may not be spelled similarly but *sound* similar.
   *
   * Dictionaries that use this feature are unfortunately quite rare.
   */
  PHONE?: PhonetTable

  /** Limits the number of compound suggestions given. */
  MAXCPDSUGS: number

  /**
   * Sets the maximum number of ngram suggestions. Setting this to 0
   * disables the ngram check entirely.
   */
  MAXNGRAMSUGS: number

  /**
   * Sets the similarity factor for ngram based suggestions.
   *
   * - `5`: The default value.
   * - `0`: Fewer ngram suggestions, but always at least one.
   * - `10`: Maximum value, yields `MAXNGRAMSUGS` number of suggestions.
   */
  MAXDIFF: number

  /**
   * If true, all bad ngram suggestions will be removed, rather than
   * keeping at least one.
   */
  ONLYMAXDIFF: boolean

  /**
   * A mapping of {@link Flag}s to {@link Prefix}es. A flag may correspond to
   * multiple prefixes, so they're stored in a set.
   */
  PFX: PrefixMap

  /**
   * A mapping of {@link Flag}s to {@link Suffix}es. A flag may correspond to
   * multiple suffixes, so they're stored in a set.
   */
  SFX: SuffixMap

  /**
   * Flag that marks a word as being invalid if not found with any affixes.
   * Can also be given to a {@link Prefix} or {@link Suffix} to mark that
   * affix as requiring *other* affixes to be valid.
   */
  NEEDAFFIX?: Flag

  /**
   * A flag that marks a {@link Prefix}/{@link Suffix} as only being valid if
   * the word has the corresponding {@link Prefix}/{@link Suffix} with this
   * flag on the opposite end. e.g. if a suffix has the `CIRCUMFIX` flag, a
   * word with that suffix must also have a prefix that has the same
   * `CIRCUMFIX` flag.
   */
  CIRCUMFIX?: Flag

  /**
   * If deep {@link Prefix} stripping is allowed, i.e. more than one prefix.
   * This is a very rare directive in dictionaries - of all of
   * LibreOffice's and Firefox's dictionaries, only Firefox's Zulu has this
   * directive.
   */
  COMPLEXPREFIXES: boolean

  /**
   * If an affix is allowed to remove the entirety of a stem. This is
   * technically unused, but that is because algorithm used by
   * Spylls/Espells naturally handles when this happens.
   */
  FULLSTRIP: boolean

  /**
   * A set of `RegExp` that describes how a word should be split apart into
   * multiple, if needed. e.g. `dashed-word` shouldn't be spellchecked as
   * one word, but as two: `dashed`, and `word`.
   */
  BREAK: Set<RegExp>

  /**
   * A set of {@link CompoundRule} objects, which describe how to construct
   * compound words.
   */
  COMPOUNDRULE: Set<CompoundRule>

  /** Minimum length needed for a word to be checked for compounding. */
  COMPOUNDMIN: number

  /** The maximum number of words within a compounded word. */
  COMPOUNDWORDMAX?: number

  /**
   * A flag that marks a word/affix as valid for being a part of a compound
   * word. This is a general sort of flag - the `COMPOUNDBEGIN`,
   * `COMPOUNDMIDDLE`, and `COMPOUNDEND` flags are a more precise, if more
   * involved, of marking this.
   */
  COMPOUNDFLAG?: Flag

  /**
   * A flag that marks a word/affix as valid for being at the beginning of
   * a compound word.
   */
  COMPOUNDBEGIN?: Flag

  /**
   * A flag that marks a word/affix as valid for being in the middle of a
   * compound word.
   */
  COMPOUNDMIDDLE?: Flag

  /** A flag that marks a word/affix as valid for being at the end of a compound word. */
  COMPOUNDEND?: Flag

  /**
   * A flag that marks a word/affix as valid only if it is a part of a
   * compound word, and never by itself.
   */
  ONLYINCOMPOUND?: Flag

  /**
   * A flag that marks affixes as being valid *inside* of a compound word,
   * not just at the beginning or end of it.
   */
  COMPOUNDPERMITFLAG?: Flag

  /**
   * A flag that markes affixes as not being valid if at the start or end
   * of a compound word.
   */
  COMPOUNDFORBIDFLAG?: Flag

  /**
   * A flag that marks a word as setting the casing rules for the whole of
   * a compound rule. This flag will apply only if said word is at *the
   * end* of the compound word.
   */
  FORCEUCASE?: Flag

  /**
   * If true, uppercased characters will not be allowed at word boundaries
   * inside of compound words.
   */
  CHECKCOMPOUNDCASE: boolean

  /**
   * If true, compounds which are just repeated instances of a stem (e.g
   * `foofoo`) will be forbidden.
   */
  CHECKCOMPOUNDUP: boolean

  /**
   * If true, compounding will be forbidden if the `REP` table can
   * transform the compound word into a *non*-compound word. This is
   * usually a very strong indicator that the compound word was invalid to
   * begin with.
   */
  CHECKCOMPOUNDREP: boolean

  /**
   * If true, compounding will be forbidden if the compound creates a
   * triplet of repeating characters. e.g. `foo + ox : fooox`.
   */
  CHECKCOMPOUNDTRIPLE: boolean

  /**
   * A set of {@link CompoundPattern} objects that describes patterns for
   * matching *invalid* compound words. Any pair of words matched by one of
   * these patterns is invalid.
   */
  CHECKCOMPOUNDPATTERN: Set<CompoundPattern>

  /**
   * If true, simplified 2-letter forms of compounds forbidden by
   * `CHECKCOMPOUNDTRIPLE` will be allowed.
   */
  SIMPLIFIEDTRIPLE: boolean

  /**
   * Apparently needed for special compounding rules in Hungarian. Isn't
   * implemented in either Spylls or Espells.
   */
  COMPOUNDSYLLABLE?: [number, string] // unused

  /**
   * Allows deep twofold suffixes within compounds. Not implemented in
   * Spylls or Espells, and this directive doesn't even have any tests in Hunspell.
   */
  COMPOUNDMORESUFFIXES: boolean // unused

  /**
   * A flag that marks the compounds within the dictionary. Only used by
   * Hungarian. Isn't implemented in Spylls or Espells.
   */
  COMPOUNDROOT?: Flag // unused

  /** Input conversion table that is applied to words before they are checked. */
  ICONV?: ConvTable

  /**
   * Output conversion table applied to suggestions before being returned
   * by the spellchecker.
   */
  OCONV?: ConvTable

  /**
   * An array of flag sets, with the index of said flag set being an
   * "alias" for that set. e.g. if a stem has the flag `1`, that would be
   * the first entry within this table. The `.dic`/`.aff` format uses
   * 1-indexing for this, but Espells accounts for this and converts that
   * to 0-indexing.
   */
  AF: Flags[]

  /**
   * Acts identically in logic to `AF`, but used for {@link Word} `data`
   * properties. e.g. `AM is:gendered` can then be used like `someword/a 1`.
   */
  AM: Set<string>[]

  /**
   * A flag that is marked on usually rare words that *technically* spelled
   * correctly but are usually a mistake. Not implemented in Spylls, but
   * *is* implemented in Espells.
   */
  WARN?: Flag

  /**
   * If true, words marked with the `WARN` flag are instead treated as
   * misspellings and not just warnings.
   */
  FORBIDWARN: boolean

  /**
   * Apparently, going by the *only documentation Hunspell gives*, this is
   * "needed for special compounding rules in Hungarian". Not implemented
   * in Spylls or Espells.
   */
  SYLLABLENUM?: string // unused

  /**
   * A flag that marks words/affixes not to be used in morphological
   * generation and for root words to be removed from suggestions. Not
   * implemented in Spylls or Espells.
   */
  SUBSTANDARD?: Flag // unused
}

/**
 * A variant of {@link AffData} that only contains properties which are
 * safely overridable by a user when instantiating an {@link Espells} instance.
 */
export type OverridableAffData = Omit<
  Partial<AffData>,
  | "REP"
  | "MAP"
  | "PHONE"
  | "PFX"
  | "SFX"
  | "COMPOUNDRULE"
  | "CHECKCOMPOUNDPATTERN"
  | "ICONV"
  | "OCONV"
>
