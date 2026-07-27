# Custom page layouts

- Feature ID: `layout-custom`
- Category: `layout`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Render custom page layouts with the documented placeholders, conditional sections, element order, identifiers, and nesting.

## Implementation contract

- The Wikidot layout renderer MUST emit the documented regions, identifiers, order, and nesting.
- Conditional regions and placeholders MUST use the documented context and visibility rules.
- Browser tests MUST verify final DOM and any user-visible intermediate state.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms
- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:47` through line 116 (canonical)

## Documentation-derived behavioral evidence

### doc:layout-reference (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:47` through line 116  
SHA-256 of complete source file: `bdb2ffc85a5b5e200b2df4a63c32fe5a86a2699a5c8ce58678103af949ab93ba`

```wikidot
L0047 + Custom layout
L0048 
L0049 Users with Pro subscription can create their own custom layout, i.e. HTML structure of the every page on the Wiki inside the {{<body> ... </body>}} tags. In other words, the default layout, which reference is available above, may be altered to fit specific user needs for creating sophisticated and highly custom themes.
L0050 
L0051 For security reasons, user can't use {{<body>}} tag or {{id=""}} elements. Within the layout, you may want to use so called Modules (independent from [http://www.wikidot.com/doc:modules these modules]), i.e. elements which are responsible for rendering vital page and interface elements.
L0052 
L0053 List of available modules:
L0054 
L0055 [[code]]
L0056 [[module NaviBar]] - Wikidot's branded top bar
L0057 [[module FooterBar]] - Wikidot's Interesting Sites
L0058 [[module LoginStatus]] - Sign in/Create account button or User logged in
L0059 [[module PageOptionsBottom]] - Page options: edit, tags etc.
L0060 [[action_area]] - Indicates the position on the page that PageOptionsBottom will use when it needs to display additional content, e.g. a file upload form. It's needed for correct functioning of PageOptionsBottom module
L0061 
L0062 [[module AdModuleAboveContent]] - Ad box for Pro users
L0063 [[module AdModuleBelowContent]] - Ad box for Pro users
L0064 [[module AdModuleAboveSidebar]] - Ad box for Pro users
L0065 [[module AdModuleBelowSidebar]] - Ad box for Pro users
L0066 [[module AdModuleBelowFooter]] - Ad box for Pro users
L0067 [[module Ad label="custom_location"]] - Ad box for Pro users (custom location support)
L0068 
L0069 [[site_name]] - Site title, former <h1>
L0070 [[site_subtitle]] - Site subtitle, former <h2>
L0071 [[content]] - It's rather obvious, content of the page
L0072 [[search_box]] - Box for searching within a site
L0073 [[site_locked]] - Information about a lock on the site
L0074 [[page_title]] - Page title
L0075 [[breadcrumbs]] - Breadcrumbs elements
L0076 [[tags]] - Displays list of tags
L0077 [[topbar]] - Top navigation
L0078 [[sidebar]] - Side navigation, displayed if enabled
L0079 [[ssl_warning]] - Warning about disabled SSL if Pro+ subscription expires
L0080 [[page_not_exists]] - Information displayed when page does not exist
L0081 [[license_text]] - License text (set up in Admin Panel)
L0082 [[footer]] - Inserts footer, default or custom
L0083 [[/code]]
L0084 
L0085 +++ Possible if statement in layouts
L0086 [[code]]
L0087 [[if name]]
L0088 if code ...
L0089 [[/if]]
L0090 
L0091 [[if !name]]
L0092 if code ...
L0093 [[/if]]
L0094 
L0095 [[if name]]
L0096 if code ...
L0097 [[else]]
L0098 else code ...
L0099 [[/if]]
L0100 [[/code]]
L0101 
L0102 List of available if statements:
L0103 [[code]]
L0104 [[if site_subtitle]]
L0105 [[if site_locked]]
L0106 [[if page_title]]
L0107 [[if breadcrumbs]]
L0108 [[if tags]]
L0109 [[if topbar]]
L0110 [[if sidebar]]
L0111 [[if ssl_warning]]
L0112 [[if page_exists]]
L0113 [[if license_text]]
L0114 [[if custom_footer]]
L0115 [[/code]]
L0116 
```
