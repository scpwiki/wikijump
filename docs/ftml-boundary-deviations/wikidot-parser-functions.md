# Deviation: Wikidot parser functions and literal-region protection

- Shim: `resolve_parser_functions` and its bounded expression parser in
  `deepwell/src/services/render/wikidot_expression.rs`;
  `LiteralRegionIndex` in `deepwell/src/services/render/literal_regions.rs`;
  and the `resolve_wikidot_simple_if_fragments` preparation pass in
  `deepwell/src/services/render/service.rs`.
- Reason it lives in Wikijump: ListPages and CountPages substitute runtime
  values before these parser functions can be selected. Deepwell therefore
  owns evaluation for the currently evidenced arithmetic, boolean, and branch
  forms, while the literal-region index prevents evaluation inside authored
  code and raw regions.
- Why FTML is not yet sufficient: FTML does not preserve `[[#expr]]`,
  `[[#ifexpr]]`, or the relevant `[[#if]]` forms as delayed nodes that expose
  their condition and branches to a runtime evaluator. Passing unresolved
  forms to FTML can instead reinterpret their marker text as unrelated syntax.
- Evidence: final production-path replay runs
  `/home/roku/codex-thread-workspaces/local-wikidot-lab-20260706/evidence/run217-render-replay-ftml-0a2a4d9c-final-det-e-20260711`
  and
  `/home/roku/codex-thread-workspaces/local-wikidot-lab-20260706/evidence/run217-render-replay-ftml-0a2a4d9c-final-det-f-20260711`
  each passed all 20 former render failures with zero parser errors, timeouts,
  crashes, or fallback outcomes. The affected corpus shapes include rating
  expressions, nested branches, and literal examples in code regions.
- FTML backlog decision: this is newly accepted Wikijump-side debt under the
  existing BND-10 delayed-conditional design track recorded in
  `candidate-ftml-backlog.md` of the 2026-07-09 FTML/Wikijump boundary audit.
  Extending FTML first would require the still-open shared design for delayed
  nodes and caller-side evaluation; this PR intentionally implements only the
  bounded corpus-evidenced subset.
- Migration condition: FTML preserves these parser functions and conditionals
  as structured delayed nodes, exposes a reviewed caller evaluation hook, and
  Wikidot-layout fixtures prove nested, malformed, and literal-region behavior.
- Owner: Rokurolize/Wikijump maintainers.
- Review trigger: the next FTML pin bump that changes parser-function or
  conditional handling, or completion of the BND-10 delayed-node design.
