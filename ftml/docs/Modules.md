## Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. [As noted above](#blocks), all modules accept separate newlines and do not accept special or variant flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name  | AST Output           | htmlOutput                               | Notes |
|--------------|----------------------|-------------------------------------------|-------|
| Backlinks    | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| Categories   | `Module::Categories` | `<div class="categories-module-box">`     | |
| CSS          | N/A                  | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| Join         | `Module::Join`       | `<div class="join-box">`                  | |
| PageTree     | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| Rate         | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |

TODO: add individual doc sections for each module
