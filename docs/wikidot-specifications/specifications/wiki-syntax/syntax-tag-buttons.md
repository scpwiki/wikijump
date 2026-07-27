# Button for tag update syntax

- Feature ID: `syntax-tag-buttons`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented button for tag update syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:tag-buttons/source.wikidot.txt:1` through line 23 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:tag-buttons (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:tag-buttons/source.wikidot.txt:1` through line 23  
SHA-256 of complete source file: `4c669a62b9cb6cb5a3858e620b41aae50b881092d3aefd0ba86aad8793db6116`

```wikidot
L0001 You can use {{@@[[button set-tags <tag_alterations> text="<button_text>"]]@@}} to change page tags easily.
L0002 
L0003 <tag_alterations> is one or more of the following (separated by space):
L0004 
L0005 * {{+tag}} -- will add a tag to the page if not already present
L0006 * {{-tag}} -- will remove a tag from the page if present
L0007 * {{-*}} -- will remove all the visible tags from the page (those not starting from "_")
L0008 * {{-_*}} -- will remove all the hidden tags from the page (those starting from "_")
L0009 
L0010 The action will happen when user clicks on the button and has permissions to edit the page. The page will reload afterwards (this is useful if you have some iftags constructions on the page).
L0011 
L0012 Any tag removal will happen before tag addition.
L0013 
L0014 Examples:
L0015 
L0016 ||~ code ||~ creates button that ... when clicked ||
L0017 || {{@@[[button set-tags +tag1 -tag2 text="Change tags"]]@@}} || adds tag {{tag1}} and removes tag {{tag2}} ||
L0018 || {{@@[[button set-tags +favorite +_book -_movie text="Change tags"]]@@}} || add tags {{favorite}} and {{_book}} and removes tag {{_movie}} ||
L0019 || {{@@[[button set-tags +favorite -* text="Change tags"]]@@}} || add tags {{favorite}} and removes other visible tags (tags starting with "_" are kept) ||
L0020 || {{@@[[button set-tags -* -_* text="Change tags"]]@@}} || clears all the tags ||
L0021 || {{@@[[button set-tags -* +favorite +_book text="Change tags"]]@@}} || adds tag favorite, removes other visible tags and adds tag _book keeping all tags starting with "_" ||
L0022 
L0023 Class and style attributes work like for standalone buttons for page actions.
```
