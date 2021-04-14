import type { Tag, tags } from '@codemirror/highlight'
import type { NodePropSource } from 'lezer-tree'
import type { GrammarContext } from './grammar'

/** A Tarnation grammar definition. */
export interface Grammar {
  /** {@link Action} to fallback onto when the grammar can't find a match. */
  fallback?: Action | ActionObject
  /** Specifies if the grammar is case sensitive. */
  ignoreCase?: boolean
  /** A list of {@link Bracket} definitions.
   *  To be used with the special `@BR` node type. */
  brackets?: Bracket[]
  /** A list of {@link Variable} definitions.
   *  These can be referenced in a {@link Match}. */
  variables?: Record<string, Variable>
  /** The starting {@link State} of the grammar. */
  start?: string
  /** A list of {@link Rule} definitions (a {@link State}) that the grammar will always try to match last. */
  global?: State
  /** The primary list of {@link State} definitions. */
  states: Record<string, State>
}

/** The `Bracket` definition is a way of defining opening / closing pair nodes easily.
 *  A bracket can be referenced in a {@link Type} using a special syntax.
 *
 *  `@BR</O|/C><:[hint]>`, where anything in `<>` brackets is optional.
 *  The `/O` and `/C` symbols specify directly if the node is an opening or closing delimiter.
 *
 *  Finally, you can choose not to provide the `pair` property. If `pair` isn't provided,
 *  no actions will be generated for the brackets/delimiters, but the node names will
 *  still be generated. A name of `Emphasis` would be emitted as `EmphasisOpen`, `EmphasisClose`.
 *
 *  @example
 *  rule = [/(\{)(.*?)(\})/, ['@BR', '', '@BR']]
 *  rule = [/(\{)(.*?)(\})/, ['@BR/O', '', '@BR/C']]
 *  rule = [/(\{)(.*?)(\})/, ['@BR:curly', '', '@BR:curly']] */
export interface Bracket {
  /** The `name` property provides the prefix for the `Open` and `Close` nodes of the bracket.
   *  e.g. providing `Variable` would yield the node types `VariableOpen` and `VariableClose`. */
  name: Type
  /** The `pair` property is the pair of strings that specify the opening and closing delimiters.
   *  If only a string is provided, the string will be duplicated into an array pair automatically. */
  pair?: [string, string] | string
  /** The `hint` property effectively "namespaces" a bracket.
   *  This is so that conflicting bracket characters can be used. */
  hint?: string
  /** The `tag` property is a helper utility for assigning a highlighting tag to the brackets. */
  tag?: TagType | ''
  /** The `parented` property is a flag for making it so that bracket highlighting is based off the parent.
   *  The "parent" in this case is the `name` property.
   *
   *  Defaults to `true` if the `pair` property isn't provided. */
  parented?: boolean
}

/** A list of {@link Rule} definitions.
 *  A `State` can be switched to by an {@link Action} or simply used as reservoir of rules for other states.
 *  To include the rules of a state into another state, you can use an {@link IncludeDirective}. */
export type State = (Directive | Rule | RuleState)[]

/** Directives are special rules that don't have direct tokenization behavior. */
export type Directive = IncludeDirective | PropsDirective | StyleDirective | BracketsDirective | VariablesDirective

/** @see IncludeDirective#include */
export interface IncludeDirective {
  /** A directive that includes the rules of another {@link State}.
   *  @example
   *  include = { include: '#foo' } */
  include: StateRef
}

/** @see PropsDirective#props */
export interface PropsDirective {
/** A directive that allows for an inline configuration of node types.
 *
 *  Directives are simply stringed together as they're found - this directive is for convinence,
 *  and doesn't have any scoping behavior.
 *  @example
 *  props = { props: [
 *    NodeProp.group.add({ 'FieldDeclaration': ['Declaration'] })
 *  ] } */
  props: NodePropSource[]
}

/** @see StyleDirective#style */
export interface StyleDirective {
  /** A directive that allows for an inline configuration of node highlighting styles.
   *  This directive is simply a shorthand for using a {@link PropsDirective}.
   *
   *  Directives are simply stringed together as they're found - this directive is for convinence,
   *  and doesn't have any scoping behavior.
   *  @example
   *  style = { style: {
   *    'Function': t.function(t.name)
   *  } } */
  style: Record<string, Tag | readonly Tag[] | undefined>
  // typescript bug makes me require undefined here, I have no idea why
}

