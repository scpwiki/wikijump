# ListPages Module

- Feature ID: `module-listpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ListPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:listpages-module-prev/source.wikidot.txt:1` through line 243 (legacy)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:note-template-in-modules/source.wikidot.txt:1` through line 5 (included)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:page-selection/source.wikidot.txt:1` through line 122 (included)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listpages-module/source.wikidot.txt:1` through line 403 (canonical)

## Documentation-derived behavioral evidence

### doc-include:listpages-module-prev (legacy)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:listpages-module-prev/source.wikidot.txt:1` through line 243  
SHA-256 of complete source file: `4bf93e034e0c771ccfde586a80bbf6eb12332c90d42b697196fad004fefcbd7a`

```wikidot
L0001 The ListPages module allows one to select pages based on various criteria and list them in a custom way.  ListPages is a very general-purpose module for listing pages in a way similar to the [http://www.wikidot.com/doc:frontforum-module FrontForum module]. The module allows custom content formatting, ordering, pagination, support for tags and RSS feed generation.
L0002 
L0003 As shown in the examples it can be used to create blogs, news systems etc. but also quick lists of recently edited or created pages. Thanks to the very flexible tag support one can even use it for content classification and online catalogs.
L0004 
L0005 A [http://blog.wikidot.com/design:4 new syntax for ListPages] is in beta-testing.
L0006 
L0007 ++ Attributes
L0008 
L0009 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0010 || category || no || comma- or space-separated names || current category || names of the categories that pages would be fetched from, use {{category="*"}} to fetch pages from all categories; by default pages from the current category are selected (i.e. the category that the page containing the module belongs to); also aliased as {{categories}} ||
L0011 || tags || no || comma- or space-separated tag names with {{+}} and {{-}} modifiers, _
L0012 or {{@URL}} || none || lists tags that are used as a criteria to select pages, the "+" before the tag makes it required, "-" means "without a tag" and tags without modifiers translate to "pages that have any of those tags"; _
L0013 a special tag "=" adds all the tags that are present in the current page ||
L0014 || tagTarget || no || name of a landing page for clickable tags || none || if this parameter is set to a name of a wiki page, all tags (generated with %%tags%% will be clickable and will lead to this page ||
L0015 || date || no || {{year}} or {{year}}.{{month}} _
L0016 (numbers) _
L0017 or {{last //n// //unit//}} where //n// - number (1 if skipped), //unit// - day(s), week(s), month(s) _
L0018 or {{@URL}} || none || adds a date criteria to selection, valid values are e.g. 2007 (to select only pages created in 2007), 2008.05 (to select only pages from May 2008) ||
L0019 || parent || no || //category:pagename// || none || restricts results to children of specified page. ||
L0020 || range || no || {{before, after, others}} || none || "before" means pages up to but not including current (in order), "after" means pages after current page (in order), "others" means pages except current page. ||
L0021 || skipCurrent || no || {{true, false, yes, no}} or {{@URL}}|| {{no}} || skips the current page from the list ||
L0022 || perPage || no || number or {{@URL}} || 20 || number of items (pages) to display on one screen (when paginating) ||
L0023 || limit || no || number or {{@URL}} || none || limits the total number of selected pages ||
L0024 || order || no || {{dateCreatedDesc}} _
L0025 {{dateCreatedAsc}} _
L0026 {{dateEditedDesc}} _
L0027 {{dateEditedAsc}} _
L0028 {{titleDesc}} _
L0029 {{titleAsc}} _
L0030 {{ratingDesc}} _
L0031 {{ratingAsc}} _
L0032 {{pageLengthAsc}} _
L0033 {{pageLengthDesc}} _
L0034 {{random}} _
L0035 or {{@URL}} || {{dateCreatedDesc}} || selects ordering of the pages, the default one is: newest pages first; {{random}} gives you pages in random order, but the result is cached for 60s ||
L0036 || separate || no || {{true}}, {{false}}, {{yes}}, {{no}} || {{true}} || if {{true}} page items are placed in individual containers, but if {{false}} -- they are rendered without any breaks or splits, which allows to create e.g. simple lists containing titles of recently edited pages (see below) ||
L0037 || prependLine || no || wiki-formatted string || || if {{separate="false"}}, this line of wiki text will be prepended to the processed list of pages; one can use it e.g. to generate table headers ||
L0038 || appendLine || no || wiki-formatted string || || if {{separate="false"}}, this line of wiki text will be appended to the processed list of pages ||
L0039 || urlAttrPrefix || no || any alphanumeric || none || prefix for the parameters passed via the URL; handy when more ListPages modules are on the page ||
L0040 || rss || no || any text || none || title for the RSS feed; if not given, the corresponding RSS feed will not be linked; also aliased as rssTitle for compatibility ||
L0041 || rssHome || no || any URL || the main wiki page || feed source page -- you can use it with %%linked_channel_title%% variable in [[[doc:Feed module]]] ||
L0042 || rssDescription || no || any text || description of the site || feed description ||
L0043 
L0044 +++ @URL
L0045 
L0046 Some of the attributes accept a special {{@URL}} value which tells the module to read the value of the parameter from the URL. This is an advanced use but is handy in many situations.
L0047 
L0048 Parameters and their values in URL are delimited by slashes (/). So if there is a ListPages module on a page "blog" at a given wiki and you want to read date from the URL, you need to put
L0049 
L0050 [[code]]
L0051 [[module ListPages date="@URL"]]
L0052 [[/code]]
L0053 
L0054 and the module will read the {{date}} parameter from the properly-constructed URL, e.g.
L0055 [[code]]
L0056 http://www.wikidot.com/blog/date/2008.07
L0057 [[/code]]
L0058 
L0059 Several parameters can be combined in the URL like this:
L0060 
L0061 [[code]]
L0062 http://www.wikidot.com/blog/date/2008.07/order/ratingDesc/limit/3
L0063 [[/code]]
L0064 
L0065 The URLs need to be created manually at this point.
L0066 
L0067 +++ More than one module in the page
L0068 
L0069 Since some of the parameters can be passed in the URL of the request there might be conflict when more than one ListPages module is present in the page. One most likely conflict can occur when both modules use pagination -- clicking "next" on one of them would also affect the other.
L0070 
L0071 To prevent such conflicts the {{urlAttrPrefix}} parameter can be used. It simply prepends a text (unique for each of the modules) to the parameter names in the URL. So the .../date/2008.7 would become .../prefix_date/2008.07. If you can set unique prefixes for each of the ListPages instances you would avoid any conflicts.
L0072 
L0073 A very simple example follows:
L0074 
L0075 [[code]]
L0076 [[module ListPages perPage="5" limit="15" urlAttrPrefix="prefix1"]]
L0077 %%title%%
L0078 [[/module]]
L0079 [[module ListPages perPage="5" limit="15" urlAttrPrefix="prefix2"]]
L0080 %%title%%
L0081 [[/module]]
L0082 [[/code]]
L0083 
L0084 [[table]]
L0085 [[row]]
L0086 [[column]]
L0087 [[div style="padding-right: 5em"]]
L0088 [[module ListPages perPage="5" limit="15" urlAttrPrefix="prefix1"]]
L0089 %%title%%
L0090 [[/module]]
L0091 [[/div]]
L0092 [[/column]]
L0093 [[column]]
L0094 [[module ListPages perPage="5" limit="15" urlAttrPrefix="prefix2"]]
L0095 %%title%%
L0096 [[/module]]
L0097 [[/column]]
L0098 [[/row]]
L0099 [[/table]]
L0100 
L0101 ++ Item format
L0102 
L0103 A custom format for displaying news items can be chosen.  To specify a custom format one should use module invocation:
L0104 
L0105 [[code]]
L0106 [[module ListPages category="blog"]]
L0107 <custom format>
L0108 [[/module]]
L0109 [[/code]]
L0110 
L0111 where the inner {{<custom format>}} element is any block of text following the wiki-syntax, where special variables can be used:
L0112 
L0113 ||~ variable ||~ aliases ||~ description||
L0114 || {{%%title%%}} || || title of the page ||
L0115 || {{%%linked_title%%}} || {{%%title_linked%%}} || title of the page linked to the page itself||
L0116 || {{%%page_unix_name%%}} || {{%%full_page_name%%}} || //unix name// of the page -- the one that is displayed in the URL of a page ||
L0117 || {{%%page_name%%}} || || name of the page without the category ||
L0118 || {{%%category%%}} || || name of the category of the page ||
L0119 || {{%%link%%}} || || URL pointing to the page ||
L0120 || {{%%author%%}} || || prints author that created page ||
L0121 || {{%%author_edited%%}} || %%user_edited%% || prints author that recently edited the page ||
L0122 || {{%%date%%}} || || prints the date the page was created ||
L0123 || {{%%date|//format//%%}} || || prints date with a custom format. Most tokens from php's [http://php.net/manual/en/function.strftime.php strftime] are accepted. You may find [http://community.wikidot.com/howto:frontforum-date-variable the howto] contributed by community useful. ||
L0124 || {{%%date_edited%%}} || || prints the date the page was recently edited ||
L0125 || {{%%date_edited|//format//%%}} || || same as above, with custom formatting ||
L0126 || {{%%description%%}} || {{%%short%%}}, {{%%summary%%}} || short summary of page, equivalent to %%content{1}%% if there is a separator ({{====}}) within the page, otherwise a short version will be automatically generated (e.g. by using the first paragraph) ||
L0127 || {{%%first_paragraph%%}} || || displays the first paragraph of a page. ||
L0128 || {{%%content{n}%%}} || || selects and displays the n-th content segment if the content is split using the {{====}} separator ||
L0129 || {{%%content%%}} || {{%%text%%}}, {{%%long%%}}, {{%%body%%}} || full content of the page ||
L0130 || {{%%preview%%}} || || first 200 characters of the post ||
L0131 || {{%%preview(n)%%}} || || first //n// characters of the post (//n// -- any positive integer number) ||
L0132 || {{%%tags%%}} || || displays space-separated list of tags for a given page ||
L0133 || {{%%rating%%}} || || displays a number -- rating of the page ||
L0134 || {{%%comments%%}} || || displays number of comments to the page ||
L0135 
L0136 The default format is:
L0137 [[code]]
L0138 + %%linked_title%%
L0139 
L0140 by %%author%% %%date|%O ago (%e %b %Y, %H:%M %Z)%%
L0141 
L0142 %%short%%
L0143 [[/code]]
L0144 
L0145 If {{separate}} is set to {{true}} (default), each page item is embedded in the HTML {{<div class="list-pages-item">...</div>}} element.
L0146 
L0147 ++ Examples
L0148 
L0149 +++ Blog-like front page
L0150 
L0151 To make a front page for a blog structure, i.e. make a list of pages from the category {{blog}} ordered by "most recent first" and show only a short version of their content (i.e. first paragraph or first section if the {{====}} separator is used) one can do it with the code:
L0152 
L0153 [[code]]
L0154 [[module ListPages category="blog" rss="My Blog Posts"]]
L0155 [[/code]]
L0156 
L0157 The default format might be just enough for it, but one can easily create a custom format using the formatting tags above:
L0158 
L0159 [[code]]
L0160 [[module ListPages category="blog" rss="My Blog Posts" tags="technology news +apple -funny"]]
L0161 +++ %%linked_title%%
L0162 
L0163 by %%author%% %%date%%
L0164 
L0165 %%content%%
L0166 
L0167 tags: %%tags%%
L0168 [[/module]]
L0169 [[/code]]
L0170 
L0171 In both examples we are pointing to an RSS feed with recent pages from the blog: category and we are setting a title for this blog feed.
L0172 
L0173 The tag string (i.e. tags="technology news +apple -funny") means:
L0174 * select pages that have any of tags //technology// or //news//
L0175 * AND pages must have the "apple" tag
L0176 * AND pages must not have the "funny" tag applied
L0177 
L0178 There is also a **special tag: {{=}}** (equal sign). It adds all the tags that are present in the current page, without +/-. So if a current page has tags: blog wikidot, the tags="=" is equivalent to tags="blog wikidot".
L0179 
L0180 It can be used to create a list of similar pages. You can combine the {{=}} tag with other tags. If {{=}} is the only listed tag (i.e. explicitly {{tags="="}}) it implies {{skipCurrent="yes"}} so that you can use simply:
L0181 
L0182 [[code]]
L0183 + Similar pages
L0184 
L0185 [[module ListPages tags="="]]
L0186 [[/code]]
L0187 
L0188 +++ Short list of recently edited pages
L0189 
L0190 [[code]]
L0191 [[module ListPages  category="*" limit="10" separate="false" order="dateEditedDesc"]]
L0192 * %%linked_title%% _
L0193 %%date_edited|%O ago%%
L0194 [[/module]]
L0195 [[/code]]
L0196 
L0197 This piece of code selects pages from all categories, number of pages is limited to 10, it switches off the //separate// so that the list can be smoothly processed (if {{separate="true"}}, each page item would create a separate, one-element list), we choose not to create an RSS feed for the selection and the pages are ordered by the date of last edit (most recent first).
L0198 
L0199 The custom format should look familiar, and here is the result:
L0200 
L0201 [[module ListPages  category="*" limit="10" separate="false" order="dateEditedDesc"]]
L0202 * %%linked_title%% _
L0203 %%date_edited|%O ago%%
L0204 [[/module]]
L0205 
L0206 Advanced processing is possible thanks to the {{prependLine}} and {{appendLine}} parameters. When combined with {{separate="false"}}, they allow creating a custom wiki-formatted block from the page elements. Look at an example:
L0207 
L0208 [[code]]
L0209 [[module ListPages separate="false" prependLine="||~ Page||~ Date created||~ Created by ||" limit="5"]]
L0210 || %%linked_title%% || %%date%% || %%author%% ||
L0211 [[/module]]
L0212 [[/code]]
L0213 
L0214 [[module ListPages separate="false" prependLine="||~ Page||~ Date created||~ Created by ||" limit="5"]]
L0215 || %%linked_title%% || %%date%% || %%author%% ||
L0216 [[/module]]
L0217 
L0218 +++ Random page(s)
L0219 
L0220 Using order="random" one can easily pull a random page from a wiki, e.g.
L0221 
L0222 ++++ Random pages
L0223 [[code]]
L0224 [[module ListPages  category="doc" limit="5" separate="false" order="random"]]
L0225 * %%linked_title%%
L0226 [[/module]]
L0227 [[/code]]
L0228 
L0229 [[module ListPages  category="doc" limit="5" separate="false" order="random"]]
L0230 * %%linked_title%%
L0231 [[/module]]
L0232 
L0233 ++++ Single random page
L0234 [[code]]
L0235 [[module ListPages  category="doc" limit="1" order="random"]]
L0236 %%linked_title%%
L0237 [[/module]
L0238 [[/code]]
L0239 [[module ListPages  category="doc" limit="1" order="random"]]
L0240 %%linked_title%%
L0241 [[/module]]
L0242 
L0243 Result of random listing is cached for 1 minute, so if you are reloading a page every few seconds the random choice will not change. After a minute however a new result is picked up.
```

