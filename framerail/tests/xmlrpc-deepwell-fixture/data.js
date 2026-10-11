/**
 * @typedef {{
 *   compiled_html?: boolean
 *   wikitext?: boolean
 * }} PageDetails
 *
 *
 * @typedef {{
 *   compiled_body_html: string
 *   compiled_body_styles?: string[]
 *   creator_user_id: number
 *   page_created_at: string
 *   page_id: number
 *   page_revision_count: number
 *   page_updated_at: string | null
 *   rating: number
 *   revision_created_at: string
 *   revision_id: number
 *   revision_user_id: number
 *   slug: string
 *   tags: string[]
 *   title: string
 *   wikitext: string
 * }} FixturePage
 *
 *
 * @typedef {{
 *   content: string
 *   created_at: string
 *   created_by: string
 *   html: string
 *   id: number
 *   reply_to: number | null
 *   title: string
 * }} FixtureForumPost
 */

/** @type {Record<string, FixturePage>} */
export const pages = {
  main: {
    page_id: 3000001,
    revision_id: 9000001,
    page_created_at: "2008-07-19T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2008-07-19T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Main",
    slug: "main",
    tags: [],
    rating: 0,
    wikitext: "Main",
    compiled_body_html: "<p>Main</p>"
  },
  "data-form-date-options-flow:example": {
    page_id: 3000400,
    revision_id: 9000400,
    page_created_at: "2026-08-16T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-16T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Data Form Date Options",
    slug: "data-form-date-options-flow:example",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      "date-primary: 1411099200\ndate-secondary: 1418360400\nalt-date: 12/11/2014",
    compiled_body_html: "<p>Data form date options fixture.</p>"
  },
  "scp-173": {
    page_id: 3000173,
    revision_id: 9000173,
    page_created_at: "2008-07-26T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 3,
    revision_created_at: "2008-07-26T00:00:00Z",
    revision_user_id: 456,
    creator_user_id: 123,
    title: "SCP-173",
    slug: "scp-173",
    tags: ["scp", "euclid"],
    rating: 173,
    wikitext: "**Item #:** SCP-173",
    compiled_body_html: "<p><strong>Item #:</strong> SCP-173</p>"
  },
  "scp-173-parent": {
    page_id: 3000172,
    revision_id: 9000172,
    page_created_at: "2008-07-25T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2008-07-25T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "SCP Foundation",
    slug: "scp-173-parent",
    tags: ["hub"],
    rating: 1,
    wikitext: "Parent",
    compiled_body_html: "<p>Parent</p>"
  },
  "private-page": {
    page_id: 3000199,
    revision_id: 9000199,
    page_created_at: "2026-07-01T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-01T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Private Page",
    slug: "private-page",
    tags: ["private"],
    rating: 0,
    wikitext: "Private page body marker.",
    compiled_body_html: "<p>Private page body marker.</p>"
  },
  "public-child-private-parent": {
    page_id: 3000197,
    revision_id: 9000197,
    page_created_at: "2026-07-02T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-02T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Public Child",
    slug: "public-child-private-parent",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Public child body marker.",
    compiled_body_html: "<p>Public child body marker.</p>"
  },
  "ambiguous-parent-child": {
    page_id: 3000196,
    revision_id: 9000196,
    page_created_at: "2026-07-03T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-03T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Ambiguous Parent Child",
    slug: "ambiguous-parent-child",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Ambiguous parent child body marker.",
    compiled_body_html: "<p>Ambiguous parent child body marker.</p>"
  },
  "xmlrpc-post-page": {
    page_id: 3000300,
    revision_id: 9000300,
    page_created_at: "2026-06-20T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-06-20T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "XML-RPC Post Page",
    slug: "xmlrpc-post-page",
    tags: ["fixture"],
    rating: 5,
    wikitext: "XML-RPC post fixture page.",
    compiled_body_html: "<p>XML-RPC post fixture page.</p>"
  },
  "theme:yossistyle": {
    page_id: 3000310,
    revision_id: 9000310,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "YOSSISTYLE",
    slug: "theme:yossistyle",
    tags: ["theme"],
    rating: 0,
    wikitext:
      "[[module CSS]]\n#header h2 span { margin-left: 1px; }\n[[/module]]\nXML-RPC theme body marker.",
    compiled_body_html: "<p>XML-RPC theme body marker.</p>",
    compiled_body_styles: [
      "body { font-family: verdana, arial, helvetica, sans-serif; font-size: 0.8em; }",
      "#header h2 span { margin-left: 1px; }"
    ]
  },
  "wikidot-tabview": {
    page_id: 3000320,
    revision_id: 9000320,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Tabview",
    slug: "wikidot-tabview",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      "[[tabview]]\n[[tab First]]First panel[[/tab]]\n[[tab Second]]Second panel[[/tab]]\n[[/tabview]]",
    compiled_body_html:
      '<script type="text/javascript" src="http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--javascript/yahooui/tabview-min.js"></script>\n<div id="wiki-tabview-0123456789abcdef0123456789abcdef" class="yui-navset"><ul class="yui-nav"><li class="selected"><a href="javascript:;"><em>First</em></a></li><li><a href="javascript:;"><em>Second</em></a></li></ul><div class="yui-content"><div id="wiki-tab-0-0" style="display: block;"><p>First panel</p></div><div id="wiki-tab-0-1" style="display: none;"><p>Second panel</p></div></div></div>'
  },
  "wikidot-toc": {
    page_id: 3000321,
    revision_id: 9000321,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Table of Contents",
    slug: "wikidot-toc",
    tags: ["fixture"],
    rating: 0,
    wikitext: "+ First heading\n+ Second heading\n\n[[toc]]\n[[f>toc]]",
    compiled_body_html:
      '<h1 id="toc0"><span>First heading</span></h1><h1 id="toc1"><span>Second heading</span></h1><table style="margin:0; padding:0"><tr><td style="margin:0; padding:0"><div id="toc"><div id="toc-action-bar"><a href="javascript:;" onclick="WIKIDOT.page.listeners.foldToc(event)">Fold</a><a style="display: none" href="javascript:;" onclick="WIKIDOT.page.listeners.unfoldToc(event)">Unfold</a></div><div class="title">Table of Contents</div><div id="toc-list"><div style="margin-left: 1em;"><a href="#toc0">First heading</a></div><div style="margin-left: 1em;"><a href="#toc1">Second heading</a></div></div></div></td></tr></table><div id="toc" class="floatright"><div id="toc-action-bar"><a href="javascript:;" onclick="WIKIDOT.page.listeners.foldToc(event)">Fold</a><a style="display: none" href="javascript:;" onclick="WIKIDOT.page.listeners.unfoldToc(event)">Unfold</a></div><div class="title">Table of Contents</div><div id="toc-list"><div style="margin-left: 1em;"><a href="#toc0">First heading</a></div><div style="margin-left: 1em;"><a href="#toc1">Second heading</a></div></div></div>'
  },
  "wikidot-footnotes": {
    page_id: 3000326,
    revision_id: 9000326,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Footnotes",
    slug: "wikidot-footnotes",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      "Reference one.[[footnote]]Note one.[[/footnote]] Reference two.[[footnote]]Note two.[[/footnote]]\n\n[[footnoteblock]]",
    compiled_body_html:
      '<p>Reference one.<sup class="footnoteref"><a id="footnoteref-1" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference(\'footnote-1\')">1</a></sup> Reference two.<sup class="footnoteref"><a id="footnoteref-2" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference(\'footnote-2\')">2</a></sup></p><div class="footnotes-footer"><div class="footnote-footer" id="footnote-1"><a href="javascript:;" onclick="WIKIDOT.page.utils.scrollToReference(\'footnoteref-1\')">1</a>. Note one.</div><div class="footnote-footer" id="footnote-2"><a href="javascript:;" onclick="WIKIDOT.page.utils.scrollToReference(\'footnoteref-2\')">2</a>. Note two.</div></div><div style="height: 1200px"></div><a id="ordinary-javascript-link" href="javascript:;" onclick="window.__unexpectedFootnoteAction = true">unrelated</a>'
  },
  "search:all": {
    page_id: 3000325,
    revision_id: 9000325,
    page_created_at: "2026-08-09T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-09T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Search All",
    slug: "search:all",
    tags: ["fixture"],
    rating: 0,
    wikitext: "[[module SearchAll]]",
    compiled_body_html:
      '<div class="search-box"><div class="query-area"><form action="dummy" id="search-form-all"><div><input class="text" type="text" size="30" name="query" id="search-form-all-input" value="" aria-label="Search all Wikis"><input class="button" type="submit" value="Search"></div><div style="margin-top:5px;"><input id="search-all-pf" class="radio" type="radio" name="area" value="pf" checked="checked"><label for="search-all-pf">pages and forums</label><input id="search-all-p" class="radio" type="radio" name="area" value="p"><label for="search-all-p">pages only</label><input id="search-all-f" class="radio" type="radio" name="area" value="f"><label for="search-all-f">forums only</label></div></form></div><div class="search-results"></div></div>'
  },
  "platform:search": {
    page_id: 3000327,
    revision_id: 9000327,
    page_created_at: "2026-08-09T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-09T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikijump: Search all sites",
    slug: "platform:search",
    tags: ["fixture"],
    rating: 0,
    wikitext: "[[module SearchAll]]",
    compiled_body_html:
      '<div class="search-box"><div class="query-area"><form action="dummy" id="search-form-all"><div><input class="text" type="text" size="30" name="query" id="search-form-all-input" value="" aria-label="Search all Wikis"><input class="button" type="submit" value="Search"></div><div style="margin-top:5px;"><input id="search-all-pf" class="radio" type="radio" name="area" value="pf" checked="checked"><label for="search-all-pf">pages and forums</label><input id="search-all-p" class="radio" type="radio" name="area" value="p"><label for="search-all-p">pages only</label><input id="search-all-f" class="radio" type="radio" name="area" value="f"><label for="search-all-f">forums only</label></div></form></div><div class="search-results"></div></div>'
  },
  "search:site": {
    page_id: 3000326,
    revision_id: 9000326,
    page_created_at: "2026-08-09T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-09T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Search",
    slug: "search:site",
    tags: ["fixture"],
    rating: 0,
    wikitext: "[[module Search]]",
    compiled_body_html:
      '<div class="error-block">Search is temporarily unavailable, we are working to bring it online!</div>'
  },
  "wikidot-collapsible": {
    page_id: 3000330,
    revision_id: 9000330,
    page_created_at: "2026-07-22T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-22T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Collapsible",
    slug: "wikidot-collapsible",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      '[[collapsible show="+ Show" hide="- Hide" hideLocation="both"]]Folded body[[/collapsible]]\n[[collapsible folded="no" show="+ Open" hide="- Close"]]Open body[[/collapsible]]',
    compiled_body_html:
      '<div id="folded-collapsible" class="collapsible-block"><div class="collapsible-block-folded"><a class="collapsible-block-link" href="javascript:;">+&nbsp;Show</a></div><div class="collapsible-block-unfolded" style="display:none"><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Hide</a></div><div class="collapsible-block-content"><p>Folded body</p></div><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Hide</a></div></div></div><div id="open-collapsible" class="collapsible-block"><div class="collapsible-block-folded" style="display:none"><a class="collapsible-block-link" href="javascript:;">+&nbsp;Open</a></div><div class="collapsible-block-unfolded"><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Close</a></div><div class="collapsible-block-content"><p>Open body</p></div></div></div><details id="native-collapsible"><summary>Native summary</summary><p>Native body</p></details>'
  },
  "wikidot-code-highlighting": {
    page_id: 3000350,
    revision_id: 9000350,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Code Highlighting",
    slug: "wikidot-code-highlighting",
    tags: ["fixture"],
    rating: 0,
    wikitext: '[[code type="css"]]\n#header h2 span { color: red; }\n[[/code]]',
    compiled_body_html:
      '<div class="code" data-wj-language="css"><pre><code>#header h2 span { color: red; }</code></pre></div>'
  },
  "wikidot-code-math": {
    page_id: 3000355,
    revision_id: 9000355,
    page_created_at: "2026-08-05T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-05T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Code and Math",
    slug: "wikidot-code-math",
    tags: ["fixture"],
    rating: 0,
    wikitext: String.raw`[[code language="rust"]]
fn main() { println!("oracle"); }
[[/code]]
[[math]]
x^2 + y^2 = z^2
[[/math]]`,
    compiled_body_html: String.raw`<div class="code"><pre><code>fn main() { println!(&quot;oracle&quot;); }</code></pre></div><br>
<span class="equation-number">(1)</span>
<div class="math-equation" id="equation-1">\begin{equation} x^2 + y^2 = z^2 \end{equation}</div>`
  },
  "page-workflow-probe": {
    page_id: 3000340,
    revision_id: 9000340,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Page Workflow Probe",
    slug: "page-workflow-probe",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Page workflow probe",
    compiled_body_html:
      '<p>Page workflow probe</p><a id="navigate-history-target" href="/scp-173" data-sveltekit-reload="off">Navigate to history target</a>'
  },
  "page-workflow-star-probe": {
    page_id: 3000341,
    revision_id: 9000341,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Page Workflow Star Probe",
    slug: "page-workflow-star-probe",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Page workflow star probe",
    compiled_body_html: "<p>Page workflow star probe</p>"
  },
  "page-workflow-rating-probe": {
    page_id: 3000342,
    revision_id: 9000342,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Page Workflow Rating Probe",
    slug: "page-workflow-rating-probe",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Page workflow rating probe",
    compiled_body_html: "<p>Page workflow rating probe</p>"
  },
  "authoring-history-probe": {
    page_id: 3000345,
    revision_id: 9000345,
    page_created_at: "2026-07-24T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 3,
    revision_created_at: "2026-07-24T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Authoring History Probe",
    slug: "authoring-history-probe",
    tags: ["fixture"],
    wikitext: "Authoring history probe",
    compiled_body_html:
      '<p>Authoring history probe</p><a id="navigate-history-target" href="/scp-173" data-sveltekit-reload="off">Navigate to history target</a>'
  },
  "navigation-style-a": {
    page_id: 3000360,
    revision_id: 9000360,
    page_created_at: "2026-07-27T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-27T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Navigation Style A",
    slug: "navigation-style-a",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Navigation style A",
    compiled_body_html:
      '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1&amp;css=.styleframe-a%7Bcolor%3Ared%7D"></iframe><a id="navigate-style-b" href="/navigation-style-b">Navigate to B</a>',
    compiled_body_styles: [".generated-style-a { color: red; }"]
  },
  "navigation-style-b": {
    page_id: 3000370,
    revision_id: 9000370,
    page_created_at: "2026-07-27T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-27T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Navigation Style B",
    slug: "navigation-style-b",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Navigation style B",
    compiled_body_html:
      '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=2&amp;theme=%2Fnavigation-style-b-theme.css&amp;css=%23page-title%7Bdisplay%3Anone%7D"></iframe><a id="navigate-style-a" href="/navigation-style-a">Navigate to A</a>',
    compiled_body_styles: [
      ".generated-style-b-one { color: blue; }",
      ".generated-style-b-two { color: green; }"
    ]
  },
  "navigation-style-c": {
    page_id: 3000380,
    revision_id: 9000380,
    page_created_at: "2026-07-27T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-27T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Navigation Style C",
    slug: "navigation-style-c",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Navigation style C",
    compiled_body_html:
      '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=3&amp;css=.styleframe-c%7Bcolor%3Apurple%7D"></iframe><a id="navigate-style-d" href="/navigation-style-d">Navigate to D</a>',
    compiled_body_styles: [
      ".generated-style-c-one { color: purple; }",
      ".generated-style-c-two { color: orange; }"
    ]
  },
  "navigation-style-d": {
    page_id: 3000390,
    revision_id: 9000390,
    page_created_at: "2026-07-27T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-27T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Navigation Style D",
    slug: "navigation-style-d",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Navigation style D",
    compiled_body_html:
      '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=4&amp;css=.styleframe-d%7Bcolor%3Ablack%7D"></iframe>',
    compiled_body_styles: [".generated-style-d { color: black; }"]
  },
  "navigation-style-duplicate": {
    page_id: 3000395,
    revision_id: 9000395,
    page_created_at: "2026-08-12T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-12T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Navigation Style Duplicate",
    slug: "navigation-style-duplicate",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Navigation style duplicate",
    compiled_body_html:
      '<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1&amp;theme=%2Fnavigation-style-duplicate-theme.css"></iframe><div id="cascade-probe">Cascade probe</div>',
    compiled_body_styles: [
      '@import url("/navigation-style-duplicate-theme.css"); #cascade-probe { color: rgb(0, 0, 255); }'
    ]
  },
  "listpages-navigation": {
    page_id: 3000400,
    revision_id: 9000400,
    page_created_at: "2026-07-28T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-28T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "ListPages Navigation",
    slug: "listpages-navigation",
    tags: ["fixture"],
    rating: 0,
    wikitext: '[[module ListPages perPage="1"]]%%title%%[[/module]]',
    compiled_body_html: "<p>ListPages navigation fixture.</p>"
  },
  "newpage-helper": {
    page_id: 3000410,
    revision_id: 9000410,
    page_created_at: "2026-07-29T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-29T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "NewPage Helper",
    slug: "newpage-helper",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      '[[module NewPage button="Default create"]]\n[[module NewPage mode="save-and-go" tags="alpha beta" parent="main" button="Autosave"]]\n[[module NewPage template="template:fixture-newpage-template-a" tags="alpha beta" parent="main" button="Template"]]',
    compiled_body_html:
      '<div id="default-newpage" class="new-page-box" style="text-align: center; margin: 1em 0;"><form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event);"><input class="text" name="pageName" type="text" size="30" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/><input type="submit" class="button" value="Default create" style="margin: 1px;"/></form></div><div id="autosave-newpage" class="new-page-box" style="text-align: center; margin: 1em 0;"><form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event);"><input class="text" name="pageName" type="text" size="30" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/><input type="submit" class="button" value="Autosave" style="margin: 1px;"/><input type="hidden" name="mode" value="save-and-go"/><input type="hidden" name="tags" value="alpha beta"/><input type="hidden" name="parent" value="main"/></form></div><div id="template-newpage" class="new-page-box" style="text-align: center; margin: 1em 0;"><form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event);"><input class="text" name="pageName" type="text" size="30" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/><input type="submit" class="button" value="Template" style="margin: 1px;"/><input type="hidden" name="template" value="1469068384"/><input type="hidden" name="tags" value="alpha beta"/><input type="hidden" name="parent" value="main"/></form></div>'
  },
  "gallery-lightbox": {
    page_id: 3000415,
    revision_id: 9000415,
    page_created_at: "2026-08-19T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-08-19T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Gallery Lightbox",
    slug: "gallery-lightbox",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      '[[gallery]]\n: first.png\n: second.png\n[[/gallery]]\n\n[[gallery viewer="false"]]\n: disabled.png\n[[/gallery]]\n\n[[gallery]]\n: broken.png\n[[/gallery]]',
    compiled_body_html:
      '<div class="gallery-box" id="gallery-box-1"><div class="gallery-item thumbnail"><table><tbody><tr><td><a id="gallery-viewer-one" href="/gallery-image-one.webp" class="with-lb" title="First image"><img src="/gallery-thumb-one.webp" alt="first" class="gallery-image-size-thumbnail"/></a></td></tr></tbody></table></div><div class="gallery-item thumbnail"><table><tbody><tr><td><a id="gallery-viewer-two" href="/gallery-image-two.webp" class="with-lb" title="Second image"><img src="/gallery-thumb-two.webp" alt="second" class="gallery-image-size-thumbnail"/></a></td></tr></tbody></table></div><div class="gallery-item thumbnail"><table><tbody><tr><td><a id="gallery-custom-link" href="/main"><img src="/gallery-thumb-custom.webp" alt="custom" class="gallery-image-size-thumbnail"/></a></td></tr></tbody></table></div></div><div class="gallery-box" id="gallery-box-2"><div class="gallery-item thumbnail"><table><tbody><tr><td><a id="gallery-viewer-disabled" href="/gallery-image-disabled.webp" class="with-lb"><img src="/gallery-thumb-disabled.webp" alt="disabled" class="gallery-image-size-thumbnail"/></a></td></tr></tbody></table></div></div><div class="gallery-box" id="gallery-box-3"><div class="gallery-item thumbnail"><table><tbody><tr><td><a id="gallery-viewer-failure" href="/gallery-image-broken.webp" class="with-lb"><img src="/gallery-thumb-broken.webp" alt="broken" class="gallery-image-size-thumbnail"/></a></td></tr></tbody></table></div></div>'
  },
  "data-form-edit-flow:example": {
    page_id: 3000420,
    revision_id: 9000420,
    page_created_at: "2026-07-29T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-29T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Example",
    slug: "data-form-edit-flow:example",
    tags: [],
    rating: 0,
    wikitext: "name: 'Probe Name'\nchoice: a",
    compiled_body_html:
      '<table class="form-table"><tbody><tr class="form-row"><td class="form-labels"><span class="form-label">Name</span></td><td class="form-values"><span>Probe Name</span></td></tr><tr class="form-row"><td class="form-labels"><span class="form-label">Choice</span></td><td class="form-values"><span>Alpha</span></td></tr></tbody></table>'
  }
}

