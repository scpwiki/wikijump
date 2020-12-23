[<< Return to the README](README)

## Philosophy

Wikitext is similar to Markdown and dissimilar to C in that the grammar is loose.
Any invalid token combinations are rendered as-is, rather than producing a fatal parsing
error which halts the process. This latter, C-like (or any programming language, really)
philosophy was how the original version of ftml operated. However this presents obvious
incomaptibilities with Wikidot, and the grammar had to be increasingly complicated to handle
edge-case conditions.

This rewrite of ftml performs preprocessing substitions and tokenization like the first version,
but the parser is hand-written to allow for loose fallback rules.

More specifically, for each encountered token, the parser will attempt to match the first rule
which expects it. Incoming tokens will be handled, producing elements, or until an invalid token
is received, at which point an "error" will be produced and this rule will abort.

Following this, the parser will attempt to apply the second rule (if any), etc., until all rules are
exhausted. At this point, if no match can be made, the default "fallback" rule is applied. This is
the case where all the tokens are interpreted literally.

For any tokens which are successfully consumed to produce elements, the pointer to the remaining tokens
is bumped up (really, a later subslice is taken), and the element is appended to the final list.
Note that this operation is applied recursively, so that any containers (elements which contain other
elements) will perform this same operation to populate themselves.

It is important to note that, in accordance to the Wikidot parsing strategy, all "errors" are non-fatal.
In the worst-case scenario, all tokens fail all rules, and all are parsed with the fallback, rule, producing
an error for each incident. In a more typical case, any invalid structures will produce errors, and will
parsed as best it can.

These errors are returned to the caller to provide information on where the process failed, while still
producing the fallback render. This provides the both of best worlds: errors to assist with wikitext
debugging, but also not hard-failing rendering in case of any error.
