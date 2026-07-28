# Rate Module

- Feature ID: `module-rate`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Rate` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:rate-module/source.wikidot.txt:1` through line 42 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:rate-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:rate-module/source.wikidot.txt:1` through line 42  
SHA-256 of complete source file: `de1cfd1840b15431b1c8c69f65bbcfd7ee4e084981e2a2ee710e5f1d59815f4c`

```wikidot
L0001 ++ Description
L0002 
L0003 Displays a widget used to rate pages. Page rating must be enabled in the Site Manager.
L0004 
L0005 ++ Attributes
L0006 
L0007 No attributes required.
L0008 
L0009 ++ Examples
L0010 
L0011 [[code]]
L0012 [[module Rate]]
L0013 [[/code]]
L0014 
L0015 results in:
L0016 
L0017 [[module Rate]] 
L0018 
L0019 or displays five-stars rating mode depending on the setting in Site Manager -> Page Ratings -> Type
L0020 
L0021 You can use certain variables when using 5-star type:
L0022 
L0023 ||~ Variable ||~ Description ||
L0024 || %%rating%% || Displays rating number value ||
L0025 || %%rating_votes%% || Displays number of votes ||
L0026 || %%rating_percent%% || Displays overall average rating in percent (without the '%' character) ||
L0027 || %%rating_decimal%% || Displays decimal value of 5-star rating (e.g. 4.7) ||
L0028 
L0029 For example:
L0030 [[code]]
L0031 [[module Rate]]
L0032 Average Rating %%rating%% from %%rating_votes%% votes
L0033 [[/module]]
L0034 [[/code]]
L0035 
L0036 To make this centered use:
L0037 
L0038 [[code]]
L0039 [[=]]
L0040 [[module Rate]]
L0041 [[/=]]
L0042 [[/code]]
```
