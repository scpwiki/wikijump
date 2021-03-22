## Syntax Documentation: Blocks

ftml uses the term "block" to refer to the syntactical construction beginning in `[[`, containing some text, and ending in `]]`. Examples include `[[div]]`, `[[module]]`, and `[[span]]`.

The text after the `[[` is the name of the block. These are always case-insensitive.

Blocks have five variable properties worth noting:

1. The block may start with `[[*` rather than `[[`. This is referred to as the "special" flag.
2. Their name may end in `_`. This is referred to as the "modifier" flag. This underscore is ignored if found in the tail.
3. The block may have a number of arguments before ending in `]]`.
4. The block may accept delimited newlines. This is explained in more detail below.
5. The block may have a body. It has contents that is terminated by `[[/name]]` (where `name` is the block name).

Whether particular blocks accept these variables is noted in the table below. If a block does not permit that variance, then it will not parse. For instance, if a block does not permit special variance but a `*` is added anyways.

### Arguments

Blocks may have one of the following approaches when parsing arguments:

| Name             | Example block       | Method | Description |
|------------------|---------------------|--------|-------------|
| None             | `[[CSS]]`           | `BlockParser::get_head_none()` | Accepts no arguments. Tokens which are not `]]` will result in parsing failure. |
| Value            | `[[size 50%]]`      | `BlockParser::get_head_value()` | All of the text until `]]` is interpreted as a single text value. |
| Map              | `[[span id="abc"]]` | `BlockParser::get_head_map()` | Accepts an arbitrary mapping of `key="value"` arguments. Values must be double-quoted, and may contain escapes (e.g. `\"`, `\n`). |
| Name Map         | `[[iframe https://example.com/ style="width: 100%;"]]` | `BlockParser::get_head_name_map()` | Accepts a single text value terminated by a space, then an arbitrary mapping as described above. |

### Newlines

Blocks may accept deliminated newlines. While these blocks can be used inline, separating them on their own lines will not produce line breaks. For instance:

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

### Body

How the bodies of a block are interpreted depend on its type. They fall into one of the following categories:

| Name            | Example block | Description |
|-----------------|---------------|-------------|
| None            | `[[include]]` | Has no body. This block terminates at its head. |
| Raw text        | `[[code]]`    | Interprets the entire block body as raw text. Syntax is not parsed. |
| Nested elements | `[[div]]`     | Interprets block contents as elements in a certain context. These are then nested in the block. |
| Other           | N/A           | Uses some other means of interpreting its body. Some Wikidot blocks allow passing YAML, for instance. |

### Module

Of note that while `[[module]]` is its own block, it requires specifying a module name, and this behaves similarly to other blocks in that their attributes are determined by the module name.

## Blocks

| Block Name  | Accepted Names        | Special? | Modifier? | Newlines? | AST Output | HTML Output | Notes |
|-------------|-----------------------|----------|-----------|-----------|------------|-------------|-------|
| Anchor      | `a`, `anchor`         | No       | Yes       | No        | `Element::Anchor` | `<a>` | Modifier strips trailing and leading newlines from output. |
| Blockquote  | `blockquote`, `quote` | No       | No        | Yes       | `Element::Container(Blockquote)` | `<blockquote>` | |
| Bold        | `b`, `bold`, `strong` | No       | No        | No        | `Element::Container(Bold)` | `<strong>` | |
| Checkbox    | `checkbox`            | Yes      | No        | No        | `Element::CheckBox` | `<input type="checkbox">` | If special is set, the checkbox begins checked. |
| Code        | `code`                | No       | No        | Yes       | `Element::Code` | `<div class="code">` | |
| Collapsible | `collapsible`         | No       | No        | Yes       | `Element::Collapsible` | `<div class="collapsible-block">` | |
| CSS         | `css`                 | No       | No        | Yes       | - | `<style>` | Outputs contents as CSS. Alias for `[[module CSS]]`. |
| Deletion    | `del`, `deletion`     | No       | No        | No        | `Element::Container(Deletion)` | `<del>` | |
| Div         | `div`                 | No       | Yes       | Yes       | `Element::Container(Div)` | `<div>` | Modifier strips trailing and leading newlines from output. |
| Hidden      | `hidden`              | No       | No        | Yes       | `Element::Container(Hidden)` | `<span class="hidden">` | |
| HTML        | `html`                | No       | No        | Yes       | `Element::Html` | `<iframe>` | Embeds this as an HTML snippet on `wjfiles.com`, hosted in an iframe. |
| Iframe      | `iframe`              | No       | No        | Yes       | `Element::Iframe` | `<iframe>` |
| Include     | `include`             | No       | No        | Yes       | - | - | Handled in the preprocessor. Includes the contents from the target page here, as if pasted in. |
| Insertion   | `ins`, `insertion`    | No       | No        | No        | `Element::Container(Insertion)` | `<ins>` | |
| Invisible   | `invisible`           | No       | No        | Yes       | `Element::Container(Invisible)` | `<span class="invisible">` |
| Italics     | `i`, `italics`, `em`, `emphasis` | No | No |  No         | `Element::Container(Italics)` | `<em>` | |
| Lines       | `lines`, `newlines`   | No       | No        | Yes       | `Element::LineBreaks` | `<br>` | |
| Mark        | `mark`, `highlight`   | No       | No        | No        | `Element::Container(Mark)` | `<mark>` | |
| Module      | `module`              | No       | No        | Yes       | - | - | See [section below](#modules) on modules. |
| Monospace   | `tt`, `mono`, `monospace` | No   | No        | No        | `Element::Container(Monospace)` | `<tt>` | |
| Radio       | `radio`, `radio-button` | Yes    | No        | No        | `Element::RadioButton` | `<input type="radio">` | If special is set, the radio button begins selected. |
| Size        | `size`                | No       | No        | No        | `Element::Container(Size)` | `<span style="font-size: XXX;">` | |
| Span        | `span`                | No       | Yes       | No        | `Element::Container(Span)` | `<span>` | Modifier strips trailing and leading newlines from output. |
| Strikethrough | `s`, `strikethrough` | No      | No        | No        | `Element::Container(Strikethrough)` | `<s>` | |
| Subscript   | `sub`, `subscript`    | No       | No        | No        | `Element::Container(Subscript)` | `<sub>` | |
| Superscript | `sup`, `super`, `superscript` | No | No      | No        | `Element::Container(Superscript)` | `<sup>` | |
| Underline   | `u`, `underline`      | No       | No        | No        | `Element::Container(Underline)` | `<u>` | |

## Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. [As noted above](#blocks), all modules accept separate newlines and do not accept special or variant flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name  | AST Output           | HTML Output                               | Notes |
|--------------|----------------------|-------------------------------------------|-------|
| Backlinks    | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| Categories   | `Module::Categories` | `<div class="categories-module-box">`     | |
| CSS          | -                    | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| Join         | `Module::Join`       | `<div class="join-box">`                  | |
| PageTree     | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| Rate         | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |
