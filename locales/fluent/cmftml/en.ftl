### CodeMirror FTML Extension

## Linting

cmftml-lint =
  .warning-source = ftml({ $rule } = { $kind } at { $token }) [{ $from }, { $to }]

  .recursion-depth-exceeded = Too much recursion in markup.

  .end-of-input = Rule of type '{ $rule }' couldn't be processed before the end of the document was reached.

  .no-rules-matched = The string '{ $slice }' doesn't match anything and will be rendered as plain text.

  .rule-failed = The rule '{ $rule }' failed to match here, and had to fallback another rule.

  .not-start-of-line = The rule '{ $rule }' failed to match here, as it can only match on the start of a new line.

  .invalid-include = This include is invalid and won't be rendered.

  .list-empty = This list has nothing inside of it.

  .list-contains-non-item = This list has direct children that aren't list-item blocks.

  .list-item-outside-list = This list-item isn't within a list.

  .list-depth-exceeded = This list is nested too deeply, and can't be rendered.

  .table-contains-non-row = This table has direct children that aren't table rows.

  .table-row-contains-non-cell = This table-row has direct children that aren't cells.

  .table-row-outside-table = This table-row isn't within a table.

  .table-cell-outside-table = This table-cell isn't within a table-row.

  .footnotes-nested = This footnote is invalid because it's inside another footnote.

  .blockquote-depth-exceeded = This blockquote is nested too deeply, and can't be rendered.

  .no-such-block = Unknown block '{ $slice }'.

  .block-disallows-star = Block '{ $slice }' doesn't support a star invocation. (starting '*' character)

  .block-disallows-score = Block '{ $slice }' doesn't support a score invocation. (starting '_' character)

  .block-missing-name = Block '{ $slice }' requires a name/value, but none is specified.

  .block-missing-close-brackets = This block is missing closing ']]' brackets.

  .black-malformed-arguments = Block '{ $slice }' has malformed arguments.

  .block-missing-arguments = Block '{ $slice }' is missing one or more required arguments.

  .block-expected-end = The block of type '{ $rule }' was expected to end by at least this point.

  .block-end-mismatch = The block of type '{ $rule }' was expected to end here, not '{ $slice }'.

  .no-such-module = Unknown module '{ $slice }'.

  .module-missing-name = A module name was expected to be provided.

  .no-such-page = The page '{ $slice }' doesn't exist.

  .invalid-url = The URL '{ $slice }' is invalid.

## Block Acceptance

cmftml-block-accepts =
  .star =
    This block accepts the '*' (star) prefix.
    The effect of providing this prefix depends on the block.

  .score =
    This block accepts the '_' (score) suffix,
    which will strip leading and trailing newlines.

  .newlines =
    This block accepts newlines between its start and end nodes.

  .html-attributes =
    This block accepts generic HTML attributes/arguments.
    HTML attributes are subject to a whitelist, but regardless most can be used.

## Block Argument Types

cmftml-argument-none = NONE
  .info = This block doesn't accept any arguments.

cmftml-argument-value = VALUE
  .info = This block accepts text between the start and end of the node.

cmftml-argument-map = MAP
  .info = This block accepts arguments.

cmftml-argument-value-map = VALUE+MAP
  .info = This block accepts text, and then following a space accepts arguments.

## Block Documentation

cmftml-block-undocumented =
  This block currently does not have any documentation, but it is valid.

cmftml-block-anchor =
  .title = Anchor
  .info =
    Creates links between pages or other web pages. Like the HTML {"{{"}<a>{"}}"} tag, the
    {"{{"}href{"}}"} attribute needs to be used with the {"{{"}[[anchor]]{"}}"} block.
  .example =
    \[[a href="/scp-4000/noredirect/true" target="_blank" class="dual-link"]]Fae[[/a]]

cmftml-block-blockquote =
  .title = Blockquote
  .info =
    Indicates that enclosed text is an extended quotation,
    although the "extended quotation" interpretation of blockquotes
    may not be how they're used in practice. The meaning of blockquotes
    differs from site-to-site and often depends on the styling given to them.

    Using the {"{{"}[[blockquote]]{"}}"} block is the same as using {"{{"}> text{"}}"},
    with the difference only being the syntax used.
  .example =
    \[[blockquote]]
    Some text here.
    \[[/blockquote]]
    \[!-- equivalent to --]
    > Some text here.