/** @see BracketsDirective#brackets */
export interface BracketsDirective {
  /** A directive that allows defining {@link Bracket} definitions inline.
   *
   *  Directives are simply stringed together as they're found - this directive is for convinence,
   *  and doesn't have any scoping behavior.
   *  @example
   *  brackets = { brackets: [
   *    { name: 'BlockComment', tag: 't.blockComment' }
   *  ] } */
  brackets: Bracket[]
}

/** @see VariablesDirective#variables */
export interface VariablesDirective {
  /** A directive that allows defining {@link Variable} definitions inline.
   *
   *  Directives are simply stringed together as they're found - this directive is for convinence,
   *  and doesn't have any scoping behavior.
   *
   *  There is a caveat to that last statement - variables are fetched and stored _as they're found_, not
   *  before any rules are processed. You need to declare variables - when they're inline - before a rule
   *  can use them.
   *
   *  Additionally, variables are added by merging the new variables into the global variables object.
   *  This means that variables - when inline - can overwrite each other.
   *  @example
   *  variables = { variables: {
   *    keywords: ['const', 'let', 'var']
   *  } } */
  variables: Record<string, Variable>
}

/** A `Rule` is a definition that, altogether, describes a {@link Match} <-> {@link Action} pair.
 *  It can be given either as a terse array or a verbose object. As an array, it has a very loose syntax.
 *
 *  An array rule has the following order(s):
 *  ```raw
 *  [match -> type? -> group? -> next? -> extraOptions?]
 *  [match -> substate]
 *  ```
 *  @example
 *  rule = [/(\{\$)(.*?)(\})/, 'IncludeVariable', ['@BR:vi', 't.variableName', '@BR:vi']]
 *  rule = [/@@.*?@@/, 't.escape']
 *  rule = [/(@bsc)(\S+?)(\s*@be)/, 'BlockContainerNode', [
 *    't.bracket',
 *    { optional: false, state: [['module', 't.keyword'], [/\S+?/, 't.tagName']] },
 *    't.bracket'
 *  ], { parser: '>>/BlockContainer' }]
 *  @see Match
 *  @see Action
 *  @see SubState */
export type Rule = RuleArray | RuleObject

type RuleArray =
  | [match: Match]
  | [match: Match, ...action: Action ]

type RuleObject = { match: Match } & (ActionObject | SubState)

/** An `Action` definition tells the grammar what to do when a {@link Rule} has {@link Match | matched} something.
 *  It can be given either as a terse array or a verbose object. As an array, it has a very loose syntax.
 *
 *  An array rule has the following order(s):
 *  ```text
 *  [type? -> group? -> next? -> extraOptions?]
 *  [type? -> substate?]
 *  ```
 *  @see ActionObject for info on the properties that can be given.
 *  @see Rule for more info and examples. */
export type Action =
  | [action: ActionObject]
  | [substate: SubState]
  | [type: Type, substate: SubState]
  | [type: Type,                             opts?: ActionObject]
  | [type: Type, group?: Group, next?: Next, opts?: ActionObject]
  | [type: Type,                next?: Next, opts?: ActionObject]
  | [type: Type, group?: Group,              opts?: ActionObject]
  | [            group:  Group,              opts?: ActionObject]
  | [            group:  Group, next?: Next, opts?: ActionObject]

/** Verbose object variant of an {@link Action}.
 *
 * | | |
 * | :-- | :-- |
 * | `type`         | Assigns the matched text to the specified type.                          |
 * | `group`        | Assigns additional actions to the capturing groups of the match.         |
 * | `next`         | Pushes, or pops states from the stack.                                   |
 * | `switchTo`     | Switches to states without pushing additional states on the stack.       |
 * | `embedded`     | Informs the parser what language to nest with, or to stop nesting with.  |
 * | `parser`       | Directs the parser to open or close syntax blocks.                       |
 * | `context`      | Mutates a persistent context object that the grammar for state.          |
 * | `log`          | Logs a message whenever the associated rule is matched.                  | */
