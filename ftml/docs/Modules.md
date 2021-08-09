## Syntax Documentation: Modules

ftml uses the term "module" to refer any name accepted by the `[[module]]` block which has associated properties. Examples include `[[module Rate]]`, `[[module CSS]]`, and `[[module ListPages]]`.

The text before the first space or end of the block head is the "name", and designates which module should be used. This is always case-insensitive.

See [Blocks](Blocks.md) for an explanation for common concepts in Wikidot blocks, such as arguments and bodies.  This document assumes knowledge of those terms. See also [Wikidot's documentation on modules](https://www.wikidot.com/doc-modules:start), not all of which are implemented here.

## List of Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. [As noted above](#blocks), all modules accept separate newlines and do not accept star or score flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name               | Body | AST Output           | HTML Output                               | Notes |
|---------------------------|------|----------------------|-------------------------------------------|-------|
| [Backlinks](#backlinks)   | None | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| [Categories](#categories) | None | `Module::Categories` | `<div class="categories-module-box">`     | |
| [CSS](#css)               | Raw  | N/A                  | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| [Join](#join)             | None | `Module::Join`       | `<div class="join-box">`                  | |
| [PageTree](#pagetree)     | None | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| [Rate](#rate)             | None | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |

### Backlinks

Provides a list of pages which link to the page in question.

Body: None

Arguments:
* `page` &mdash; (Optional) The page to retrieve the back links to. Default: current page.

Example:

```
[[module Backlinks page="scp-173"]]
```

### Categories

Lists all categories on the site, and for each, a collapsible containing a list of each page within that category.

Body: None

Arguments:
* `includeHidden` &mdash; (Optional, Boolean) Whether hidden categories (those beginning with `_`) should be shown. Default: false.

Example:

```
[[module Categories]]
```

### CSS

Adds CSS styling that will be applied to the current page. An alias of `[[css]]`.

Body: Raw

Arguments:
* None

Example:

```
[[module CSS]]
#page-title {
    color: purple;
}
[[/module CSS]]
```

### Join

A button which permits users to join or apply to the current site.

Body: None

Arguments:
* `button` &mdash; (Optional, String) The text that should be present on the button.
* All accepted attributes.

Example:

```
[[module Join]]
```

### PageTree

Lists all the child pages of the page in question, including their children, in a hierarchical tree.

Body: None

Arguments:
* `root` &mdash; (Optional, Slug) The page to retrieve children for. Default: current page.
* `depth` &mdash; (Optional, Integer > 0) How deep the tree should span. Default: no limit.
* `showRoot` &mdash; (Optional, Boolean) Whether to show the root page in the tree. Default: false.

Example:

```
[[module PageTree root="scp-001" showroot="true"]]
```

### Rate

Body: None

Provides a rating module, which enables votes to be cast on a page.

Arguments:
* None

Example:
```
[[module rate]]
```
