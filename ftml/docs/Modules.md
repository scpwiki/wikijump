## Syntax Documentation: Modules

ftml uses the term "module" to refer any name accepted by the `[[module]]` block which has associated properties. Examples include `[[module Rate]]`, `[[module CSS]]`, and `[[module ListPages]]`.

The text before the first space or end of the block head is the "name", and designates which module should be used. This is always case-insensitive.

See [Blocks](Blocks.md) for an explanation for common concepts in Wikidot blocks, such as arguments and bodies.  This document assumes knowledge of those terms. See also [Wikidot's documentation on modules](https://www.wikidot.com/doc-modules:start), not all of which are implemented here.

## List of Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. [As noted above](#blocks), all modules accept separate newlines and do not accept special or variant flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name               | AST Output           | HTML Output                               | Notes |
|---------------------------|----------------------|-------------------------------------------|-------|
| [Backlinks](#backlinks)   | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| [Categories](#categories) | `Module::Categories` | `<div class="categories-module-box">`     | |
| [CSS](#css)               | N/A                  | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| [Join](#join)             | `Module::Join`       | `<div class="join-box">`                  | |
| [PageTree](#pagetree)     | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| [Rate](#rate)             | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |

### Backlinks

%

### Categories

%

### CSS

%

### Join

%

### PageTree

%

### Rate

%
