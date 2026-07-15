# Deviation: Wikidot metacomponent context selection

## Shim

`select_metacomponent_documentation` in `deepwell/src/services/render/metacomponent.rs` recognizes the provenance marker used by imported Wikidot metacomponents and selects whether their documentation belongs to the root render.

## Reason it lives in Wikijump

The selection depends on the saved caller page's tags and on whether the source is the root page or an included page. Deepwell owns both pieces of runtime context. The pass is deliberately applied to individual sources before include concatenation so a root tag cannot reveal documentation originating in an include.

## Why FTML is not yet sufficient

FTML does not preserve Wikidot comments, `iftags`, and includes as delayed structured nodes with source identity. Once Deepwell expands includes into one string, the information needed to distinguish root and included metacomponents has been erased.

## Evidence

The imported `component:croqstyle` source explains all three Wikidot states in its `Begin metacomponent context detection` comment: ordinary-page inclusion hides documentation, inclusion inside a component page's outer `iftags` hides documentation because nested `iftags` is literal, and direct component-page rendering reveals documentation. The runtime corpus copy is page ID `3000000693`, revision ID `3000000692` in the standing runtime database.

## FTML backlog decision

This is new, bounded BND-10 debt. The eventual FTML work must preserve comment and conditional nodes through include composition and expose source identity plus nesting context to the runtime evaluator. Until then, Deepwell treats all included metacomponent regions as hidden. This matches ordinary article inclusion and component inclusion inside the documented outer-`iftags` usage, but intentionally does not emulate the unsupported component-page include outside that wrapper.

## Migration condition

Remove the textual pass when FTML can compose delayed comment, conditional, and include nodes without losing source identity, and Deepwell can select root-level `iftags` using caller tags while preserving Wikidot's nested-conditional behavior.

## Owner

Rokurolize.

## Review trigger

Re-evaluate on the next FTML pin that changes comment, include, or conditional parsing, or when delayed conditional nodes become available.
