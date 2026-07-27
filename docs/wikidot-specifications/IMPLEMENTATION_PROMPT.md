# Prompt: implement the complete Wikidot feature catalog with TDD

Here is the specification set. Implement every feature listed in `docs/wikidot-specifications/catalog.json` using test-driven development. Treat the catalog as one coherent compatibility campaign rather than creating one pull request per feature.

## Inputs

1. Read the repository's `AGENTS.md` completely.
2. Read `docs/wikidot-specifications/catalog.json`. It is the complete work queue; do not substitute a hand-selected subset.
3. Read `docs/wikidot-specifications/CATALOG.md` and `docs/wikidot-specifications/README.md`.
4. For each catalog item, read the exact Markdown file named by its `specification` field before designing or changing code.
5. Use `docs/wikidot-specifications/source-coverage.json` to inspect corroborating, redirect, runtime-composition, and non-feature source classifications when provenance is relevant.
6. Follow the repository architecture boundaries: FTML owns syntax parsing and rendering primitives; Wikijump/Deepwell owns site, page, query, import, file, permission, actor, module evaluation, and URL state; Framerail owns HTTP and browser runtime behavior.

## Authority and ambiguity

- Implement the documented contract, including legacy names, aliases, defaults, limits, output structure, URLs, permissions, side effects, and stated limitations.
- Do not modernize compatibility-sensitive syntax, DOM, identifiers, or routes.
- Live Wikidot is the behavioral oracle when the snapshot is ambiguous, incomplete, contradictory, or wrong.
- For an `invocation-only`, `high-level-documentation`, or `partially-documented` item, do not invent missing semantics. Design a minimal live-oracle experiment, preserve the evidence and exact fixture, update the specification, and then implement the observed behavior.
- Unsupported or unverified input must fail closed, remain literal, or use an evidenced fallback. It must not silently broaden queries or permissions.

## Mandatory TDD process

Before writing any test, produce a seam map for the current vertical slice and obtain confirmation. State the public interface being tested and why it is the appropriate observable boundary. Suggested seams in each spec are recommendations, not pre-approval.

Then repeat this loop for every behavior in every catalog item:

1. Select one small, user-observable vertical slice.
2. Write one behavior-focused test through the confirmed public seam.
3. Use an independent expected value from the specification or captured live Wikidot evidence.
4. Run the test and demonstrate that it fails for the intended missing behavior (red).
5. Write only enough production code to satisfy that test (green).
6. Run the focused test and the nearest affected suite.
7. Continue with the next learned behavior. Do not write all tests first and all implementation later.

Tests must describe what callers or users observe and must survive internal refactors. Do not test private methods, internal call counts, or database rows through a side channel when a public read interface exists. Do not mock code owned by the repository. Mock only true system boundaries when unavoidable; prefer the real parser, renderer, test database, HTTP route, and browser runtime.

Refactoring is a review-stage activity after a coherent set of red→green slices, not a speculative step inside the loop.

## Required coverage per catalog item

For every item, cover all documented:

- valid syntax and ordinary behavior;
- aliases, legacy spellings, defaults, omitted and empty values;
- limits, boundary values, malformed values, and documented fallbacks;
- argument and feature interactions;
- permissions, visibility, actor, page, category, and site context;
- output text, DOM structure, IDs, classes, links, routes, and side effects;
- escaping, sanitization, and literal/fail-closed boundaries;
- URL, reload, direct navigation, back/forward, and client-runtime behavior where applicable;
- examples and stated limitations.

Add regression tests for every discovered defect. Preserve the original failing input and minimize fuzz or mutation failures into stable fixtures without losing provenance.

## Work tracking

Create a machine-readable implementation ledger keyed by every `catalog.json` feature ID. Each entry must record:

- status: `pending`, `in_progress`, `implemented`, or `blocked`;
- confirmed public seams;
- test files and test names;
- implementation files;
- documentation and live-oracle evidence used;
- unresolved ambiguities or blockers.

There must be exactly one ledger entry per catalog item. An item is not `implemented` merely because adjacent code exists; all explicit behaviors in its specification must have durable tests or a concrete, documented blocker.

Keep the work in one focused campaign and normal review sequence unless repository ownership boundaries require a deliberately coordinated FTML change. Do not split routine discoveries into one pull request per example or per catalog item.

## Validation and completion

Run focused tests during each slice, then run formatting, linting, clippy/build checks, relevant integration suites, verifier suites, and browser tests in proportion to the changed surfaces. For browser-visible behavior, capture fresh evidence against exact source, dependency, fixture, and runtime identities and check visible intermediate states as well as settled DOM.

Do not declare completion until:

- every catalog item has a terminal ledger status;
- every documented behavior has regression coverage;
- every differential or fuzz result is classified;
- no known reproducible compatibility gap lacks a fix or concrete blocker;
- generated catalog/specification validation passes;
- the normal project review and merge process has completed without force or admin merge.

A merge is not a deployment. After browser-visible changes, refresh the standing runtime and verify the served URL before reporting the behavior fixed.