/** @param {FixturePage} page */
export const toArticleViewResult = (page, siteId = 6000005) => ({
  site: {
    site_id: siteId,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: null,
    deleted_at: null,
    from_wikidot: false,
    slug: siteId === 6000006 ? "www" : "scp-wiki",
    name: siteId === 6000006 ? "Wikijump" : "SCP Foundation",
    tagline: siteId === 6000006 ? "Fighting Ozone Pollution" : "Secure, Contain, Protect",
    description: "Fixture site",
    locale: "en",
    default_page: "main",
    top_bar_page: null,
    side_bar_page: null,
    preferred_domain: null,
    layout: siteId === 6000006 ? "wikijump" : "wikidot",
    license: "cc-by-sa-3.0"
  },
  site_file_domain:
    siteId === 6000006 ? "www.wjfiles.localhost" : "scp-wiki.wjfiles.localhost",
  license_name: "CC BY-SA 3.0",
  license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
  user_session: null,
  article_page_cache_key: `deepwell:article-view:page:v1:site=${siteId}:page=${page.page_id}:rev=${page.revision_id}:updated=0:permission=site=0,user=0:body=fixture`,
  public_content_cache_fence: "0",
  anonymous_permission_cache_fence: "site=0,user=0",
  page: {
    type: "found",
    data: {
      options: {
        edit: false,
        title: null,
        parent: null,
        tags: null,
        no_redirect: false,
        no_render: false,
        debug: false,
        renderer: false,
        comments: false,
        history: false,
        offset: null,
        data: ""
      },
      redirect_page: page.redirect_page ?? null,
      wikitext: page.wikitext,
      compiled_body_html: page.compiled_body_html,
      compiled_body_styles: page.compiled_body_styles ?? [],
      compiled_top_bar_html: null,
      compiled_side_bar_html: null,
      page: {
        page_id: page.page_id,
        created_at: page.page_created_at,
        updated_at: page.page_updated_at,
        deleted_at: null,
        from_wikidot: false,
        site_id: siteId,
        latest_revision_id: page.revision_id,
        page_category_id: 1,
        slug: page.slug,
        discussion_thread_id: null,
        layout: siteId === 6000006 ? "wikijump" : "wikidot"
      },
      page_revision: {
        revision_id: page.revision_id,
        revision_type: "create",
        created_at: page.revision_created_at,
        updated_at: null,
        revision_number: page.page_revision_count - 1,
        page_id: page.page_id,
        site_id: siteId,
        user_id: page.revision_user_id,
        from_wikidot: false,
        changes: [],
        wikitext_hash: [],
        compiled_body_html_hash: [],
        compiled_top_bar_html_hash: null,
        compiled_side_bar_html_hash: null,
        compiled_at: page.revision_created_at,
        compiled_generator: "fixture",
        comments: "",
        hidden: [],
        title: page.title,
        alt_title: null,
        slug: page.slug,
        tags: page.tags
      },
      wikidot_snapshot: null,
      wikidot_breadcrumbs: [],
      attributions: [],
      page_rating: {
        enabled: true,
        permission: "registered",
        visibility: "visible",
        rating_type: page.slug === "page-workflow-star-probe" ? "stars" : "plus_minus"
      },
      page_discussion: { enabled: false },
      data_form: null,
      legacy_actions: [],
      rate_actions: null,
      membership_actions: [],
      meta_tags: []
    }
  }
})

