## Syntax Documentation: Blocks

ftml uses the term "block" to refer to the syntactical construction beginning in `[[`, containing some text, and ending in `]]`. Examples include `[[div]]`, `[[module]]`, and `[[span]]`.

The text after the `[[` is the name of the block. These are always case-insensitive.

Blocks have five variable properties worth noting:

1. The block may start with `[[*` rather than `[[`. This is referred to as the "special" flag.
2. Their name may end in `_`. This is referred to as the "modifier" flag. This underscore is ignored if found in the tail.
3. The block may have some arguments before ending in `]]`.
4. The block may accept delimited newlines. This is explained in more detail [below](#newlines).
5. The block may have a body. It has contents that is terminated by `[[/name]]` (where `name` is the block name), referred to as the tail.

Whether particular blocks accept these variables is noted in the table below. If a block does not permit that variance, then it will fail to parse.

### Arguments

Blocks may have one of the following approaches when parsing arguments:

| Name             | Example block       | Method | Description |
|------------------|---------------------|--------|-------------|
| None             | `[[CSS]]`           | `BlockParser::get_head_none()` | Accepts no arguments. Tokens which are not `]]` will result in parsing failure. |
| Value            | `[[size 50%]]`      | `BlockParser::get_head_value()` | All of the text until `]]` is interpreted as a single text value. |
| Map              | `[[span id="abc"]]` | `BlockParser::get_head_map()` | Accepts an arbitrary mapping of `key="value"` arguments. Values must be double-quoted, and may contain escapes (e.g. `\"`, `\n`). |
| Name + Map       | `[[iframe https://example.com/ style="width: 100%;"]]` | `BlockParser::get_head_name_map()` | Accepts a single text value terminated by a space, then an arbitrary mapping as described above. |

Like block names, argument keys are case-insensitive.

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
| Other           | N/A           | Uses some other means of interpreting its body. Some Wikidot blocks allow passing YAML for instance. |

### Module Block

Of note that while `[[module]]` is its own block, it requires specifying a module name, and this behaves similarly to other blocks in that their attributes are determined by the module name.

## Blocks

Here is a table showing the options each block has with regards to its construction:

| Block Name  | Accepted Names                   | Special? | Modifier? | Newlines? | Argument Type | Body Type |
|-------------|----------------------------------|----------|-----------|-----------|---------------|-----------|
| Anchor      | `a`, `anchor`                    | No       | Yes       | No        | Map           | Elements  |
| Blockquote  | `blockquote`, `quote`            | No       | No        | Yes       | Map           | Elements  |
| Bold        | `b`, `bold`, `strong`            | No       | No        | No        | Map           | Elements  |
| Checkbox    | `checkbox`                       | Yes      | No        | No        | Map           | None      |
| Code        | `code`                           | No       | No        | Yes       | Map           | Raw       |
| Collapsible | `collapsible`                    | No       | No        | Yes       | Map           | Elements  |
| CSS         | `css`                            | No       | No        | Yes       | None          | Raw       |
| Deletion    | `del`, `deletion`                | No       | No        | No        | Map           | Elements  |
| Div         | `div`                            | No       | Yes       | Yes       | Map           | Elements  |
| Hidden      | `hidden`                         | No       | No        | Yes       | Map           | Elements  |
| HTML        | `html`                           | No       | No        | Yes       | Map           | Raw       |
| Iframe      | `iframe`                         | No       | No        | Yes       | None          | None      |
| Include     | `include`                        | No       | No        | Yes       | Name + Map    | None      |
| Insertion   | `ins`, `insertion`               | No       | No        | No        | Map           | Elements  |
| Invisible   | `invisible`                      | No       | No        | Yes       | Map           | Elements  |
| Italics     | `i`, `italics`, `em`, `emphasis` | No       | No        | No        | Map           | Elements  |
| Lines       | `lines`, `newlines`              | No       | No        | Yes       | Value         | None      |
| Mark        | `mark`, `highlight`              | No       | No        | No        | Map           | Elements  |
| Module      | `module`                         | No       | No        | Yes       | (See below)   | (See below) |
| Monospace   | `tt`, `mono`, `monospace`        | No       | No        | No        | Map           | Elements  |
| Radio       | `radio`, `radio-button`          | Yes      | No        | No        | Name + Map    | None      |
| Size        | `size`                           | No       | No        | No        | Map           | Elements  |
| Span        | `span`                           | No       | Yes       | No        | Map           | Elements  |
| Strikethrough | `s`, `strikethrough`           | No       | No        | No        | Map           | Elements  |
| Subscript   | `sub`, `subscript`               | No       | No        | No        | Map           | Elements  |
| Superscript | `sup`, `super`, `superscript`    | No       | No        | No        | Map           | Elements  |
| Underline   | `u`, `underline`                 | No       | No        | No        | Map           | Elements  |

Each of the blocks will be described in more detail below:

### Anchor

**Accepts:**
* Modifier &emdash; Strips leading and trailing newlines.

**Abstract Syntax Tree Output:** `Element::Anchor`

**HTML Output:** `<a>`

**Arguments:**
* All accepted attributes

**Example:**

```
[[a href="/scp-4000/noredirect/true" target="_blank" class="dual-link"]]Fae[[/a]]
```

### Blockquote

**Accepts:**
* Newlines

**Abstract Syntax Tree Output:** `Element::Container(ContainerType::Blockqote)`

**HTML Output:** `<blockquote>`

**Arguments:**
* All accepted attributes

**Example:**

```
[[blockquote]]
Some text here.
[[/blockquote]]
```

### Bold

**Accepts:**
* (none)

**Abstract Syntax Tree Output:** `Element::Bold`

**HTML Output:** `<strong>`

**Arguments:**
* All accepted attributes

**Example:**

```
Some [[b]]text![[/b]]
```

### Checkbox

**Accepts:**
* Special &emdash; Element starts checked

**Abstract Syntax Tree Output:** `Element::CheckBox`

**HTML Output:** `<input type="checkbox">`

**Arguments:**
* All accepted attributes

**Example:**

```
[[checkbox Apple]]
[[*checkbox Blueberry]]
[[checkbox Cherry]]
[[checkbox Durian]]
```

### Code

**Accepts:**
* Newlines

**Abstract Syntax Tree Output:** `Element::Code`

**HTML Output:** `<div class="code">`

**Arguments:**
* `type` &emdash; What language this block is in, both for its Content-Type and syntax highlighting.

**Example:**

```
[[code]]
This text is **not** rendered as Wikitext, but output as-is!
[[/code]]
```

## Modules

The table below follows essentially the same schema as for blocks in general, with a few changes. [As noted above](#blocks), all modules accept separate newlines and do not accept special or variant flags. Additionally, the list of accepted names is the same as the module name (but case-insensitive).

| Module Name  | AST Output           | HTML Output                               | Notes |
|--------------|----------------------|-------------------------------------------|-------|
| Backlinks    | `Module::Backlinks`  | `<div class="backlinks-module-box"> <ul>` | |
| Categories   | `Module::Categories` | `<div class="categories-module-box">`     | |
| CSS          | N/A                  | `<style>`                                 | Outputs contents as CSS. Alias for `[[css]]`. |
| Join         | `Module::Join`       | `<div class="join-box">`                  | |
| PageTree     | `Module::PageTree`   | `<div class="pagetree-module-box"> <ul>`  | |
| Rate         | `Module::Rate`       | `<div class="page-rate-widget-box">`      | |