export interface ActionObject {
  /** The `type` specifies what node type matched text should be "scoped" or "tagged" with in the emitted tree.
   *  It can be specified in a few different ways:
   *  ```text
   *  '[Name]'      | Capitalized custom scope name.
   *  't.[tagname]' | Shorthand/helper for automatically using a CodeMirror highlighting tag.
   *  '@RE'         | Special type that causes the tokenizer to completely reverse the current match's progress,
   *                  and then restart the tokenizer from that point again. The purpose of this is that state changes
   *                  are still processed. This allows you to 'cancel' or 'lookahead' with state changes.
   *  '@BR'         | See `Bracket` for usage.
   *  ```
   *  @example
   *  rule = [/foo/, 'CustomName']
   *  rule = [/foo/, 't.keyword']
   *  rule = [/foo/, '@RE']
   *  rule = [/foo/, '@BR']
   *  @see Bracket */
  type?: Type,
  /** Describes a group of {@link Action} objects.
   *  Each particular `Action` is associated, in order, to the capturing groups of a rule's regex.
   *  If there is no capturing groups, the group itself is invalid.
   *  @example
   *  rule = [/(match1)(match2)(match3)/, [action1, action2, action3]] */
  group?: Group,
  /** The {@link Next} {@link State} to go to before the next match.
   *  A state's name must be preceeded with a `#`.
   *
   *  It can take three special values: `@pop`, `@popall`, and `@push`.
   *
   *  This property can be used with a {@link Substitute}.
   *  @example
   *  action = { next: '#next_state' }
   *  action = { next: '@pop' } */
  next?: Next | Substitute
  /** The `switchTo` property is like the `next` property,
   *  except the state specified is switched to without altering the stack.
   *
   *  Special {@link Next} values can't be used with this property.
   *
   *  This property can be used with a {@link Substitute}.
   *  @see Next */
  switchTo?: StateRef | Substitute
  /** The `parser` property attaches special meaning to the tokens it is defined on.
   *  Tokens with this property inform the parser to make special decisions
   *  regarding opening and closing syntax nodes.
   *
   *  The syntax for the property is: `[mode][type]`, where `[type]` is any non-special {@link Type},
   *  and `[mode]` is either `>>` or `<<`. `>>` is _inclusive_, and `<<` is _exclusive_.
   *  Following the arrows with a `/` indicates closing, rather than opening.
   *  @example
   *  action = { parser: '>>BlockComment' }
   *  action = { parser: '>>/BlockComment' }
   *  action = { parser: ['<<BlockNode', '>>/BlockContainer'] } */
  parser?:  ParserTarget | ParserTarget[]
  /** The `embedded` property looks somewhat like the `next` property,
   *  but instead of states it nests embedded languages. Unlike `next`, you cannot stack `embedded`.
   *  It is more like a flag that is set, with the tokenizer tracking what range of text should be
   *  filled in with the specified language.
   *
   *  If an exclamation mark is found at the end of the language name, that will signify
   *  to the tokenizer that the specified language should be embedded for only this token.
   *  Do note: if you use this feature, other properties of the rule won't be considered
   *  when tokenizing.
   *
   *  This property can be used with a {@link Substitute}. */
  embedded?: '@pop' | `${Substitute | string}${'!' | ''}`,
  /** The `context` property defines a set of values to be added to the current stack context.
   *  These values are "merged" into the current context, rather than replacing the current context in its entirety.
   *
   *  You can use the value of a `context` property in any {@link Substitute} compatible property.
   *  To do this, you use the `::[key]` syntax.
   *
   *  Additionally, the value of the keys can accept substitutions.
   *  @example
   *  action = { context: { myValue: 'foo' } }
   *  rule = ['::myValue', 'foo', 'SomeAction'] */
  context?: Context
  /** The `log` property logs (with `console.log`) the
   *  specified message whenever the associated rule is matched.
   *
   *  This property can be used with a {@link Substitute}, one or more, anywhere in the message. */
  log?: string
}

/** Describes a group of {@link Action} objects.
 *
 *  Each particular `Action` is associated, in order, to the 'capturing' groups of a rule's {@link Match}.
 *  If there are no capturing groups, or a disjointed quantity of them, the group is invalid and will throw.
 *  @example
 *  rule = [/(match1)(match2)(match3)/, [action1, action2, action3]] */
export type Group = (Type | Action | ActionObject | SubState)[]

/** A `SubState` definition is a {@link State}-like syntax that the grammar can "fall into".
 *  This allows a parser to take a chunk of matched text and _parse it again_, except with different rules.
 *  The results will be inserted into the final group of matches.
 *
 *  This definition extends {@link ActionObject} - but excludes the `group` property.
 *
 *  | | |
 *  | :-- | :-- |
 *  | `repeat`       | Sets the state to repeatedly match until the matched string is consumed. |
 *  | `optional`     | Determines if the substate is allowed to fail to match any rules.        |
 *  | `all`          | Determines if the substate must match every character to be valid.       |
 *  | `strict`       | Helper property to set the `repeat`, `optional`, and`all` properties.    |
 *  | `state`        | List of substate rules.                                                  | */
