# Clone Module

- Feature ID: `module-clone`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Clone` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:clone-module/source.wikidot.txt:1` through line 30 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:clone-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:clone-module/source.wikidot.txt:1` through line 30  
SHA-256 of complete source file: `fe1add5f9cb58cc0254127fbf41c74b62661040d989e604b86326116ba50cc22`

```wikidot
L0001 ++ Description
L0002 
L0003 Makes a copy of part of, or all of the current or a specified site.  The current logged user (who invokes the clone action) becomes the owner (master administrator) of the specified site.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || source || no || text string,{{"."}} || current site || Specifies the site to clone.  If not specified, clones the current site.  If the source site is private, the user must be a member of the site. ||
L0009 || button || no || text string  || "Clone this site" || Specifies the text for a button link. ||
L0010 
L0011 ++ How it works
L0012 
L0013 The Clone module creates a button that occupies a line by itself.  Clicking on the button shows a pop-up that asks for a destination site (which may not exist).  When the clone operation is complete, the Clone module takes the user directly to the destination site.
L0014 
L0015 The clone has all the pages, attached files, and configuration of the original site, however:
L0016 
L0017 * It has only one member, the cloner, who is the new site's master admin
L0018 * It is marked as private, even if the template was public
L0019 
L0020 ++ Examples
L0021 
L0022 On a template site, offer the user the opportunity to copy the site:
L0023 
L0024 [[code]]
L0025 [[module Clone]]
L0026 [[/code]]
L0027 
L0028 ++ Notes
L0029 
L0030 Sites, even public ones, are at present clonable by default.  This will be switched for Pro users.
```
