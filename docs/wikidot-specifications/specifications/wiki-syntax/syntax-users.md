# Users syntax

- Feature ID: `syntax-users`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented users syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### Current user syntax keeps digit-only text in the name namespace and is actor-invariant for the observed public matrix

- Observation ID: `user-syntax-name-key-special-identities-and-actor-invariance-20260908`
- Classification: `live-boundary`
- Observed at: `2026-09-08`
- Analysis: Retained 2026-09-04 PagePreview evidence shows that real pure-digit Wikidot user IDs authored as [[user ...]] text produce the ordinary missing-user error instead of resolving by numeric ID. A 2026-09-08 combined PagePreview observation then packed a known user, the starred known-user form, system, anonymous, an unknown name, a real numeric ID, and a V7 assignment-shaped name into one source and requested it exactly once as anonymous and once through each of three independent authenticated test accounts. The four response structures are identical after removing only the starred avatar URL timestamp. The known user renders printuser, the starred form renders avatarhover printuser, system renders printuser, anonymous renders the literal text Anonymous, and the unknown, numeric-ID text, and V7 name-key controls render missing-user errors. This evidence establishes the public lookup and actor boundary only; rename, delete, import, and generated-name cache invalidation remain separate local-candidate work.

Normative behavior:

- Authored [[user]] and [[*user]] positional text is resolved through the normalized user-name namespace. Digit-only text must not fall back to the numeric user-ID namespace merely because a Wikidot account with that ID exists.
- For the observed public matrix, anonymous and the three authenticated test actors must receive the same printuser, literal, and missing-user structure. Actor identity must not change user-syntax resolution or disclose an otherwise missing identity.
- The system special identity renders as ordinary printuser markup, while the anonymous special identity renders the literal text Anonymous in the observed user-syntax position.
- The starred user form may vary the avatar URL timestamp over time without changing its printuser/avatarhover identity. That timestamp is temporal presentation data, not actor-specific authority.
- This observation does not define rename, delete, import, or generated-name cache invalidation semantics; those boundaries remain separately actionable until a local candidate or stronger authority establishes them.

Evidence:

- `install/local/wikidot-verification/artifacts/issue1026-user-actor-live-20260908.json` (SHA-256 `ab581243e867a224458888ac4768ad7b38d7abfb02ba68f7968301c5bb2dfae7`), cases: `issue1026-known-user`, `issue1026-known-user-starred`, `issue1026-system-special-identity`, `issue1026-anonymous-special-identity`, `issue1026-unknown-name`, `issue1026-real-numeric-id-as-authored-text`, `issue1026-v7-duplicate-argument-name-key`


### Current user resources use HTTPS and compiled avatars carry a timestamp

Retained October 2026 page and forum observations render new user profile
links as `https://www.wikidot.com/user:info/<slug>`. Avatar and karma images
use the resolved HTTPS avatar base. A starred user's avatar includes the
fragment's compile timestamp; forum header avatars use the request render
time. Preserve the existing double escaping of avatar query separators.

Older compiled fragments can retain HTTP profile links. In particular,
imported forum post bodies preserve their observed HTML independently of
the newly rendered post header. Browser bindings accept both evidenced
schemes. Native Wikijump profile resources retain their own contract.

Evidence: the shared EN/JP corpus `html.xmlrpc.html` observations, the
2026-10-12 SCP-9506 retained browser DOM and thread responses, and the
source/compile-time analysis in
`https://github.com/Rokurolize/wikijump/issues/2380#issuecomment-6122025642`.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:users/source.wikidot.txt:1` through line 5 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:users (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:users/source.wikidot.txt:1` through line 5  
SHA-256 of complete source file: `1170d8628c4fabf4c28e0d866f36f93aa43c4afe5e643b84987d4bf21e438a5b`

```wikidot
L0001 ||~ what you type ||~ what you get ||~ comments||
L0002 || {{@@[[user@@ //user-name//]]}} _
L0003  e.g. {{@@[[user michal frackowiak]]@@}} ||  [[user michal frackowiak]] || user info (no buddy icon)||
L0004 || {{@@[[*user@@ //user-name//]]}} _
L0005  e.g. {{@@[[*user michal frackowiak]]@@}} ||  [[*user michal frackowiak]] || user info (with buddy icon)||
```