cmftml-block-bold =
  .title = Bold
  .info =
    Bolds the enclosed text.
    Using {"{{"}[[bold]]{"}}"} is the same as using {"{{"}@<**bold**>@{"}}"},
    with the difference only being the syntax used.
  .example =
    Some [[b]]text![[/b]]
    \[!-- equivalent to --]
    Some **text!**

cmftml-block-char =
  .title = Char
  .info =
    Renders an HTML entity (a special character) in place of the block.

    Note: The syntax for HTML entities is simplified for convinence. Normally,
    you would need to write {"{{"}&copy;{"}}"} to display the copyright symbol in HTML, but
    with the {"{{"}[[char]]{"}}"} block you just need to write {"{{"}copy{"}}"}.

    \[[[https://dev.w3.org/html5/html-author/charref | List of HTML entities]]]
  .example =
    This file is [[char copy]] 2021 Team Wikijump.

cmftml-block-checkbox =
  .title = Checkbox
  .info =
    Creates an interactive checkbox input that can be checked or unchecked.

    Providing the {"{{"}*{"}}"} prefix sets the checkbox to start checked.
  .example =
    \[[checkbox Apple]]
    \[[*checkbox Blueberry]]
    \[[checkbox Cherry]]
    \[[checkbox Durian]]

cmftml-block-code =
  .title = Code
  .info =
    Renders the enclosed text raw, with all whitespace preserved.
    Intended for use with programming code and the like.

    Providing the {"{{"}type{"}}"} argument sets the enclosed text to be
    rendered with syntax highlighting, if the specified language
    is available.
  .example =
    \[[code]]
    This text is **not** rendered as Wikitext, but output as-is!
    \[[/code]]
    \[!-- or, with syntax highlighting --]
    \[[code type="javascript"]]
    console.log("foo"); // syntax highlighted as JS!
    \[[/code]]

cmftml-block-collapsible =
  .title = Collapsible
  .info =
    Creates an interactive block that can be expanded or folded,
    and when expanded, will display its contents.
  .example =
    \[[collapsible
      show="+ Spoilers for Ouroboros"
      hide="- Spoilers!"
      hideLocation="bottom"
    ]]
    Overseers die.
    \[[/collapsible]]

cmftml-block-css =
  .title = CSS
  .info =
    Adds a CSS stylesheet to the page, using the enclosed text as the source.
  .example =
    \[[css]]
    #page-title {"{"}
      color: purple;
    {"}"}
    \[[/css]]

cmftml-block-del =
  .title = Deletion
  .info =
    Denotes that the enclosed text has been "deleted" from the document.
    The text is not literally deleted, instead, it is styled or otherwise
    set to appear as deleted.

    Depending on context, this may have different meanings. In the context
    of a story, the deleted text may be a storytelling element, representing
    something diagetically deleted from a document. In a programming context,
    it may represent code that was deleted from a file, e.g. in a diff.
  .example =
    I [[del]]don't[[/del]] like that haircut.

cmftml-block-div =
  .title = Div
  .info =
    Creates a generic block container element,
    which may be styled, given a class/ID, etc.
  .example =
    \[[div_ class="blockquote" style="border: none;"]]
    Some text __here!__
    \[[/div]]

cmftml-block-footnote =
  .title = Footnote
  .info =
    Adds an expandable footnote, which a reader may use to view the contents by of
    hovering or focusing on a numbered marker. The footnote's contents will also be
    displayed in a "footnote block" at (by default) the bottom of the page.
  .example =
    The author of The Dark Tower series[[footnote]]Did you know that world-renowned writer Stephen King was once hit by a car? Just something to consider.[[/footnote]] began work in the late 1970s.

cmftml-block-footnote-block =
  .title = Footnote Block
  .info =
    Moves the footnote block. The "footnote block" is added to a page whenever it has
    any {"{{"}[[footnote]]{"}}"} elements, and by default it's placed at the bottom of the
    page. This block lets you move it, if so desired. If having a footnote block at
    all is undesirable, you can use the {"{{"}hide{"}}"} argument to remove it.
  .example =
    \[[footnote-block title="Endnotes"]]
    \[[footnote-block title="Stephen King Car Crash Facts"]]

cmftml-block-hidden =
  .title = Hidden
  .info =
    Styles the enclosed block of text as being hidden.
    This block does not delete or otherwise prevent the enclosed text from
    being rendered, instead, it sets the text to be invisible. A reader who
    is persistent enough will be able to find the text.
  .example =
    This text is **visible**.
    \[[hidden]]
    This text is not.
    \[[/hidden]]

cmftml-block-html =
  .title = HTML
  .info =
    Renders the enclosed block of text in an {"{{"}<iframe>{"}}"} element.
    In other words, it allows the insertion of arbitrary HTML markup in a safe way.
    This can be used to embed widgets or even games.
  .example =
    \[[html]]
    <h2>Exciting!</h2>
    <p>
    This HTML will appear in an iframe hosted on wjfiles!
    </p>
    \[[/html]]

cmftml-block-ifcategory =
  .title = ifcategory
  .info =
    Sets the enclosed text to be conditionally rendered depending on if the
    page is in, or is not in, certain categories.

    Syntax:

    \* {"{{"}+<category>{"}}"} Requires that the page is in the category.

    \* {"{{"}-<category>{"}}"} Requires that the page //is not// in the category.

    Additionally, the {"{{"}+{"}}"} prefix can be omitted, as in {"{{"}<category>{"}}"}.
    The {"{{"}+{"}}"} is assumed if no other prefix is present.

    Finally, as a page cannot be in more than one category at once, having multiple
    {"{{"}+<category>{"}}"} statements acts as an {"{{"}OR{"}}"} conditional. In other words,
    the page has to be //in one of// the listed (positive) categories.
  .example =
    \[[ifcategory _default]]
    Will render if the page is in the default category.
    \[[/ifcategory]]

    \[[ifcategory +_default +component -fragment]]
    This will render if the page is in either the _default or component categories.
    But if the page is in the fragment category, this won't render.
    \[[/ifcategory]]

    \[[ifcategory -_default]]
    Will only render if the page is in a category that isn't the default.
    \[[/ifcategory]]

cmftml-block-iftags =
  .title = iftags
  .info =
    Sets the enclosed text to be conditionally rendered depending on if the
    page does or does not have certain tags.

    Syntax:

    \* {"{{"}<tag>{"}}"} Requires that this tag, or any other {"{{"}<tag>{"}}"}, is present.

    \* {"{{"}+<tag>{"}}"} Requires that the tag is present.

    \* {"{{"}-<tag>{"}}"} Requires that the tag //is not// present.

    To explain the first syntax ({"{{"}<tag>{"}}"}) further, all {"{{"}<tag>{"}}"} statements are
    "lumped together" and checked. If _any_ of them are present, this check passes.
    This is exactly like an {"{{"}OR{"}}"} conditional. This differs from how the {"{{"}+<tag>{"}}"}
    and {"{{"}-<tag>{"}}"} syntaxes behave, which demand that the tag is or is not present.
  .example =
    \[[iftags +science]]
    This page is labeled as: science.
    \[[/iftags]]

    \[[iftags +bug -fixed]]
    This is a bug, and it's not fixed yet.
    \[[/iftags]]

cmftml-block-iframe =
  .title = iframe
  .info =
    Creates an {"{{"}<iframe>{"}}"} element in place. This allows for the embedding of
    external content, given an URL to said content.
    This is similar to the {"{{"}[[html]]{"}}"} element.
  .example =
    My website:
    \[[iframe https://example.com/ class="website"]]

cmftml-block-image =
  .title = Image
  .info =
    Embeds an image in the rendered document.

    Accepts the following prefixes:

    \* {"{{"}={"}}"} Centers the image

    \* {"{{"}<{"}}"} Aligns the image to the left

    \* {"{{"}>{"}}"} Aligns the image to the right

    \* {"{{"}f<{"}}"} Floats the image to the left, allowing text to wrap around it

    \* {"{{"}f>{"}}"} Floats the image to the right, allowing text to wrap around it
  .example =
    \[[image green_apple.png alt="A green apple" title="Take a big bite!" style="width: 100%;" class="fruity"]]
    \[[image https://example.com/my-image.png]]
    \[[image /some-other-page/my-picture.jpeg]]
    \[[=image landscape.png]]
    \[[<image landscape.png]]
    \[[>image landscape.png]]
    \[[f<image landscape.png]]
    \[[f>image landscape.png]]
    \[[image filename.png link="#section"]]
    \[[image filename.png link="SCP-001"]]
    \[[image filename.png link="https://example.com/"]]

cmftml-block-include-elements =
  .title = Include (Elements)
  .info =
    Injects another page's contents. This lets you //include// something without
    copy-pasting it, which allows for reusable content, like pre-made headers and
    footers. Includes can accept arguments, and these arguments replace any
    placeholders of the same name when the page is injected.
  .example =
    \[[include-elements component:some-bar
      class="Keter"
      classification="4"
      taskforce="MTF-Eta-10 (\"See No Evil\")"
    ]]

cmftml-block-include-messy =
  .title = Include (Messy)
  .info =
    You shouldn't use this block unless you have to. This block takes another page's
    contents and literally pastes it directly into your page's source prior to
    rendering. The only reason it exists is for compatibility with Wikidot content.
    Use {"{{"}[[include-elements]]{"}}"} instead.
  .example =
    \[[include-messy theme:black-highlighter-theme]]

    \[[include-messy component:fancy-object-class
      class=Keter |
      classification=4 |
      taskforce=MTF-Eta-10 ("See No Evil")
    ]]

cmftml-block-ins =
  .title = Insertion
  .info =
    Denotes that the enclosed text has been "inserted" into the document.
    This does not insert the enclosed text in some special way, instead, it is
    styled or otherwise set to appear inserted.

    Depending on context, this may have different meanings. In the context
    of a story, the inserted text may be a storytelling element, representing
    something diagetically inserted into a document. In a programming context,
    it may represent new code that was inserted into a file, e.g. in a diff.
  .example =
    I would like some [[ins]]anchovy[[/ins]] pizza please, thank you.

cmftml-block-invisible =
  .title = Invisible
  .info =
    Styles the enclosed span of text as being invisible.
    This does not delete or otherwise prevent the enclosed text from
    being rendered, instead, it sets the text to be invisible. A reader who
    is persistent enough will be able to find the text.
  .example =
    This text appears [[invisible]]but still takes up space, and can be selected.[[/invisible]]

cmftml-block-italics =
  .title = Italics
  .info =
    Sets the enclosed text to be italicized.
    Using {"{{"}[[italics]]{"}}"} is the same as using {"{{"}@<//italics//>@{"}}"},
    with the difference only being the syntax used.
  .example =
    This text is regular, but [[em]]this text is emphasized[[/em]].
    \[!-- equivalent to --]
    This text is regular, but //this text is emphasized//.

cmftml-block-lines =
  .title = Lines
  .info =
    Adds a specified number of newlines to the document.
  .example =
    \[[newlines 4]]
    \[!-- Much easier than spamming "@@@@"s --]

cmftml-block-list-item =
  .title = List-Item
  .info =
    Creates an item for a list. A list-item can only be placed inside either a
    {"{{"}[[ul]]{"}}"} (unordered list) or {"{{"}[[ol]]{"}}"} (ordered list) block.
  .example =
    \[[ul]]
      \[[ol]]
        \[[li]] Item A [[/li]]
        \[[li]] Item B [[/li]]
      \[[/ol]]

      \[[li]] Item C [[/li]]
    \[[/ul]]

cmftml-block-list-ordered =
  .title = Ordered List
  .info =
    Starts an ordered list, which is a list that is sorted in order, e.g. with
    numbers or the alphabet. The children of this element should only be {"{{"}[[ul]]{"}}"}, {"{{"}[[ol]]{"}}"}, or {"{{"}[[li]]{"}}"} blocks.
  .example =
    \[[ul]]
      \[[ol]]
        \[[li]] Item A [[/li]]
        \[[li]] Item B [[/li]]
      \[[/ol]]

      \[[li]] Item C [[/li]]
    \[[/ul]]

cmftml-block-list-unordered =
  .title = Unordered List
  .info =
    Starts an unordered list. The children of this element should only be {"{{"}[[ul]]{"}}"}, {"{{"}[[ol]]{"}}"}, or {"{{"}[[li]]{"}}"} blocks.
  .example =
    \[[ul]]
      \[[ol]]
        \[[li]] Item A [[/li]]
        \[[li]] Item B [[/li]]
      \[[/ol]]

      \[[li]] Item C [[/li]]
    \[[/ul]]

cmftml-block-mark =
  .title = Mark
  .info =
    Marks/highlights the enclosed text.
  .example =
    This text is [[mark]]highlighted![[/mark]]

cmftml-block-monospace =
  .title = Monospace
  .info =
    Styles the enclosed text as being monospaced.
    This does not escape or otherwise affect the formatting of the text,
    it simply styles it.

    Using {"{{"}[[tt]]{"}}"} is the same as using {"{{"}@<{"{{"}monospace{"}}"}>@{"}}"},
    with the difference only being the syntax used.
  .example =
    \[[tt]]This looks like it came from a typewriter or computer terminal.[[/tt]]
    \[!-- equivalent to --]
    {"{{"}This looks like it came from a typewriter or computer terminal.{"}}"}

cmftml-block-paragraph =
  .title = Paragraph
  .info =
    Creates a paragraph element. This is normally done automatically, using the flow
    of linebreaks and blocks to determine where paragraphs should be placed.
    However, using this block allows avoiding the need to use {"{{"}[[div]]{"}}"} or
    {"{{"}[[span]]{"}}"} blocks as wrappers.
  .example =
    \[[p class="fruits"]]
    Apple, Banana, Cherry!
    \[[/p]]

cmftml-block-radio =
  .title = Radio
  .info =
    Creates an interactive radio-button input that can be selected or unselected.
    The value given for the block sets the radio-button's group.

    Providing the {"{{"}*{"}}"} prefix sets the radio-button to start selected.
  .example =
    Favorite kind of music:
    \[[radio music]] Disco
    \[[radio music]] Dance
    \[[radio music]] Rap
    \[[*radio music]] Noise

cmftml-block-size =
  .title = Size
  .info =
    Sets the font-size of the enclosed text. Uses CSS units, e.g. {"{{"}em{"}}"} or {"{{"}px{"}}"}.
  .example =
    This text is regular, but [[size 250%]]this text is much larger[[/size]].

cmftml-block-span =
  .title = Span
  .info =
    Creates a generic inline container element,
    which may be styled, given a class/ID, etc.
  .example =
    This text is in a span:
    \[[span class="fruit"]]banana[[/span]]

cmftml-block-strikethrough =
  .title = Strikethrough
  .info =
    Sets the enclosed text to be strucken.
    Using {"{{"}[[s]]{"}}"} is the same as using {"{{"}@<--strikethrough-->@{"}}"},
    with the difference only being the syntax used.
  .example =
    This text is [[s]]struck through![[/s]]
    \[!-- equivalent to --]
    This text is --struck through!--

cmftml-block-subscript =
  .title = Subscript
  .info =
    Sets the enclosed text as being a subscript.
    Using {"{{"}[[sub]]{"}}"} is the same as using {"{{"}@<,,subscript,,>@{"}}"},
    with the difference only being the syntax used.
  .example =
    Let this variable be called x[[sub]]A[[/sub]].
    \[!-- equivalent to --]
    Let this variable be called x,,A,,.

cmftml-block-superscript =
  .title = Superscript
  .info =
    Sets the enclosed text as being a superscript.
    Using {"{{"}[[sup]]{"}}"} is the same as using {"{{"}@<^^sup^^>@{"}}"},
    with the difference only being the syntax used.
  .example =
    Thus, the result is n[[sup]]2[[/sup]].
    \[!-- equivalent to --]
    Thus, the result is n^^2^^.

cmftml-block-table =
  .title = Table
  .info =
    Starts a table. Tables must follow a specific structure, with a {"{{"}[[table]]{"}}"}
    containing only {"{{"}[[row]]{"}}"} blocks, and those containing only {"{{"}[[cell]]{"}}"} or
    {"{{"}[[hcell]]{"}}"} blocks. A cell can contain generic content, including other tables.
  .example =
    \[[table]]
      \[[row]]
        \[[hcell]] Name [[/hcell]]
        \[[hcell]] Price [[/hcell]]
        \[[hcell]] Stock [[/hcell]]
      \[[/row]]
      \[[row]]
        \[[cell]] Banana [[/cell]]
        \[[cell]] $0.30 [[/cell]]
        \[[cell]] 87 [[/cell]]
      \[[/row]]
    \[[/table]]

cmftml-block-table-row =
  .title = Table Row
  .info =
    Creates a row in a table. Tables must follow a specific structure, with a
    {"{{"}[[table]]{"}}"} containing only {"{{"}[[row]]{"}}"} blocks, and those containing only
    {"{{"}[[cell]]{"}}"} or {"{{"}[[hcell]]{"}}"} blocks. A cell can contain generic content,
    including other tables.
  .example =
    \[[table]]
      \[[row]]
        \[[hcell]] Name [[/hcell]]
        \[[hcell]] Price [[/hcell]]
        \[[hcell]] Stock [[/hcell]]
      \[[/row]]
      \[[row]]
        \[[cell]] Banana [[/cell]]
        \[[cell]] $0.30 [[/cell]]
        \[[cell]] 87 [[/cell]]
      \[[/row]]
    \[[/table]]

cmftml-block-table-cell-regular =
  .title = Table Cell
  .info =
    Creates a cell in a table row. Tables must follow a specific structure, with a
    {"{{"}[[table]]{"}}"} containing only {"{{"}[[row]]{"}}"} blocks, and those containing only
    {"{{"}[[cell]]{"}}"} or {"{{"}[[hcell]]{"}}"} blocks. A cell can contain generic content,
    including other tables.
  .example =
    \[[table]]
      \[[row]]
        \[[hcell]] Name [[/hcell]]
        \[[hcell]] Price [[/hcell]]
        \[[hcell]] Stock [[/hcell]]
      \[[/row]]
      \[[row]]
        \[[cell]] Banana [[/cell]]
        \[[cell]] $0.30 [[/cell]]
        \[[cell]] 87 [[/cell]]
      \[[/row]]
    \[[/table]]

cmftml-block-table-cell-header =
  .title = Table Cell (Header)
  .info =
    Creates a cell in a table row, styled as a header for a column. Tables must
    follow a specific structure, with a {"{{"}[[table]]{"}}"} containing only {"{{"}[[row]]{"}}"}
    blocks, and those containing only {"{{"}[[cell]]{"}}"} or {"{{"}[[hcell]]{"}}"} blocks. A cell
    can contain generic content, including other tables.
  .example =
    \[[table]]
      \[[row]]
        \[[hcell]] Name [[/hcell]]
        \[[hcell]] Price [[/hcell]]
        \[[hcell]] Stock [[/hcell]]
      \[[/row]]
      \[[row]]
        \[[cell]] Banana [[/cell]]
        \[[cell]] $0.30 [[/cell]]
        \[[cell]] 87 [[/cell]]
      \[[/row]]
    \[[/table]]

cmftml-block-toc =
  .title = Table of Contents
  .info =
    Render's a table of contents for the page, based off the heading elements found
    in the page. This lets readers quickly navigate your page.
  .example =
    \[[toc]]

    + Part 1: The Part You Wish You Could Skip

    + Part 2: The Good Stuff

cmftml-block-user =
  .title = User
  .info =
    Renders a user-info widget, which will link to and
    display info about the specified user.

    Providing a {"{{"}*{"}}"} prefix displays the user's avatar and karma as well.
  .example =
    \[[user xXx-epic-dude-xXx]]

cmftml-block-underline =
  .title = Underline
  .info =
    Underlines the enclosed text.
    Using {"{{"}[[u]]{"}}"} is the same as using {"{{"}@<__underline__>@{"}}"},
    with the difference only being the syntax used.
  .example =
    \[[u]]Testing log 7192-45:[[/u]]
    \[!-- equivalent to --]
    __Testing log 7192-45:__
