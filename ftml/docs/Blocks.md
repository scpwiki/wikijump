## Syntax Documentation: Blocks

ftml uses the term "block" to refer to the syntactical construction beginning in `[[`, containing some text, and ending in `]]`. Examples include `[[div]]`, `[[module]]`, and `[[span]]`.

The text after the `[[` is the name of the block. This is always case-insensitive.

Blocks have five variable properties worth noting:

1. The block may start with `[[*` rather than `[[`. This is referred to as the "star" flag.
2. Their name may end in `_`. This is referred to as the "score" flag. This underscore is ignored if found in the foot.
3. The block may have some arguments before ending in `]]`.
4. The block may accept delimited newlines. This is explained in more detail [below](#newlines).
5. The block may have a body. It has contents that is terminated by `[[/name]]` (where `name` is the block name), referred to as the block footer or tail.

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
| None            | `[[iframe]]`  | Has no body. This block terminates at its head. |
| Raw text        | `[[code]]`    | Interprets the entire block body as raw text. Syntax is not parsed. |
| Nested elements | `[[div]]`     | Interprets block contents as elements in a certain context. These are then nested in the block. |
| Other           | N/A           | Uses some other means of interpreting its body. Some Wikidot blocks allow passing YAML for instance. |

### Module Block

Of note that while `[[module]]` is its own block, it requires specifying a module name, and this behaves similarly to other blocks in that their attributes are determined by the module name.

See [Modules](Modules.md) for information on each module currently implemented.

## List of Blocks

A list of all blocks and their attributes is available at [`conf/blocks.toml`](../conf/blocks.toml) (with an explanation of the format in [`conf/blocks.schema.toml`](../conf/blocks.schema.toml). Our continuous integration system enforces that it is always up-to-date.

Alternatively you may look here for a formatted list: (though it may not be updated as consistently)

| Block Name                        | Accepted Names                   | Star? | Score? | Newlines? | Argument Type | Body Type |
|-----------------------------------|----------------------------------|-------|--------|-----------|---------------|-----------|
| [Anchor](#anchor)                 | `a`, `anchor`                    | No    | Yes    | No        | Map           | Elements  |
| [Blockquote](#blockquote)         | `blockquote`, `quote`            | No    | No     | Yes       | Map           | Elements  |
| [Bold](#bold)                     | `b`, `bold`, `strong`            | No    | No     | No        | Map           | Elements  |
| [Char](#char)                     | `char`, `character`              | No    | No     | No        | Value         | None      |
| [Checkbox](#checkbox)             | `checkbox`                       | Yes   | No     | No        | Map           | None      |
| [Code](#code)                     | `code`                           | No    | No     | Yes       | Map           | Raw       |
| [Collapsible](#collapsible)       | `collapsible`                    | No    | No     | Yes       | Map           | Elements  |
| [CSS](#css)                       | `css`                            | No    | No     | Yes       | None          | Raw       |
| [Deletion](#deletion)             | `del`, `deletion`                | No    | No     | No        | Map           | Elements  |
| [Div](#div)                       | `div`                            | No    | Yes    | Yes       | Map           | Elements  |
| [Footnote](#footnote)             | `footnote`                       | No    | No     | No        | None          | Elements  |
| [Footnote Block](#footnote-block) | `footnoteblock`                  | No    | No     | Yes       | Map           | None      |
| [Hidden](#hidden)                 | `hidden`                         | No    | No     | Yes       | Map           | Elements  |
| [HTML](#html)                     | `html`                           | No    | No     | Yes       | Map           | Raw       |
| [IfCategory](#ifcategory)         | `ifcategory`                     | No    | No     | Yes       | Value         | Elements  |
| [IfTags](#iftags)                 | `iftags`                         | No    | No     | Yes       | Value         | Elements  |
| [Iframe](#iframe)                 | `iframe`                         | No    | No     | Yes       | None          | None      |
| [Image](#image)                   | `image`                          | No    | No     | No        | Name + Map    | None      |
| [Include](#include)               | `include`                        | No    | No     | Yes       | Name + Map    | None      |
| [Insertion](#insertion)           | `ins`, `insertion`               | No    | No     | No        | Map           | Elements  |
| [Invisible](#invisible)           | `invisible`                      | No    | No     | Yes       | Map           | Elements  |
| [Italics](#italics)               | `i`, `italics`, `em`, `emphasis` | No    | No     | No        | Map           | Elements  |
| [Lines](#lines)                   | `lines`, `newlines`              | No    | No     | Yes       | Value         | None      |
| [List Blocks](#list)              | `ul`, `ol`, `li`                 | No    | Yes    | Yes       | Map           | Elements  |
| [Mark](#mark)                     | `mark`, `highlight`              | No    | No     | No        | Map           | Elements  |
| [Module](#module)                 | `module`                         | No    | No     | Yes       | (See below)   | (See below) |
| [Monospace](#monospace)           | `tt`, `mono`, `monospace`        | No    | No     | No        | Map           | Elements  |
| [Paragraph](#paragraph)           | `p`, `paragraph`                 | No    | No     | Yes       | Map           | Elements  |
| [Radio](#radio)                   | `radio`, `radio-button`          | Yes   | No     | No        | Name + Map    | None      |
| [Size](#size)                     | `size`                           | No    | No     | No        | Value         | Elements  |
| [Span](#span)                     | `span`                           | No    | Yes    | No        | Map           | Elements  |
| [Strikethrough](#strikethrough)   | `s`, `strikethrough`             | No    | No     | No        | Map           | Elements  |
| [Subscript](#subscript)           | `sub`, `subscript`               | No    | No     | No        | Map           | Elements  |
| [Superscript](#superscript)       | `sup`, `super`, `superscript`    | No    | No     | No        | Map           | Elements  |
| [Tables](#tables)                 | `table`, `row`, `cell`, `hcell`  | No    | No     | Yes       | Map           | Elements  |
| [TOC](#toc)                       | `toc`                            | No    | No     | Yes       | Map           | None      |
| [Underline](#underline)           | `u`, `underline`                 | No    | No     | No        | Map           | Elements  |
| [User](#user)                     | `user`                           | Yes   | No     | No        | Value         | None      |

Each of the blocks will be described in more detail below:

### Anchor

Outputs: `Element::Anchor` / `<a>`

Body: Elements

Accepts score (`_`): Strips leading and trailing newlines.

Arguments:
* All accepted attributes

Example:

```
[[a href="/scp-4000/noredirect/true" target="_blank" class="dual-link"]]Fae[[/a]]
```

### Blockquote

Outputs: `Element::Container(ContainerType::Blockqote)` / `<blockquote>`

Body: Elements

Accepts newline separation.

Arguments:
* All accepted attributes

Example:

```
[[blockquote]]
Some text here.
[[/blockquote]]
```

### Bold

Outputs: `Element::Container(ContainerType::Bold)` / `<strong>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
Some [[b]]text![[/b]]
```

### Char

Outputs: `Element::Text`

Body: None

Arguments:
Value &mdash; (String) The HTML entity to place here.

Example:

```
This file is [[char copy]] 2021 Team Wikijump.
```

### Checkbox

Outputs: `Element::CheckBox` / `<input type="checkbox">`

Body: None

Accepts star (`*`): Element starts checked.

Arguments:
* All accepted attributes

Example:

```
[[checkbox Apple]]
[[*checkbox Blueberry]]
[[checkbox Cherry]]
[[checkbox Durian]]
```

### Code

Outputs: `Element::Code` / `<pre class="wj-code"><code>`

Body: Raw

Accepts newline separation.

Arguments:
* `type` &mdash; (String) What language this block is in, both for its Content-Type and syntax highlighting.

Example:

```
[[code]]
This text is **not** rendered as Wikitext, but output as-is!
[[/code]]
```

### Collapsible

Output: `Element::Collapsible` / `<div class="wj-collapsible-block">`

Body: Elements

Accepts newline separation.

Arguments:
* `show` &mdash; (String) The text to present when text is collapsed (i.e. can be shown).
* `hide` &mdash; (String) The text to present when text is expanded (i.e. can be hidden).
* `folded` &mdash; (Boolean) `true` means start collapsed (default), `false` means start expanded.
* `hideLocation` &mdash; (Enum: One of `top`, `bottom`, `both`, or `neither`) Shows in what locations the hide collapsible link in.

Example:

```
[[collapsible show="+ Spoilers for Ouroboros" hide="- Spoilers!" hideLocation="bottom"]]
Overseers die.
[[/collapsible]]
```

### CSS

Output: None / `<style>`

Body: Raw

Accepts newline separation.

Arguments:
* None

Example:

```
[[css]]
#page-title {
    color: purple;
}
[[/css]]
```

### Deletion

Output: `Element::Container(ContainerType::Deletion)` / `<del>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
I [[del]]don't[[/del]] like that haircut.
```

### Div

Output: `Element::Container(ContainerType::Div)` / `<div>`

Body: Elements

Accepts score (`_`): Strips leading and trailing newlines.  
Accepts newline separation.

Arguments:
* All accepted attributes

Example:

```
[[div_ class="blockquote" style="border: none;"]]
Some text __here!__
[[/div]]
```

### Footnote

Output: `Element::Footnote`

Body: Elements

Arguments:
* None

Example:

```
The author of The Dark Tower series[[footnote]]Did you know that world-renowned writer Stephen King was once hit by a car? Just something to consider[[/footnote]] began work in the late 1970s.
```

### Footnote Block

Output: `Element::FootnoteBlock`

Body: None

Arguments:
* `hide` &mdash; (Boolean) Whether to hide the footnote block, effectively not rendering it.
* `title` &mdash; (String) An alternate title to the footnote block. In English, the default is `Footnotes`.

### Hidden

Output: `Element::Container(ContainerType::Hidden)` / `<span class="wj-hidden">`

Body: Elements

Accepts newline separation.

Arguments:
* All accepted attributes

Example:

```
This text is **visible**.

[[hidden]]
This text is not.
[[/hidden]]
```

### HTML

Output: `Element::Html` / `<iframe>`

Body: Raw

Accepts newline separation.

Arguments:
* None

Example:

```
[[html]]
<h2>Exciting!</h2>

<p>
This HTML will appear in an iframe hosted on wjfiles!
</p>
[[/html]]
```

### IfCategory

Output: `Element::IfCategory`

Body: Elements

Accepts newline separation.

Arguments:
* A list of space separated category names, optionally prefixed with `+` or `-`

Example:
```
[[ifcategory +_default -component]]
This text won't appear in the component: category!
[[/ifcategory]]
```

### IfTags

Output: `Element::IfTags`

Body: Elements

Accepts newline separation.

Arguments:
* A list of space separated category names, optionally prefixed with `+` or `-`

Example:
```
[[iftags -admin -hub +scp +euclid]]
This appears if tagged {{scp}} and {{euclid}}!
[[/iftags]]
```

### Iframe

Output: `Element::Iframe` / `<iframe>`

Body: None

Accepts newline separation.

Arguments:
* All accepted attributes

Example:

```
My website:

[[iframe https://example.com/ class="website"]]
```

### Image

Output: `Element::Image` / `<img>`

Body: None

Arguments:
* Value &mdash; (String) The source of the image.
* `link` &mdash; (String) The link that this image should point to.
* All accepted attributes.

### Include

This is not a typical block, as it is handled in the preprocessor. Parsing here is handled differently, but this block is still documented for completion sake.

Output: N/A

Body: None

Accepts newline separation.

Arguments:
* All arguments are passed as variables to the included page

Example:

```
[[include theme:black-highlighter-theme]]

[[include component:fancy-object-class
    class=Keter |
    classification=4 |
    taskforce=MTF-Eta-10 ("See No Evil")
]]
```

### Insertion

Output: `Element::Container(ContainerType::Insertion)` / `<ins>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
I would like some [[ins]]anchovy[[/ins]] pizza please, thank you.
```

### Invisible

Output: `Element::Container(ContainerType::Invisible)` / `<span class="wj-invisible">`

Body: Elements

Accepts newline separation.

Arguments:
* All accepted attributes

Example:

```
This text appears [[invisible]]but still takes up space, and can be selected.[[/invisible]]

More correct and much more portable than setting the font color to "white".
```

### Italics

Output: `Element::Container(ContainerType::Italics)` / `<em>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
This text is regular, but [[em]]this text is emphasized[[/em]].
```

### Lines

Output: `Element::LineBreaks` / `<br>`

Body: None

Accepts newline separation.

Arguments:
Value &mdash; (Positive integer) Number of line breaks to output

Example:

```
[[newlines 4]]

[!-- Much easier than spamming "@@@@"s --]
```

### List Blocks

Input: `[[ul]]`, `[[ol]]`, `[[li]]`

Output: `Element::List` / `<ul>` / `<ol>` / `<li>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
[[ul]]
  [[li]]
    [[ol]]
      [[li]] Item A [[/li]]
      [[li]] Item B [[/li]]
    [[/ol]]
  [[/li]]
  [[li]] Item C [[/li]]
[[/ul]]
```

The parser will produce a warning if `[[li]]` items are not within an `[[ol]]` or `[[ul]]` block.

### Mark

Output: `Element::Container(ContainerType::Mark)` / `<mark>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
This text is [[mark]]highlighted![[/mark]]
```

### Module

Output: `Element::Module` / Depends on module type

Body: Depends on module type

Accepts newline separation.

Arguments:
* [See documentation for specific modules](Modules.md)

Example:

```
[[module NameOfModuleHere someArgument="yes"]]
```

### Monospace

Output: `Element::Container(ContainerType::Monospace)` / `<tt>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
[[tt]]This output looks like it came from a typewriter or computer terminal.[[/tt]]
```

### Paragraph

Output: `Element::Container(ContainerType::Paragraph)` / `<p>`

Body: Elements

Arguments:
* All accepted attributes

Example

```
[[p]]My contents of a paragraph here.[[/p]]
```

### Radio

Accepts star (`*`): Element starts selected.

Body: None

Output: `Element::RadioButton` / `<input type="radio">`

Arguments:
* All accepted attributes

Example:

```
Favorite kind of music:

[[radio]] Disco
[[radio]] Dance
[[radio]] Rap
[[*radio]] Noise
```

### Size

Output: `Element::Container(ContainerType::Size)` / `<span style="font-size: XXX;">`

Body: Elements

Arguments:
Value &mdash; (String) The CSS string denoting what size should be used here.

Example:

```
This text is regular, but [[size 250%]]this text is much larger[[/size]].
```

### Span

Output:`Element::Span` / `<span>`

Body: Elements

Accepts score (`_`): Strips leading and trailing newlines.

Arguments:
* All accepted attributes

Example:

```
This text is in a span: [[span class="fruit"]]banana[[/span]]
```

### Strikethrough

Output: `Element::Container(ContainerType::Strikethrough)` / `<s>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
This text is [[s]]struck through![[/s]]
```

### Subscript

Output: `Element::Container(ContainerType::Subscript)` / `<sub>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
Let this variable be called x[[sub]]A[[/sub]].
```

### Superscript

Output: `Element::Container(ContainerType::Superscript)` / `<sup>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
Thus, the result is n[[sup]]2[[/sup]].
```

### Tables

Input: `[[table]]`, `[[row]]`, `[[cell]]`, `[[hcell]]`

Output: `Element::Table` / `<table>` / `<tr>` / `<td>` / `<th>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
[[table]]
  [[row]]
    [[hcell]] Name [[/hcell]]
    [[hcell]] Price [[/hcell]]
    [[hcell]] Stock [[/hcell]]
  [[/row]]

  [[row]]
    [[cell]] Banana [[/cell]]
    [[cell]] $0.30 [[/cell]]
    [[cell]] 87 [[/cell]]
  [[/row]]

  [[row]]
    [[cell]] Cherry [[/cell]]
    [[cell]] $0.80 [[/cell]]
    [[cell]] 64 [[/cell]]
  [[/row]]
[[/table]]
```

The parser requires a structure of `[[table]]` containing only `[[row]]`s, and
those containing only `[[cell]]`s or `[[hcell]]`s. Cells may contain other tables.

### TOC

Name: Table of Contents

Output: `Element::TableOfContents` / `<div id="wj-toc">`

### User

Output: `Element::User` / `<div class="wj-user-info">`

### Underline

Output: `Element::Container(ContainerType::Underline)` / `<u>`

Body: Elements

Arguments:
* All accepted attributes

Example:

```
[[u]]Testing log 7192-45:[[/u]]
```
