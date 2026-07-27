# Feed Module

- Feature ID: `module-feed`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Feed` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:feed-module/source.wikidot.txt:1` through line 146 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:feed-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:feed-module/source.wikidot.txt:1` through line 146  
SHA-256 of complete source file: `d01594a9ee5d0a8ef38d81bdb1d452a91c885ba5f4dc908192e2751d1b89324f`

```wikidot
L0001 ++ Description
L0002 
L0003 The //Feed// module can import RSS or Atom feeds from (almost) any web location and display them in a customizable way. It can also combine feeds from multiple sources.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || src || yes || single URL address or semicolon-separated list of URL addresses || none || points to the location of source RSS or Atom feed(s) || 
L0009 || limit || no || any reasonable number || none || limits the number of news items ||
L0010 || offset || no || any reasonable number || 0 || which item number to start from ||
L0011 
L0012 ++ Display format
L0013 
L0014 If no format is specified all the news items are displayed using the default format. To specify a custom format one should use module invocation:
L0015 
L0016 [[code]]
L0017 [[module Feed src="somesource"]]
L0018 <custom format>
L0019 [[/module]]
L0020 [[/code]]
L0021 
L0022 where the inner {{<custom format>}} element is any block of text following the wiki-syntax, where special variables can be used:
L0023 
L0024 ||~ variable ||~ aliases ||~ description||
L0025 || {{%%title%%}} || || title of the news item ||
L0026 || {{%%linked_title%%}} || {{%%title_linked%%}} || title of the news item linking to the original web page ||
L0027 || {{%%channel_title%%}} || || title of the news channel (useful when combining multiple feed sources)||
L0028 || {{%%linked_channel_title%%}} || {{%%channel_title_linked%%}} || title of the channel linking to the feed source page ||
L0029 || {{%%link%%}} || || URL address to the original news item ||
L0030 || {{%%description%%}} || {{%%short%%}}, {{%%summary%%}} || short summary of the item ||
L0031 || {{%%content%%}} || {{%%long%%}}, {{%%body%%}} || full content of the item (only when available; falls back to {{description}} if not)||
L0032 || {{%%date%%}} || || date of the item publication ||
L0033 || {{%%date|format%%}} || || prints date with a custom format. Most tokens from php's [*http://php.net/manual/en/function.strftime.php strftime] are accepted. You may find the [*http://community.wikidot.com/howto:frontforum-date-variable howto] contributed by community useful.||
L0034 || {{%%custom%%}} || || gives access to any field in the feed (see below)||
L0035 
L0036 The default format is: 
L0037 [[code]]
L0038 ++ %%linked_title%%
L0039 
L0040 %%date%%
L0041 
L0042 %%description%%
L0043 [[/code]]
L0044 
L0045 ++ How to use %%custom%%
L0046 
L0047 To access any field in the feed environment, use 
L0048 
L0049 [[code]]
L0050 %%custom <pointer>%%
L0051 [[/code]]
L0052 
L0053 where pointer is a path to the requested element. It is easier to learn this by example:
L0054 
L0055 Look at fragment of an item from [http://www.digg.com Digg]:
L0056 [[code type="xml"]]
L0057 <item>
L0058     <title>UFO gathering draws believers and belittlers</title>
L0059     <link>
L0060     	http://digg.com/space/UFO_gathering_draws_believers_and_belittlers
L0061     </link>
L0062     <description>
L0063     	The 37th annual Mutual UFO Network symposium is being held this weekend in Denver, 
L0064     	attracting throngs of believers and the downright curious — as well as upright skeptics 
L0065     	and debunkers.  
L0066     </description>
L0067     <pubDate>Sat, 15 Jul 2006 15:32:45 GMT</pubDate>
L0068     <guid isPermaLink="true">
L0069     	http://digg.com/space/UFO_gathering_draws_believers_and_belittlers
L0070     </guid>
L0071     <digg:diggCount>72</digg:diggCount>
L0072     <digg:submitter>
L0073     	<digg:username>capn_caveman</digg:username>
L0074     	<digg:userimage>http://digg.com/userimages/capn_caveman/medium.jpg</digg:userimage>
L0075     </digg:submitter>
L0076     <digg:category>Space</digg:category>
L0077     <digg:commentCount>13</digg:commentCount>
L0078 </item>
L0079 [[/code]]
L0080 
L0081 To access the {{<digg:diggCount>}} and display it use:
L0082 [[code]]
L0083 digg counts: %%custom digg:diggCount%%
L0084 [[/code]]
L0085 
L0086 To access a nested element {{<digg:username>}} use:
L0087 [[code]]
L0088 submitted by %%custom digg:submitter/digg:username%%
L0089 [[/code]]
L0090 
L0091 Now to access any element starting from the root feed element: look at the fragment of the digg feed code again:
L0092 [[code type="xml"]]
L0093 <rss version="2.0">
L0094     <channel>
L0095     	<title>digg</title>
L0096     	<language>en-us</language>
L0097     	<link>http://digg.com/</link>
L0098     	<description>digg</description>
L0099     	...
L0100 [[/code]]
L0101 To access the channel tittle element use:
L0102 [[code]]
L0103 %%custom feed/channel/title%%
L0104 [[/code]]
L0105 It is important to start with the {{feed}} word. Ater that the full path to the element follows.
L0106 
L0107 You can also use {{%%custom%%}} inside {{@@[[image ...]]@@}} and some other places. In some cases however a space character must be replaced with an underscore not to confuse the parser, e.g. to display user submitter image from the digg feed you can use:
L0108 
L0109 
L0110 [[code]]
L0111 [[image %%custom_digg:submitter_userimage%%]]
L0112 [[/code]]
L0113 
L0114 ++ Examples
L0115 
L0116 +++ Combine [http://slashdot.org/ Slashdot] and [http://digg.com/view/technology Digg technology] feeds
L0117 
L0118 URL for the feeds are:
L0119 * http://rss.slashdot.org/Slashdot/slashdot
L0120 * http://digg.com/rss/containertechnology.xml
L0121 
L0122 Simply do:
L0123 [[code]]
L0124 [[module Feed src="http://rss.slashdot.org/Slashdot/slashdot;http://digg.com/rss/containertechnology.xml"]]
L0125 [[/code]]
L0126 
L0127 More advanced example:
L0128 [[code]]
L0129 [[module Feed src="http://rss.slashdot.org/Slashdot/slashdot;http://digg.com/rss/containertechnology.xml" limit="20"]]
L0130 ++ %%linked_title%% (%%linked_channel_title%%)
L0131 
L0132 %%date%%, submitted by %%custom digg:submitter_username%% %%custom dc:creator%%
L0133 
L0134 %%description%%
L0135 [[/module]]
L0136 
L0137 [[module Feed src="http://rss.slashdot.org/Slashdot/slashdot;http://digg.com/rss/containertechnology.xml" offset="20" limit="20"]]
L0138 **%%linked_title%%** (%%linked_channel_title%%, %%date%% by %%custom digg:submitter_username%% %%custom dc:creator%%)
L0139 [[/module]]
L0140 [[/code]]
L0141 
L0142 
L0143 
L0144 which displays first 20 items in detail and next 20 items just printing the title, channel title, date and submitter.
L0145 
L0146 BTW: a very nice digg feed import is presented at our [http://snippets.wikidot.com/code:import-the-digg-feed snippets code repository here].
```
