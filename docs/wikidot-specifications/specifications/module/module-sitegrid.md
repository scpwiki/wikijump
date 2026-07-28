# SiteGrid Module

- Feature ID: `module-sitegrid`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `SiteGrid` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:sitegrid-module/source.wikidot.txt:1` through line 104 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:sitegrid-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:sitegrid-module/source.wikidot.txt:1` through line 104  
SHA-256 of complete source file: `632c9413ee2919c3fb22df2bdcd27c336e229ae0d9d7c896c4675e66841e4212`

```wikidot
L0001 ++ Description
L0002 
L0003 SiteGrid module gives you a possibility to create a grid of thumbnails of sites on Wikidot with a descriptions and some details in a form of pop-up when you move your mouse cursor over thumbnail.
L0004 
L0005 SiteGrid module is sorting thumbnails randomly. It's used on the main Wikidot page to list featured sites.
L0006 
L0007 Inside the module you have to put a list either of the names of sites, i.e. **community** (if the address is //community.wikidot.com//) or full address, i.e. **community.wikidot.com** or **www.digistan.org** (if a site has a custom domain).
L0008 
L0009 ++ Attributes
L0010 
L0011 ||~ Attribute ||~ Required||~ Allowed values||~ Default||~ Description||
L0012 || limit || no || any integer || none || Limiting the number of displayed  thumbnails from the predefined list ||
L0013 
L0014 ++ Examples
L0015 
L0016 [[code]]
L0017 [[module SiteGrid limit="20"]]
L0018 wikipiano
L0019 michal
L0020 wikiwealth
L0021 quake
L0022 fretsonfire
L0023 squark
L0024 istorijska-biblioteka
L0025 angels
L0026 string-theory
L0027 fifa360
L0028 liquidrescale
L0029 qttabbar
L0030 osx86
L0031 moonworld
L0032 wiihd
L0033 wherearethejoneses
L0034 sniki.org
L0035 wow-unity
L0036 heroesmush
L0037 bvswiki.com
L0038 scp-wiki
L0039 www.digistan.org
L0040 mechanics
L0041 aeldaria
L0042 coffeetime
L0043 f-g
L0044 thehurl
L0045 arch1k
L0046 comicbooks
L0047 karmalab
L0048 scmapdb
L0049 tibasicdev
L0050 gamedesign
L0051 herald-tips-tricks
L0052 swib
L0053 skyscraper-en
L0054 terrasdeportugal
L0055 scmapdb
L0056 l4dmapdb
L0057 [[/module]]
L0058 [[/code]]
L0059 
L0060 
L0061 Which transfers to...
L0062 
L0063 [[module SiteGrid limit="20"]]
L0064 wikipiano
L0065 michal
L0066 wikiwealth
L0067 quake
L0068 fretsonfire
L0069 squark
L0070 istorijska-biblioteka
L0071 angels
L0072 string-theory
L0073 fifa360
L0074 liquidrescale
L0075 qttabbar
L0076 osx86
L0077 moonworld
L0078 wiihd
L0079 wherearethejoneses
L0080 sniki.org
L0081 wow-unity
L0082 heroesmush
L0083 bvswiki.com
L0084 scp-wiki
L0085 www.digistan.org
L0086 mechanics
L0087 aeldaria
L0088 coffeetime
L0089 f-g
L0090 thehurl
L0091 arch1k
L0092 comicbooks
L0093 karmalab
L0094 scmapdb
L0095 tibasicdev
L0096 gamedesign
L0097 herald-tips-tricks
L0098 swib
L0099 skyscraper-en
L0100 terrasdeportugal
L0101 scmapdb
L0102 l4dmapdb
L0103 [[/module]]
L0104 This is a long list of featured sites, but only 20 are displayed in a random order.
```
