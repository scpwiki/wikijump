## Syntax Documentation: Blocks

ftml uses the term "block" to refer to the syntactical construction beginning in `[[`, containing some text, and ending in `]]`.

Blocks may start with `[[*` (referred to in the code as its `special` flag), have a case-insensitive name, and then optional arguments until it terminates in `]]`. Some blocks require a body, and some only exist as a head. All bodies are terminated by `[[/name]]` (where `name` being the case-insensitive block name).

How the bodies of a block are interpreted depend on its type. They fall into one of the following categories:

| Name            | Example block | Description |
|-----------------|---------------|-------------|
| None            | `[[include]]` | Has no body. This block terminates at its head. |
| Raw text        | `[[code]]`    | Interprets the entire block body as raw text. Syntax is not parsed. |
| Nested elements | `[[div]]`     | Interprets block contents as elements in a certain context. These are then nested in the block. |
| Other           |               | Uses some other means of interpreting its body. Some Wikidot blocks allow passing YAML, for instance. |

Of note that while `[[module]]` is its own block, it requires specifying a module name, and this behaves similarly to other blocks in that their attributes are determined by the module name.

Blocks may also have a variant, which means that terminating the name in `_` produces modified behavior. Only the presence of the trailing `_` in the head is used to determine the block's behavior.

Finally, blocks may accept deliminated newlines. While these blocks can be used inline, separating them on their own lines will not produce line breaks. For instance:

The `[[div]]` block accepts separate newlines. These two constructions are the same:

```
[[div]]Apple[[/div]]
```

```
[[div]]
Apple
[[/div]]
```

The `[[span]]` block does not accept separate newlines. These two constructions are different, as the latter will add line breaks for each newline in the source:

```
[[span]]Banana[[/span]]
```

```
[[span]]
Banana
[[/span]]
```

### Blocks

| Block Name  | Accepted Names        | Special? | Variant? | Newlines? | AST Output | HTML Output | Notes |
|-------------|-----------------------|----------|----------|-----------|------------|-------------|-------|
| Anchor      | `a`, `anchor`         | No       | Yes      | No        | `Element::Anchor` | `<a>` | Variant strips trailing and leading newlines from output. |
| Blockquote  | `blockquote`, `quote` | No       | No       | Yes       | `Element::Container(Blockquote)` | `<blockquote>` | |
| Bold        | `b`, `bold`, `strong` | No       | No       | No        | `Element::Container(Bold)` | `<strong>` | |
| Checkbox    | `checkbox`            | Yes      | No       | No        | `Element::CheckBox` | `<input type="checkbox">` | If special is set, the checkbox begins checked. |
| Code        | `code`                | No       | No       | Yes       | `Element::Code` | `<div class="code">` | |
| Collapsible | `collapsible`         | No       | No       | Yes       | `Element::Collapsible` | `<div class="collapsible-block">` | |
| CSS         | `css`                 | No       | No       | Yes       | - | `<style>` | Outputs contents as CSS. Alias for `[[module CSS]]`. |
| Deletion    | `del`, `deletion`     | No       | No       | No        | `Element::Container(Deletion)` | `<del>` | |
| Div         | `div`                 | No       | Yes      | Yes       | `Element::Container(Div)` | `<div>` | Variant strips trailing and leading newlines from output. |
| Hidden      | `hidden`              | No       | No       | Yes       | `Element::Container(Hidden)` | `<span class="hidden">` | |
| HTML        | `html`                | No       | No       | Yes       | `Element::Html` | `<iframe>` | Embeds this as an HTML snippet on `wjfiles.com`, hosted in an iframe. |
| Iframe      | `iframe`              | No       | No       | Yes       | `Element::Iframe` | `<iframe>` |
| Include     | `include`             | No       | No       | Yes       | - | - | Handled in the preprocessor. Includes the contents from the target page here, as if pasted in. |
| Insertion   | `ins`, `insertion`    | No       | No       | No        | `Element::Container(Insertion)` | `<ins>` | |
| Invisible   | `invisible`           | No       | No       | Yes       | `Element::Container(Invisible)` | `<span class="invisible">` |
| Italics     | `i`, `italics`, `em`, `emphasis` | No | No | No         | `Element::Container(Italics)` | `<em>` | |
| Lines       | `lines`, `newlines`   | No       | No       | Yes       | `Element::LineBreaks` | `<br>` | |
| Mark        | `mark`, `highlight`   | No       | No       | No        | `Element::Container(Mark)` | `<mark>` | |
| Module      | `module`              | No       | No       | Yes       | - | - | See section below on modules. |
| Monospace   | `tt`, `mono`, `monospace` | No   | No       | No        | `Element::Container(Monospace)` | `<tt>` | |
| Radio       | `radio`, `radio-button` | Yes    | No       | No        | `Element::RadioButton` | `<input type="radio">` | If special is set, the radio button begins selected. |
| Size        | `size`                | No       | No       | No        | `Element::Container(Size)` | `<span style="font-size: XXX;">` | |
| Span        | `span`                | No       | Yes      | No        | `Element::Container(Span)` | `<span>` | Variant strips trailing and leading newlines from output. |
| Strikethrough | `s`, `strikethrough` | No      | No       | No        | `Element::Container(Strikethrough)` | `<s>` | |
| Subscript   | `sub`, `subscript`    | No       | No       | No        | `Element::Container(Subscript)` | `<sub>` | |
| Superscript | `sup`, `super`, `superscript` | No | No     | No        | `Element::Container(Superscript)` | `<sup>` | |
| Underline   | `u`, `underline`      | No       | No       | No        | `Element::Container(Underline)` | `<u>` | |

### Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. As noted above, all modules accept separate newlines and do not accept special or variant flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name  | AST Output           | HTML Output                               | Notes |
|--------------|----------------------|-------------------------------------------|-------|
| Backlinks    | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| Categories   | `Module::Categories` | `<div class="categories-module-box">`     | |
| CSS          | -                    | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| Join         | `Module::Join`       | `<div class="join-box">`                  | |
| PageTree     | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| Rate         | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |
