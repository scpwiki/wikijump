# PetitionAdmin Module

- Feature ID: `module-petitionadmin`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `PetitionAdmin` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:petitionadmin-module/source.wikidot.txt:1` through line 16 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:petitionadmin-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:petitionadmin-module/source.wikidot.txt:1` through line 16  
SHA-256 of complete source file: `39d06571629be6648c9b905744dacec92c458f22baec55363bc08b387c5e02bd`

```wikidot
L0001 ++ Description
L0002 
L0003 This module allows you to set up and manage online internet petitions. You can set up several campaigns to collect signatures, download data directly into your spreadsheet application (OpenOffice Calc, Gnumeric or Excel) etc. Just awesome. One of the examples can be found here: http://noooxml.wikidot.com/petition
L0004 
L0005 ++ Attributes
L0006 
L0007 None required. During the setup you will be instructed how to configure your site to run the petition campaign.
L0008 
L0009 ++ Example
L0010 
L0011 Put this piece of code on any of your (admin) pages:
L0012 [[code]]
L0013 [[module PetitionAdmin]]
L0014 [[/code]]
L0015 
L0016 You will be able to administer your petition from the above module provided that you have administrative rights within the site.
```
