## Syntax Documentation: Modules

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

TODO: add individual doc sections for each module
