# Deviation: root-level Wikidot iftags selection

## Shim

`resolve_outermost_wikidot_iftags` in `deepwell/src/services/render/iftags.rs`, called by the existing Deepwell conditional preparation passes.

## Reason it lives in Wikijump

Selecting an iftags branch depends on saved caller-page tags supplied by Deepwell. This change corrects the existing runtime-owned textual selector so repeated preparation passes preserve Wikidot's one-level nested behavior.

## Why FTML is not yet sufficient

FTML does not yet expose a delayed conditional node that Deepwell can resolve with page-tag context. Moving selection now would either duplicate runtime tag state in FTML or require the handle architecture that `docs/ftml-boundary.md` reserves for explicit architecture review.

## Evidence

The four saved caller-tag states, tagged preview confirmations, authenticated saved-page observations, and cleanup proof are retained at `/home/roku/wjlab/evidence/ftml-oracle-20260713T070955Z/run-iftags-nested`. The machine verdict is `verdict.json` in that directory.

## FTML backlog decision

This remains part of the existing BND-10 textual iftags debt. The correction adds no new conditional language; it replaces recursive evaluation with the evidenced root-level pairing rule and protects nested tokens across existing repeated passes.

## Migration condition

Shrink this shim when FTML preserves iftags as a delayed structured node, including nested token bytes and malformed boundaries, and Deepwell can select only root-level nodes using caller-page tags.

## Owner

Rokurolize.

## Review trigger

Re-evaluate on the next FTML pin that changes conditional parsing, or when a delayed iftags node becomes available.
