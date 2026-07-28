# NewPage Module

- Feature ID: `module-newpage`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `NewPage` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:newpage-module/source.wikidot.txt:1` through line 73 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:newpage-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:newpage-module/source.wikidot.txt:1` through line 73  
SHA-256 of complete source file: `33cd8e98e4ba9a73150c91eda9458cb87b4360d59b4b54db22afde887e5d3537`

```wikidot
L0001 ++ Description
L0002 
L0003 Displays a form that allows easier creation of new pages.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || {{category}} || no || name of a page category || none || forces the given page category by prepending the page name by the //categoryname:// _
L0009 Note: Cannot use {{_default}} category for this. ||
L0010 || {{template}} || no || name of a template page || none || a page (or comma-separated list of pages) to be used as a template for the new page ||
L0011 || {{size}} || no || any positive integer || 30 || size of the displayed input field ||
L0012 || {{button}} || no || any string || "create page" || text displayed within the //create page// button ||
L0013 || {{format}} || no || any valid regular expression || none || forces the input value to match the required format ||
L0014 || {{tags}} || no || space-separated list of tags || none || automatically adds given tags to created pages ||
L0015 || {{parent}} || no || name of a {{page}} or {{category:page}} || none || automatically adds parent page to created pages ||
L0016 
L0017 +++ Attributes for AutoSave function
L0018 
L0019 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0020 || {{mode}} || no || {{edit}}, {{save-and-refresh}}, {{save-and-go}} || {{edit}} || "edit" takes you to an editor. "save-and-refresh" saves the page and refreshes the current page. "save-and-go" saves the page and goes to it (without editor) unless {{goTo}} attribute is passed ||
L0021 || goTo || no || valid page name || none || specifies which page to go to after automatically saving a page ||
L0022 
L0023 Any page that would be used as a template (passed via the {{template}} attribute) must belong to the {{template}} category, i.e. its name should contain the {{template:}} prefix, e.g. {{template:pagename}}. And must already exist.
L0024 
L0025 If you choose several templates (names separated by a comma) an additional field will be displayed asking to choose a template for the page that a user wishes to create.
L0026 
L0027 If you want new pages to fit match a given pattern, you can use the {{format}} attribute. To learn more about regular expressions you can see the [*http://pl2.php.net/manual/en/reference.pcre.pattern.syntax.php Pattern Syntax description] at the PHP main page.
L0028 Anyway, you could do:
L0029 {{format="/^[0-9]{5}$/"}} -- page names would consist of exactly 5 numbers
L0030 {{format="/^[\d]{4}[- \/.](0[1-9]|1[012])[- \/.](0[1-9]|[12][0-9]|3[01])$/"}} -- a simple expression to match a valid date (not 100% accurate, assumes all months have 31 days)
L0031 etc.
L0032 
L0033 [[note]]
L0034 You cannot use NewPage module to create a hidden page (i.e. page whose name starts with an underscore -- "_"). On the feedback site, there is a wish to change it. If you also feel this way, [http://feedback.wikidot.com/wish:404 rate it up].
L0035 [[/note]]
L0036 
L0037 ++ Examples
L0038 
L0039 To make creating pages within the //doc// category:
L0040 
L0041 [[code]]
L0042 [[module NewPage category="doc"]]
L0043 [[/code]]
L0044 
L0045 Results in:
L0046 
L0047 [[module NewPage category="doc"]]
L0048 
L0049 (you will not be able to create a page in the documentation section - this is just for demonstration purposes).
L0050 
L0051 To use a template:
L0052 
L0053 [[code]]
L0054 [[module NewPage template="template:module"]]
L0055 [[/code]]
L0056 
L0057 To use several templates to choose from:
L0058 
L0059 [[code]]
L0060 [[module NewPage template="template:module,template:howto"]]
L0061 [[/code]]
L0062 
L0063 And now a perfect module to insert into you side-bar for easier page creation:
L0064 
L0065 [[code]]
L0066 +++ Add a new page
L0067 [[module NewPage size="15" button="new page"]]
L0068 [[/code]]
L0069 
L0070 [[div style="width: 13em"]]
L0071 +++ Add a new page
L0072 [[module NewPage size="15" button="new page"]]
L0073 [[/div]]
```