### doc-include:note-template-in-modules (included)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:note-template-in-modules/source.wikidot.txt:1` through line 5  
SHA-256 of complete source file: `9bc49111232c35a1f5dcb2757737a7a23caeb0766a7073f622fbe4cacf4d6097`

```wikidot
L0001 [!--
L0002 This note applies to modules that have some body (template). The template cannot contain any tags that are parsed before module rule in wiki syntax. Includes ListPages and ListUsers at least.
L0003 --]
L0004 
L0005 Module body cannot contain @@[[code]]@@ or @@[[html]]@@. In case it contains those tags, module will not work at all.
```

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

### doc-modules:listpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listpages-module/source.wikidot.txt:1` through line 403  
SHA-256 of complete source file: `d5c5aac7290c3b8108378d93ad32e444e682f58c3f21eedb4871a2a66a224e02`

```wikidot
L0001 [[f>toc]]
L0002 
L0003 The ListPages module is a general-purpose and widely-used tool that selects and display pages within a site.
L0004 
L0005 To use ListPages, you decide some or all of:
L0006 
L0007 * what pages to select (from the site, from categories, by parent, by tags, by date, etc.)
L0008 * how to order the pages (ascending, descending)
L0009 * how to break the output into blocks (pagination)
L0010 * how to display the results as Wikidot text (templating in module body)
L0011 * how to export the results as an RSS feed
L0012 
L0013 The general syntax for ListPages is:
L0014 
L0015 [[code]]
L0016 [[module ListPages arguments...]]
L0017 module body
L0018 [[/module]]
L0019 [[/code]]
L0020 
L0021 By default, ListPages will show all visible pages in the current category, from newest to oldest.
L0022 
L0023 [[include doc-include:note-template-in-modules]]
L0024 
L0025 ++ Example
L0026 
L0027 This example lists the pages in the current category, along with details of who created the page, and when:
L0028 
L0029 [[code]]
L0030 [[module ListPages separate="no" limit="5"]]
L0031 %%title_linked%% - [[user %%created_by%%]] - %%created_at%%
L0032 [[/module]]
L0033 [[/code]]
L0034 
L0035 In action:
L0036 
L0037 [[module ListPages separate="no" limit="5"]]
L0038 %%title_linked%% - [[user %%created_by%%]] - %%created_at%%
L0039 [[/module]]
L0040 
L0041 ++ Naming conventions
L0042 
L0043 Older argument names are {{inMixedCase}}.  Newer argument names are {{in_lower_case}}, and this style will be used more systematically in Wikidot.  Dates are always {{//something//_at}} and user names are always {{//someone//_by}}.  Linked fields are always {{//somefield//_linked}}.
L0044 
L0045 ++ Selecting pages
L0046 [[include doc-include:page-selection]]
L0047 
L0048 ++ Ordering pages
L0049 
L0050 To order the pages, use:
L0051 
L0052 ||~ Argument    ||~ Meaning ||
L0053 || order        || Specify order criteria ||
L0054 
L0055 Order criteria:
L0056 
L0057 * "//property//" means "ascending by this property"
L0058 * optionally followed by " desc" meaning "descending"
L0059 * optionally followed by " desc desc" meaning "ascending", which makes "desc" safe to add after any sort order
L0060 * default is "created_at desc"
L0061 
L0062 ||~ Property    ||~ Meaning ||
L0063 || name         || Order by page name ||
L0064 || fullname     || Order by category and page name ||
L0065 || title        || Order by page title ||
L0066 || created_by   || Order by author screen name ||
L0067 || created_at   || Order by date created ||
L0068 || updated_at   || Order by date updated ||
L0069 || size         || Order by number of characters in page ||
L0070 || rating       || Order by rating ||
L0071 || votes       || Order by number of votes ||
L0072 || revisions    || Order by number of revisions ||
L0073 || comments     || Order by number of comments ||
L0074 || random       || Order randomly, cached for 60 seconds ||
L0075 || _data-form-field-name || Order by a field in a data form ||
L0076 
L0077 For example to order by rating in descending order:
L0078 
L0079 [[code]]
L0080 order="rating desc"
L0081 [[/code]]
L0082 
L0083 Caution: When listing pages from many categories, where some categories have rating type set to + or +/- and others to "stars" (in Site Manager), selecting and ordering by rating may not funtion properly. The solution is to list and order pages from categories having the same rating mode.
L0084 
L0085 This example shows how to order pages using a data form field. Note that you must prefix the data form field name with an underscore. This lists all pages from the //dictionary// category and sorts them by the data form's //mainword// field (a wiki field type in this example). The body of the module then lists the contents of the //mainword// field and creates a link to the page.
L0086 [[code]]
L0087 [!--
L0088 Note: Use %%form_raw{fieldname}%% for wiki field types,
L0089          %%form_data{fieldname}%% for other field types
L0090 --]
L0091 [[module ListPages category="dictionary" order="_mainword"]]
L0092 %%form_raw{mainword}%%@<&nbsp;>@@<&nbsp;>@([/%%fullname%% see dictionary entry])
L0093 [[/module]]
L0094 [[/code]]
L0095 
L0096 Note that "//property// asc" is not allowed and unknown order criteria give you the default order, which is "created_at desc".
L0097 
L0098 **Type casting** for [[[doc:data-forms|]]] fields:
L0099 Default order method is sort by text. You can enforce numerical sorting.
L0100 
L0101 [[code]]
L0102 [[module ListPages category="band" order="_albums::integer desc"]]
L0103 ...
L0104 [[/module]]
L0105 [[/code]]
L0106 
L0107 ++ Pagination
L0108 
L0109 To control how many items (wiki pages) will be shown in total, and how these are paginated (confusingly, also into 'pages'), use any of:
L0110 
L0111 ||~ Argument    ||~ Meaning ||
L0112 || limit        || Limit total items ||
L0113 || perPage      || Limit per pagination ||
L0114 || reverse      || Show pages in reversed order ||
L0115 
L0116 Total limit:
L0117 
L0118 * "number" - means limit the total number of selected pages.
L0119 * by default all pages matching criteria are listed
L0120 
L0121 Pagination limit:
L0122 
L0123 * "number" - means limit the number of page items shown on per pagination.
L0124 * default is 20, maximum is 250.
L0125 
L0126 Reversed display:
L0127 
L0128 * "yes" - means show wiki pages from last to first on given page.
L0129 
L0130 ++ Module body
L0131 
L0132 The body of the module allows you to specify how page properties and content is formatted.  To control this formatting, you can use these module arguments:
L0133 
L0134 ||~ Argument    ||~ Meaning ||
L0135 || separate     || Separation specifier ||
L0136 || wrapper     || Wrapper specifier ||
L0137 || prependLine  || Header specifier ||
L0138 || appendLine   || Footer specifier ||
L0139 
L0140 Separation specifier:
L0141 
L0142 * "yes" means place each page item into a separate container (divs).
L0143 * "no" means put all items into one container, so they can become a single list, for example.
L0144 * default is "yes".
L0145 
L0146 With {{separate}} set to true, each of the page is compiled (converted from wiki source to HTML) separately. While it is false, wiki compiler is invoked only once on a combined source from all selected pages.
L0147 
L0148 As a result, some page-specific variables and constructs such as {{iftags}} can generate different results. {{iftags}}, with {{separate="yes"}}, will be aware of tags of individual pages, while with {{separate="no"}} it will read tags of the main page that holds the ListPages module.
L0149 Also @@[[image :first ...]]@@ will only work with {{separate="yes"}}.
L0150 
L0151 Wrapper specifier:
L0152 
L0153 * "yes" means place all items into container (div).
L0154 * "no" means do not place all items into container (div).
L0155 * default is "yes".
L0156 
L0157 Header specifier:
L0158 
L0159 * "text" means output this text at the start of the list of pages, //only// if the separation specifier is false.
L0160 
L0161 Footer specifier:
L0162 
L0163 * "text" means output this text at the end of the list of pages, //only// if the separation specifier is false.
L0164 
L0165 +++ Sections head/body/foot
L0166 Additionally you can use **[[head]]**, **[[body]]**, **[[foot]]** sections, which simply replaces prependLine and appendLine. It allows you to create more complex header and footer for ListPages. This is particularly usable with complex table and list creation.
L0167 
L0168 Example of sections usage
L0169 [[code]]
L0170 [[module ListPages category="carousel" wrapper="no" separate="no" _active="yes"]]
L0171   [[head]]
L0172     [[ul id="u-myList" class="..."]]
L0173   [[/head]]
L0174 
L0175   [[body]]
L0176     [[li class="list-item"]]%%title_linked%% by (%%created_by%%)[[/li]]
L0177   [[/body]]
L0178 
L0179   [[foot]]
L0180     [[/ul]]
L0181   [[/foot]]
L0182 [[/module]]
L0183 [[/code]]
L0184 (also works for custom domains)
L0185 
L0186 The template consists of wiki text mixed with variables specified as {{%%variable-name%%}}.  You can use these page properties:
L0187 
L0188 ||~ Property ||~ Meaning ||
L0189 ||~ Page lifecycle ||~ ||
L0190 || %%created_at%% || Date page was created ||
L0191 || %%created_by%% || User who created page ||
L0192 || %%created_by_unix%% || "Unixified" name of user who created page -- to be used for constructing URLs ||
L0193 || %%created_by_id%% || "ID" number of user who created page -- to be used for constructing URLs ||
L0194 || %%created_by_linked%% || Icon and link to user who created page ||
L0195 || %%updated_at%% || Date page was updated (edited, tagged, parented) ||
L0196 || %%updated_by%% || User who updated page ||
L0197 || %%updated_by_unix%% || "Unixified" name of user who updated page -- to be used for constructing URLs ||
L0198 || %%updated_by_id%% || "ID" number of user who updated page -- to be used for constructing URLs ||
L0199 || %%updated_by_linked%% || Icon and link to user who updated page ||
L0200 || %%commented_at%% || Date of last comment ||
L0201 || %%commented_by%% || User who made last comment ||
L0202 || %%commented_by_unix%% || "Unixified" name of user who made last comment -- to be used for constructing URLs ||
L0203 || %%commented_by_id%% || "ID" number of user who made last comment -- to be used for constructing URLs ||
L0204 || %%commented_by_linked%% || Icon and link to user who made last comment ||
L0205 ||~ Page structure ||~ ||
L0206 || %%name%% || Page name without category ||
L0207 || %%category%% || Page category if any ||
L0208 || %%fullname%% || Page name with category if any ||
L0209 || %%title%% || Page title ||
L0210 || %%title_linked%% || Link to page showing title as text (works also for custom domain)||
L0211 || %%parent_name%% || Parent page name without category ||
L0212 || %%parent_category%% || Parent page category if any ||
L0213 || %%parent_fullname%% || Parent page name with category if any ||
L0214 || %%parent_title%% || Parent page title ||
L0215 || %%parent_title_linked%% || Link to Parent page showing title as text ||
L0216 || %%link%% || URL pointing to page (not working for custom domains!)||
L0217 || %%content%% || Page content ||
L0218 || %%content{n}%% || Numbered content section ||
L0219 || %%preview%% || First 200 characters of the page ||
L0220 || %%preview(n)%% || First //n// characters of the page ||
L0221 || %%summary%% || Summary of content ||
L0222 || %%first_paragraph%% || The first paragraph of the page ||
L0223 || %%tags%% || Page visible tags (not starting with underscore) ||
L0224 || %%tags_linked%% || Page visible tags linked to system:page-tags/tag/{tag} ||
L0225 || %%tags_linked|link_prefix%% || Page visible tags linked to link_prefix{tag} ||
L0226 || %%_tags%% || Page hidden tags (starting with underscore) ||
L0227 || %%_tags_linked%% || Page hidden tags linked to system:page-tags/tag/{tag} ||
L0228 || %%_tags_linked|link_prefix%% || Page hidden tags linked to link_prefix{tag} ||
L0229 || %%form_data{name}%% || Field value from page [/doc:data-forms data form] if any ||
L0230 || %%form_raw{name}%% || For select fields, the internal value saved in the page form data, if any ||
L0231 || %%form_label{name}%% || The label of the field as defined in the [/doc:data-forms data form] if any ||
L0232 || %%form_hint{name}%% || The hint of the field as defined in the [/doc:data-forms data form] if any ||
L0233 ||~ Page reporting ||~ ||
L0234 || %%children%% || Number of child pages ||
L0235 || %%comments%% || Number of comments on page ||
L0236 || %%size%% || Number of characters in page ||
L0237 || %%rating%% || Page rating value (number or stars depending on Rating settings in Site Manager ||
L0238 || %%rating_votes%% || Number of votes ||
L0239 || %%rating_percent%% || Percent value of 5-star rating only ||
L0240 || %%revisions%% || Number of revisions to page ||
L0241 || %%index%% || Page index in ListPages output + offset (1 to %%total%%) ||
L0242 || %%total%% || Total number of pages ignoring limit (may be higher than %%limit%%) || 
L0243 || %%limit%% || Limit passed to ListPages (empty if not passed) ||
L0244 || %%total_or_limit%% || Total number of pages in ListPages output (highest %%index%%). _
L0245 If limit is passed to the module, %%total_or_limit%% is %%total%% or %%limit%% whichever is smaller ||
L0246 ||~ Current context ||~ ||
L0247 || %%site_title%% || Title of current site ||
L0248 || %%site_name%% || Wikidot Unix name for site ||
L0249 || %%site_domain%% || Active domain name of current site ||
L0250 
L0251 Date formatting:
L0252 
L0253 * All _at fields are dates and allow a custom format via the {{|//format//}} specifier.
L0254 
L0255 Most tokens from PHP's [http://php.net/manual/en/function.strftime.php strftime] are accepted. You may find [http://community.wikidot.com/howto:frontforum-date-variable the howto] contributed by community useful.
L0256 
L0257 [[note]]
L0258 Editor's note: this section needs expanding with the most useful formatting options.
L0259 [[/note]]
L0260 
L0261 Tag linking:
L0262 
L0263 * If no link_prefix is specified, tags link to system:page-tags/tag/name-of-tag
L0264 * If link_prefix is specified, tags link to ##blue|link_prefix##name-of-tag (colors irrelevant)
L0265 * if link_prefix is empty but the pipe is present, %%tags_linked|%% generates links to pages with names corresponding to tags
L0266 * Examples
L0267 
L0268 ||~ if syntax is: ||~ "shiny" tag will link to: ||
L0269 || %%tags_linked%% || /system:page-tags/tag/shiny ||
L0270 || %%tags_linked|system:page-tags/tag/%% || /system:page-tags/tag/shiny ||
L0271 || %%tags_linked|interesting-list/category/%% || /interesting-list/category/shiny ||
L0272 || %%tags_linked|player:%% || /player:shiny ||
L0273 || %%tags_linked|very_%% || /very_shiny ||
L0274 || %%tags_linked|@@http://myothersite.wikidot.com/@@see-also/tag/%% || @@http://myothersite.wikidot.com@@/see-also/tag/shiny ||
L0275 || %%tags_linked|%% || /shiny ||
L0276 
L0277 ++ Advanced Use
L0278 
L0279 This section describes additional functionality that will be useful for advanced users.
L0280 
L0281 +++ RSS feeds
L0282 
L0283 You can export any ListPages result as an RSS feed.  Use these arguments to control the RSS feed generation:
L0284 
L0285 ||~ Argument    ||~ Meaning ||
L0286 || rss          || feed title ||
L0287 || rssDescription || feed description ||
L0288 || rssHome      || feed homepage ||
L0289 || rssLimit      || feed limit ||
L0290 || rssOnly      || only show feed link ||
L0291 
L0292 Feed title:
L0293 
L0294 * "text" means use this text for the RSS feed title.
L0295 * Default is to not generate any RSS feed.
L0296 
L0297 Feed description:
L0298 
L0299 * "text" means use this text for the RSS feed description.
L0300 
L0301 Feed homepage:
L0302 
L0303 * "//pagename//" means tell RSS clients this is the home page for the feed.
L0304 * Default is {{@@http://your-site.wikidot.com@@}}
L0305 * Setting value to "blog:_start" actually means {{@@http://your-site.wikidot.com/blog:_start@@}}
L0306 
L0307 Feed limit:
L0308 
L0309 * sets limit for RSS feed, and can be different to the ListPages limit
L0310 * Default RSS limit inherits lower value from limit and perPage arguments
L0311 
L0312 Feed only:
L0313 
L0314 * "true" or "yes" displays the RSS feed link without showing ListPages results
L0315 
L0316 **Important**: RSS feed ignores "{{created_at}}" selector.
L0317 
L0318 +++ Passing arguments via URL
L0319 
L0320 ListPages lets you create variations of a single list using specially constructed links, consisting of the page URL (link) followed by arguments and values.  These are mainly useful to invoke new selectors, and change the ordering or display.
L0321 
L0322 You can pass any arguments in the URL by specifying {{argument="@URL|default-value"}} as the argument value and then appending "/{{name}}/{{value}}" to the URL used to invoke the page.  If the URL does not contain a value for the argument, the default is used.  Arguments that do not have @URL in their value cannot be set via the URL.  The default value is optional: if you use only {{argument="@URL"}} and do not provide a value on the URL, then the argument behaves as if it was not set.
L0323 
L0324 The two main ways of using arguments-by-URL are (a) to create links to a page explicitly, on another page and (b) to generate links within the ListPages itself, so it will reshow itself with different configurations.  Here is a simple example:
L0325 
L0326 [[code]]
L0327 [[module ListPages category="@URL|design"]]
L0328 %%name%% in category %%category%%
L0329 [[/module]]
L0330 [[/code]]
L0331 
L0332 Another example shows how to select blog entries by created_at date:
L0333 
L0334 [[code]]
L0335 [[module ListPages category="blog" created_at="@URL"]]
L0336 [[/code]]
L0337 
L0338 and the module will read the {{created_at}} argument from the properly-constructed URL, e.g.
L0339 [[code]]
L0340 http://www.wikidot.com/blog/created_at/2008.07
L0341 [[/code]]
L0342 
L0343 You can specify multiple arguments like this:
L0344 
L0345 [[code]]
L0346 http://www.wikidot.com/blog/created_at/2008.07/order/rating desc/limit/3
L0347 [[/code]]
L0348 
L0349 To pass tags with (+/-) "+" need to be encoded with "%2b"
L0350 {{+apple,-banana}}
L0351 [[code]]
L0352 http://www.wikidot.com/blog/tags/%2bapple,-banana
L0353 [[/code]]
L0354 
L0355 You can create the URLs manually or within ListPages itself.  Some modules also generate compatible URLs.
L0356 
L0357 [[note]]
L0358 Editor's note: list of modules that produce compatible URLs should be documented here.
L0359 [[/note]]
L0360 
L0361 +++ More than one module in the page
L0362 
L0363 Since some of the arguments can be passed in the URL of the request there might be conflict when more than one ListPages module is present in the page. One most likely conflict can occur when both modules use pagination -- clicking "next" on one of them would also affect the other.
L0364 
L0365 To prevent such conflicts use the {{urlAttrPrefix}} argument. This prepends a text (unique for each of the modules) to the argument names in the URL. So the .../created_at/2008.7 would become .../prefix_created_at/2008.07. If you can set unique prefixes for each of the ListPages instances you would avoid any conflicts.
L0366 
L0367 ++ Deprecated functionality
L0368 
L0369 These arguments and variables can still be used but are disrecommended.  You should when possible use the modern replacements.  At some future date, deprecated functionality may be removed.
L0370 
L0371 ||~ Instead of this               ||~ Use this ||
L0372 || skipCurrent="yes"            || range="others" ||
L0373 || categories=                     || category= ||
L0374 || tag=                                || tags= ||
L0375 || tagTarget="pagename"     || %%tags_linked|/pagename/tag/%% ||
L0376 || date=                              || created_at= ||
L0377 || order="dateCreatedAsc"   || order="created_at" ||
L0378 || order="dateCreatedDesc"  || order="created_at desc" ||
L0379 || order="dateEditedAsc"    || order="updated_at" ||
L0380 || order="dateEditedDesc"   || order="updated_at desc" ||
L0381 || order="titleAsc"         || order="title" ||
L0382 || order="titleDesc"        || order="title desc" ||
L0383 || order="ratingAsc"        || order="rating" ||
L0384 || order="ratingDesc"       || order="rating desc" ||
L0385 || order="pageLengthAsc"    || order="size" ||
L0386 || order="pageLengthDesc"   || order="size desc" ||
L0387 || rssTitle=                || rss= ||
L0388 || %%linked_title%%         || %%title_linked%% ||
L0389 || %%page_unix_name%%       || %%fullname%% ||
L0390 || %%full_page_name%%       || %%fullname%% ||
L0391 || %%page_name%%            || %%name%% ||
L0392 || %%author%%               || %%created_by%% ||
L0393 || %%author_edited%%        || %%updated_by%% ||
L0394 || %%user_edited            || %%updated_by%% ||
L0395 || %%date%%                 || %%created_at%% ||
L0396 || %%date_edited%%          || %%updated_at%% ||
L0397 || %%description%%          || %%summary%% ||
L0398 || %%short%%                || %%summary%% ||
L0399 || %%text%%                 || %%content%% ||
L0400 || %%long%%                 || %%content%% ||
L0401 || %%body%%                 || %%content%% ||
L0402 
L0403 ListPages supports a 'default format', where you do not specify any module body and no {{[[/module]]}}.  This functionality is deprecated and you should avoid using it.
```