export interface SubState extends Omit<ActionObject, 'group'> {
  /** The `repeat` property specifies if the substate will repeatedly match against the captured string.
   *  If it is false, the substate can only match one rule before the substate is exited.
   *
   *  Defaults to `true`. */
  repeat?: boolean,
  /** The `optional` property specifies if it is valid for a substate to not match any of its given rules.
   *  If it is false, then the substate _must_ match something for it not to invalidate the rule.
   *
   *  Defaults to `true`. */
  optional?: boolean,
  /** The `all` property specifies if the substate must provide a token
   *  for every character of the captured string.
   *
   * Defaults to `false`. */
  all?: boolean,
  /** The `strict` property is a helper property
   *  which sets the properties `repeat`, `optional`, and `all` to `false`, `false`, and `true` respectively.
   *
   * Defaults to `true`. */
  strict?: boolean,
  /** The `rules` property is the list of
   *  {@link Rule}, {@link SubRule}, or {@link IncludeDirective} definitions for the state.
   *
   *  Alternatively, a state reference (e.g. `'#foo'`) can be provided,
   *  which acts like an {@link IncludeDirective}. */
  rules: (Directive | SubRule | Rule | RuleState)[] | StateRef
}

/** A special type of {@link Rule} that allows for a `target` property to be provided.
 *  The `target` property is a {@link Substitute} string that specifies what the rule's {@link Match}
 *  will test against.
 *
 *  It is important to note that the subrule's match will be _inherited_, rather than capturing strings from the
 *  specified target. */
export type SubRule =
  | [target: SubRuleTarget, ...rule: RuleArray ]
  | { target: SubRuleTarget } & RuleObject

// TODO: document
export interface RuleState {
    begin: Rule
    end: Rule
    type?: TagType | CustomType | ''
    embedded?: `${Substitute | string}!`
    rules?: StateRef | (Directive | Rule | RuleState)[]
}

/** A special function can be provided in place of a {@link Matchable}.
 *  It is valid to return `[]`, an empty array.
 *  This signifies a successful match - but doesn't advance the input. */
export type MatchFunction = (cx: GrammarContext, str: string, pos: number) => string[] | null

// Basic Types

// sins... deep sins
type Alphabet =
  |'A'|'B'|'C'|'D'|'E'|'F'|'G'|'H'|'I'|'J'|'K'|'L'|'M'
  |'N'|'O'|'P'|'Q'|'R'|'S'|'T'|'U'|'V'|'W'|'X'|'Y'|'Z'

/** The `type` specifies what node type matched text should be "scoped" or "tagged" with in the emitted tree.
 *  It can be specified in a few different ways:
 *  ```raw
 *  '[Name]'      | Capitalized custom scope name.
 *  't.[tagname]' | Shorthand/helper for automatically using a CodeMirror highlighting tag.
 *  '@RE'         | Special type that causes the tokenizer to completely reverse the current match's progress,
 *                  and then restart the tokenizer from that point again. The purpose of this is that state changes
 *                  are still processed. This allows you to 'cancel' or 'lookahead' with state changes.
 *  '@BR'         | See `Bracket` for usage.
 *  ```
 *  @example
 *  rule = [/foo/, 'CustomName']
 *  rule = [/foo/, 't.keyword']
 *  rule = [/foo/, '@RE']
 *  rule = [/foo/, '@BR']
 *  @see Bracket */
export type Type = SpecialType | TagType | CustomType | ''

/** Ordinary node type. */
type CustomType = `${Alphabet}${string}`
/** CodeMirror highlighting tag. */
type TagType = `t.${keyof typeof tags}`
/** Special rematching or bracket shorthand tag. */
type SpecialType = '@RE' | (`@BR${'/O' | '/C' | ''}${`:${string}` | ''}`)

/** References a key in the grammar's current {@link Context} object. */
export type ContextRef = `::${string}`
/** References a variable in the grammar's {@link Language#variables} object. */
export type VariableRef = `@${string}`
/** References a named state in the grammar's {@link Language#states} object. */
export type StateRef = `#${string}`

