# AdSenseUnit Module

- Feature ID: `module-adsenseunit`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `AdSenseUnit` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:adsenseunit-module/source.wikidot.txt:1` through line 32 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:adsenseunit-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:adsenseunit-module/source.wikidot.txt:1` through line 32  
SHA-256 of complete source file: `4c0d3e76ed8798a40099c1bb0f13f3aa982a8666ca4dcbe2055b5c08ebd1453c`

```wikidot
L0001 [[note]]
L0002 This module was **deprecated** when Wikidot's AdSense integration was removed on 6 April 2010, in favour of a more flexible system.
L0003 
L0004 Paying users are now able to use a much larger variety of advertisement providers - and are no longer limited to just Google AdSense.
L0005 
L0006 You can read the [http://blog.wikidot.com/blog:advertising official announcement] if you'd like to know more.
L0007 [[/note]]
L0008 
L0009 [[f>toc]]
L0010 
L0011 The AdSenseUnit module lets you insert Google AdSense ads into a wiki page.  You must have enabled AdSense on your wiki for this module to work.  Normally, Wikidot will provide the code for you and you do not need to create your own code to use the AdSenseUnit module.
L0012 
L0013 The syntax for the AdSenseUnit module is:
L0014 
L0015 [[code]]
L0016 [[module AdSenseUnit arguments...]]
L0017 [[/code]]
L0018 
L0019 ++ Example
L0020 
L0021 This example shows two left-aligned blocks of adverts:
L0022 
L0023 [[code]]
L0024 [[div style="margin-left:auto; margin-right:auto; padding: 10px; margin:0 0em 1em 2em; text-align: center; background-color: transparent; float:left; border: none; width: 15%;"]]
L0025 [[module AdSenseUnit label="your add"]]
L0026 [[module AdSenseUnit label="your add"]]
L0027 [[/div]]
L0028 [[/code]]
L0029 
L0030 ++ Arguments
L0031 
L0032 The **label** argument specifies the label to show, and is mandatory.
```