/** @type {Record<string, FixtureForumPost[]>} */
export const forumPostsByPage = {
  "xmlrpc-integer-post-page": [
    {
      id: 7000301,
      reply_to: null,
      title: "XML-RPC integer ID proof",
      content: "XML-RPC integer post ID proof body.",
      html: "<p>XML-RPC integer post ID proof body.</p>",
      created_by: "administrator",
      created_at: "2026-06-22T00:00:00Z"
    }
  ],
  "xmlrpc-post-page": [
    {
      id: 7000300,
      reply_to: null,
      title: "XML-RPC comment proof",
      content: "XML-RPC page comment proof body.",
      html: "<p>XML-RPC page comment proof body.</p>",
      created_by: "administrator",
      created_at: "2026-06-21T00:00:00Z"
    }
  ]
}

/** @type {Record<string, string | string[]>} */
export const parentBySlug = {
  "scp-173": "scp-173-parent",
  "public-child-private-parent": "private-page",
  "ambiguous-parent-child": ["main", "scp-173-parent"]
}

/**
 * @param {FixturePage | null} page
 * @param {PageDetails} details
 * @returns {Record<string, unknown> | null}
 */
export const toPageResult = (page, details) => {
  if (!page) return null

  /** @type {Record<string, unknown>} */
  const result = {
    page_created_at: page.page_created_at,
    page_id: page.page_id,
    page_updated_at: page.page_updated_at,
    page_revision_count: page.page_revision_count,
    revision_id: page.revision_id,
    revision_created_at: page.revision_created_at,
    revision_user_id: page.revision_user_id,
    title: page.title,
    slug: page.slug,
    tags: page.tags,
    rating: page.rating
  }

  if (details.wikitext) result.wikitext = page.wikitext
  if (details.compiled_html) {
    result.compiled_body_html = page.compiled_body_html
    result.compiled_body_styles = page.compiled_body_styles ?? []
  }
  return result
}
