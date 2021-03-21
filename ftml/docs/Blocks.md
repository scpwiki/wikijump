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

### Blocks

(TODO)

### Modules

(TODO)
