# Join Module

- Feature ID: `module-join`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Join` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:join-module/source.wikidot.txt:1` through line 49 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:join-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:join-module/source.wikidot.txt:1` through line 49  
SHA-256 of complete source file: `b0052f9fcc02c51858cfbb9b6644041f65cde47603e3281c5383f8490e878760`

```wikidot
L0001 The {{Join}} module provides an action button that has the same functionality as the "Join this site" action on the top toolbar shown on free sites.  When a user clicks the button, the Join module first ensures that the user has an account (and asks anonymous users to create an account) and then attempts to make the user a member of the site.
L0002 
L0003 The Join module has these arguments:
L0004 
L0005 * **button="text string"** -- specifies the text for the button, which is "Join this site" by default.
L0006 * **class="css-class"** -- specifies CSS class for the {{div}} element containing the button, allowing custom styling
L0007 
L0008 The precise behavior of the Join module depends on the access policy of the site:
L0009 
L0010 * On open sites, the user becomes a member instantly.
L0011 * On closed and private sites, the user must provide a password or apply to join the site, depending on the access policy configuration.
L0012 
L0013 ------
L0014 
L0015 **##880000|Note:##**
L0016 
L0017 When you create the join module on your site you will not see the join button when you save the page, and you might think it hasn't worked. The button doesn't display because you are already a member of your site and the module does not display the button for those that are already a member of your site. This is because they don't need to join the site again.
L0018 
L0019 There are two ways to check to see how the button looks when non-members visit your site:
L0020 * You can edit the page containing @@[[module Join]]@@ and then //preview// the page.
L0021 * You can sign out of your site. After the page refreshes, you will then see the button. You can then sign in again to carry on working on your site (and you won't see the button).
L0022 
L0023 ------
L0024 
L0025 Here is the simplest example of use:
L0026 
L0027 [[code]]
L0028 [[module Join]]
L0029 [[/code]]
L0030 
L0031 Here is an example that specifies the button text:
L0032 
L0033 [[code]]
L0034 [[module Join button="Join this site, it is cool!"]]
L0035 [[/code]]
L0036 
L0037 Join button with custom styling:
L0038 
L0039 [[code]]
L0040 [[module Join class="my-join-button"]]
L0041 [[/code]]
L0042 
L0043 If you use custom styling, remember to define the class in your custom CSS.  The default class for the join box is ".join-box" and you can style this using custom CSS like this:
L0044 
L0045 [[code type="css"]]
L0046 .join-box {
L0047     background-image: url(yourimage.png)
L0048 }
L0049 [[/code]]
```