/** Most {@link Action} properties can make use of _substitutions_,
 *  which are literal substitutions (substring replacement)
 *  derived from either the matched text or the current state/sub-states.
 *
 *  | | |
 *  | :-- | :-- |
 *  | `$#`        | Substitutes the rule's match, or match group in a group match.           |
 *  | `$[n]`      | Substitutes for the *n*th capture group. The entire match is group `$0`. |
 *  | `$S`        | Substitutes the current state of the grammar.                            |
 *  | `::[name]`  | Substitutes for a key in the grammar's current {@link Context} object.   | */
export type Substitute = `\$${'S' | '#' | number}` | ContextRef

/** @see SubRule */
export type SubRuleTarget = Substitute

/** A `variable` is a referrable {@link Match}-like object (string, string array, or regex).
 *  It can be referenced through the `@[var]` syntax inside of a match.
 *  @example
 *  control = /[>+-]/
 *  rule = [/\w+(?!@control)/] */
export type Variable = RegExp | string | string[] | MatchFunction

/** Specifies a {@link State} to go to. A state's name must be preceeded with a `#`.
 *
 *  Can take three special values: `@pop`, `@popall`, and `@push`.
 *  @example
 *  action = { next: '#next_state' }
 *  action = { next: '@pop' } */
export type Next = '@push' | '@popall' | '@pop' | StateRef

/** A `match`, or `matcher`, describes a pattern to match against text.
 *
 *  This pattern ({@link Matchable}), which can be by itself or in a chain of patterns,
 *  is either a: `RegExp`, `string`, {@link Substitute}, or a {@link MatchFunction}.
 *  Using a `string` is the most efficient, as this is checked through a more optimized method than a regex.
 *
 *  You can reference any {@link Variable} using the `@[var]` syntax.
 *
 *  With a `RegExp` match, Tarnation supports full lookahead and lookbehind, up to a certain search length.
 *  @example
 *  rule = [/foo/, 'bar']
 *  rule = ['foo', 'bar']
 *  rule = [[/foo/, /bar/, /@variable/], 'bar']
 *  rule = [['foo', '@variable1', /@variable2/], 'bar']
 *  @see Matchable */
export type Match = Matchable | Matchable[] | '@DEFAULT'

/** A {@link Match} compatible pattern.
 *  If null, this signifies an error - this is to allow a grammar to safely ignore an errored rule. */
export type Matchable = MatchFunction | RegExp | string | Substitute | null

/** Defines a set of values to be added to the current stack context.
 *  These values are "merged" into the current context, rather than replacing the current context in its entirety.
 *
 *  You can use the value of the current `context` in any {@link Substitute} compatible property.
 *  To do this, you use the `::[key]` syntax.
 *
 *  Additionally, the value of the keys can accept substitutions.
 *  @example
 *  action = { context: { myValue: 'foo' } }
 *  rule = ['::myValue', 'foo', 'SomeAction'] */
export type Context = Record<string, string | undefined>

/** Describes a `parser` directive for how to handle the nesting of nodes.
 *
 *  The syntax is: `[mode][type]`, where `[type]` is any non-special {@link Type},
 *  and `[mode]` is either `>>` or `<<`. `>>` is _inclusive_, and `<<` is _exclusive_.
 *  Following the arrows with a `/` indicates closing, rather than opening.
 *
 *  This type can be used with a {@link Substitute}.
 *
 *  Note: For the sake of TypeScript performance, the string template of this
 *  type is simplified. Regardless, it only accepts what a non-special {@link Type} would accept.
 *
 *  ```ts
 *  // actual type
 *  type ParserTarget = `${'<<' | '>>' | '<</' | '>>/'}${TagType |  CustomType | Substitute}`
 *  ```
 *  @example
 *  action = { parser: '>>BlockComment' }
 *  action = { parser: '>>/BlockComment' }
 *  action = { parser: ['<<BlockNode', '>>/BlockContainer'] }
 *  @see Type */
export type ParserTarget = PTInclusiveOpen | PTInclusiveClose | PTExclusiveOpen | PTExclusiveClose

/** Inclusively open a nesting element. */
type PTInclusiveOpen = `${'>>'}${string}`
/** Inclusively close a nesting element. */
type PTInclusiveClose = `${'>>/'}${string}`
/** Exclusively open a nesting element. */
type PTExclusiveOpen = `${'<<'}${string}`
/** Exclusively close a nesting element. */
type PTExclusiveClose = `${'<</'}${string}`
