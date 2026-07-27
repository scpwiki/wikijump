# CountPages Module

- Feature ID: `module-countpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `CountPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:page-selection/source.wikidot.txt:1` through line 122 (included)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:countpages-module/source.wikidot.txt:1` through line 19 (canonical)

## Documentation-derived behavioral evidence

### doc-include:page-selection (included)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:page-selection/source.wikidot.txt:1` through line 122  
SHA-256 of complete source file: `7e3a1f4412b05893a9c2c907b95aaa47fb2d9900a9f7c9ff4933a31e15121fc5`

```wikidot
L0001 [!--
L0002 
L0003 Applies to all modules selecting pages, at least ListPages and CountPages.
L0004 
L0005 --]
L0006 
L0007 Specify one or more of these selectors to refine the set of pages you select.  Each selector adds additional constraints:
L0008 
L0009 ||~ Argument    ||~ Meaning ||
L0010 || pagetype     || Select by type of page ||
L0011 || category     || Select by category ||
L0012 || tags         || Select by tags ||
L0013 || parent       || Select by parent page ||
L0014 || link_to      || Select by outgoing links ||
L0015 || created_at   || Select by date of creation ||
L0016 || updated_at   || Select by date of update ||
L0017 || created_by   || Select by original author ||
L0018 || rating       || Select by rating ||
L0019 || votes       || Select by number of votes ||
L0020 || offset       || Start list after an offset of pages ||
L0021 || range        || Select a range of pages ||
L0022 || name     ||  Select by page name ||
L0023 || fullname || Select by fullname ||
L0024 || _<data-form-field-name>	|| Select by a field's value in a data form ||
L0025 
L0026 Page type selector:
L0027 
L0028 * "normal" means pages without underscore in name (default)
L0029 * "hidden" means pages starting with underscore
L0030 * "*" means all pages, with or without underscores
L0031 
L0032 Category selector:
L0033 
L0034 * "." means current category (default)
L0035 * "*" means all categories
L0036 * else, a list of space/comma delimited categories
L0037 * categories are by default additive (category OR category OR category)
L0038 * "-category" means exclude pages in this category (AND NOT)
L0039 * "%%category%%" means the same category as the current page ( if used on a _template page)
L0040 
L0041 Tags selector:
L0042 
L0043 * "-" means pages with no tags, visible or invisible
L0044 * "=" means pages with any of the same visible tags as this page
L0045 * "==" means pages with the exact same visible tags as this page
L0046 * else, a list of space/comma delimited tags
L0047 * tags are by default additive (tag OR tag OR tag)
L0048 * "-tag" means pages without the tag (AND NOT)
L0049 * "+tag" means pages with the tag (AND)
L0050 
L0051 Parent page selector:
L0052 
L0053 * "-" means pages with no parent page
L0054 * "=" means siblings of current page (same parent)
L0055 * "-=" means with different parent than current page
L0056 * "." means children of current page (parent is this page)
L0057 * else specifies a single full page name
L0058 
L0059 Outgoing links selector:
L0060 * enter a single full name of an existing page to select pages that link to that page
L0061 * while "." means pages that link to current page
L0062 
L0063 Creation date selector:
L0064 
L0065 * "=" means created on same day as current page
L0066 * "yyyy" means specified year
L0067 * "yyyy.mm" means specified year and month
L0068 * optionally prefixed by ">", "<", "=", "<=", ">=", "<>" (default is "=")
L0069 * dates are not site-local but currently all UTC (GMT)
L0070 * "last n unit" or "older than n unit" where 'n' is a count (defaults to 1) and unit is "hours", "day", "week", or "month"
L0071 
L0072 Update date selector:
L0073 
L0074 * dates are not site-local but currently all UTC (GMT)
L0075 * "last n unit" or "older than n unit" where 'n' is a count (defaults to 1) and unit is "hours", "day", "week", or "month"
L0076 
L0077 Author selector:
L0078 
L0079 * "=" means by created by author of current page
L0080 * "-=" means by not created by author of current page
L0081 * else, a single user name
L0082 
L0083 Rating selector:
L0084 
L0085 * "n" means pages with rating equal to n
L0086 * "=" means pages with same rating as current page
L0087 * optionally prefixed by ">", "<", "=", "<=", ">=", "<>" (default is "=")
L0088 
L0089 **Caution:** When listing pages from many categories, where some categories have rating type set to + or +/- and others to "stars" (in Site Manager), selecting and ordering by rating may not funtion properly. The solution is to list and order pages from categories having the same rating mode.
L0090 
L0091 Votes selector:
L0092 
L0093 * "n" means pages with votes equal to n
L0094 * "=" means pages with same number of votes as current page
L0095 * optionally prefixed by ">", "<", "=", "<=", ">=", "<>" (default is "=")
L0096 
L0097 Offset selector:
L0098 
L0099 * "n" means do not show the first n pages (default is 0)
L0100 
L0101 Range selector:
L0102 
L0103 * "." means current page
L0104 * "before" means pages up to but not including current (in order after sorting)
L0105 * "after" means pages after current page (in order after sorting)
L0106 * "others" means pages except current page
L0107 
L0108 Name selector:
L0109 
L0110 * enter a single name (means the name part without the category!) of an existing page to select exact this page of a given category - or pages of different categories if also selected.  You can use a dataform field of the current page
L0111 *  "=" means pages that have exact the same name as the  current page ( makes sence only with other selected categories)
L0112 *  "s%" means all pages starting with given character "s"  or 
L0113 *  "s*" means all pages starting with given character "s"
L0114 
L0115 Fullname selector:
L0116 
L0117 * enter a single fullname  of an existing page to select exact this one page  (you can use a dataform  field of the current page)
L0118 
L0119 Data Form selector:
L0120 * Select by a field's value in a data form
L0121 * Syntax: {{_data-form-field-name="data-form-field-value"}}
L0122 * Example: {{_gender="m"}} - select all pages that have 'm' set as the 'gender' field's value in the Data Form
```

### doc-modules:countpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:countpages-module/source.wikidot.txt:1` through line 19  
SHA-256 of complete source file: `251e9373b8fc9e41c6f39baabeba8e9949d2dc6eb4b10b238efecabb8d9f1795`

```wikidot
L0001 The CountPages module lets count the number of pages that match various criteria.  CountPages is similar to the [/doc-modules:listpages-module ListPages module] in some ways but does not let you render page data, only a single symbol called @@%%total%%@@.
L0002 
L0003 ++ Selecting pages
L0004 
L0005 [[include :www:doc-include:page-selection]]
L0006 
L0007 ++ Example
L0008 
L0009 [[code]]
L0010 [[module CountPages category="wiki,blog" tags="_closed"]]
L0011 %%total%% active pages.
L0012 [[/module]]
L0013 [[/code]]
L0014 
L0015 ++ Notes
L0016 
L0017 * You can put wiki syntax (e.g. for links) into the module body.
L0018 * This module (like most) cannot be used inside a ListPages module.
L0019 * @@%%count%%@@ can be used as a synonym for @@%%total%%@@.
```
