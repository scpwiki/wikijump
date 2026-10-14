# Default page layout

- Feature ID: `layout-page`
- Category: `layout`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Render default page layout with the documented placeholders, conditional sections, element order, identifiers, and nesting.

## Implementation contract

- The Wikidot layout renderer MUST emit the documented regions, identifiers, order, and nesting.
- Conditional regions and placeholders MUST use the documented context and visibility rules.
- Browser tests MUST verify final DOM and any user-visible intermediate state.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### Wikidot document shells propagate the stored locale verbatim

- Observation ID: `wikidot-document-shell-stored-locale-20260722`
- Classification: `documentation-omission`
- Observed at: `2026-07-22`
- Analysis: Raw HTTP responses for the same run-owned sandbox article, captured immediately before and after the stored site locale changed from en to ja-corrections, isolate the document-shell language fields. This observation is intentionally limited to those two stored-locale values on a Wikidot article document shell; it does not establish native Japanese-site behavior, locale negotiation, translation behavior, error shells, or non-Wikidot routes.

Normative behavior:

- For each captured stored locale, the raw Wikidot article response starts with an html element whose attributes are exactly xmlns="http://www.w3.org/1999/xhtml", xml:lang="LOCALE", and lang="LOCALE" in that order.
- Each captured raw response contains exactly one <meta http-equiv="content-language" content="LOCALE"/> element using the same stored locale.
- The captured ja-corrections value is propagated verbatim in the root attributes and content-language metadata; the en capture is the before-mutation control.
- The evidence establishes only exact stored-locale propagation on the captured Wikidot article document shell. It makes no claim about native Japanese sites, other language effects, error shells, special routes, API responses, or Wikijump-layout documents.

Raw HTTP captures:

- before stored locale `en`: `/mnt/oracle-store/wjlab/sandbox-oracle-20260722/site-settings-scope-v1/mutations/locale/before.http.html` (SHA-256 `6861d8b772e6585475ab13c078fd7d9aaf53af5a4ee2f4ec17afc233bcad572b`)
- after stored locale `ja-corrections`: `/mnt/oracle-store/wjlab/sandbox-oracle-20260722/site-settings-scope-v1/mutations/locale/after.http.html` (SHA-256 `0641a8b445541eb39ffec2304dc5d8acac6b331ab26befecee016b60ed31c292`)

Evidence:

- `/mnt/oracle-store/wjlab/sandbox-oracle-20260722/site-settings-scope-v1/mutations/locale/verdict.json` (SHA-256 `45d07066f84d8990b768817a522a53c3b2d260f9baac8bc040527c6c70462275`), cases: none

### Current standard saved pages do not automatically render parent breadcrumbs

- Observation ID: `parent-relations-standard-layout-no-automatic-breadcrumbs-20260908`
- Classification: `documentation-correction`
- Observed at: `2026-09-08`
- Analysis: Legacy Wikidot documentation says that assigning a parent places breadcrumb navigation at the top of the page and lists div#breadcrumbs in the default page layout. Current live Wikidot contradicts that automatic standard-layout presentation. A reversible sandbox-for-codex lifecycle created a root-to-parent-to-child chain, successfully mutated parent relations, renamed the parent, exercised no-parent, deleted-parent, cyclic, and independent controls, and observed no #breadcrumbs element in any saved-page response. Independently, the synchronized www corpus records doc:site-structure with parent_fullname doc:start, while a current HTTP 200 read of that exact documentation page also contains no #breadcrumbs. The parent relationship remains authoritative for structural queries and other consumers. The separately documented Pro custom-layout [[breadcrumbs]] element was not observed and must not be inferred from this standard-layout correction.

Normative behavior:

- Persist and expose parent relationships according to the documented site-structure contract; this observation changes only automatic standard saved-page presentation.
- A standard saved page must not synthesize div#breadcrumbs solely because one or more parent relationships are available.
- Missing, deleted, renamed, or cyclic parent state must not cause the standard saved-page shell to invent breadcrumb markup.
- The Pro custom-layout [[breadcrumbs]] element remains a separate documented surface and is not redefined by this observation.

Evidence:

- `install/local/wikidot-verification/artifacts/issue1063-parent-breadcrumb-live-20260908.json` (SHA-256 `5c68a8df37651e4fc0753860ca2aac2e6a8abac637665a1ff7fd8253852ae444`), cases: `issue1063-parent-chain-standard-layout-no-breadcrumbs`, `issue1063-renamed-parent-standard-layout-no-breadcrumbs`, `issue1063-no-parent-standard-layout-control`, `issue1063-deleted-parent-standard-layout-control`, `issue1063-cycle-standard-layout-control`, `issue1063-independent-standard-layout-control`, `issue1063-current-parented-documentation-page-no-breadcrumbs`


### Standard footer and license template preserve text nodes

The retained English and Japanese standard shells captured on 2026-09-29
emit the same footer structure with localized link labels. Preserve the
template's whitespace text nodes, nonbreaking spaces before the separators,
inline visibility styles, control IDs, and linked `Powered by` text. The
cookie-consent entry remains hidden when the advertising consent runtime is
absent. Wikijump does not execute the third-party advertising script; its
known report and consent handlers remain inert under the local CSP boundary.

The standard license area preserves template whitespace and `rel="license"`.
Creative Commons links use the HTTP deed URL; the Japanese template appends
`deed.ja`. Custom license HTML and copyright-only modes retain their separate
contracts.

Frozen browser DOM evidence and the byte-exact regression oracle:

- `install/local/theme-lab/evidence/wikidot-runtime-surface-20260929.normal/scp-wiki-scp-173-normal-20260929-5d64b862fb46/live.dom.html`
- `install/local/theme-lab/evidence/wikidot-runtime-surface-20260929.normal/scp-jp-scp-173-normal-20260929-7984e750c954/live.dom.html`
- `framerail/tests/wikidot-footer.test.js`

### Standard secondary page actions and hovertip text nodes

Observation: `standard-secondary-actions-en-ja-20260929`, confirmed for the
English Basalt canary on 2026-10-12. These rules describe standard Wikidot page
chrome; they do not define the custom-layout PageOptionsBottom module.

- The secondary row remains in the initial DOM with inline `display: none`.
- Its anchor order is Edit Sections, Append, Edit Meta, Watchers, Backlinks,
  Page Source, Parent, Lock Page, Rename, Delete. The Japanese shell uses the
  corresponding retained Japanese labels in the same order.
- Lock Page has ID `page-block-button`. The standard row contains neither
  `layout-button` nor `lock-page-button`; anchors carry no `type` attribute.
- Opening and closing the row retains its elements. The Options control uses
  a nonbreaking space between its sign and label in the observed expanded state.
- The hovertip initializer preserves the server template's text nodes inside
  `div.content`, including indentation and the extra spaces in the English
  Edit Sections message. Visible text layout still follows normal HTML
  whitespace collapsing; hidden text nodes must not be trimmed.

The frozen reduced observations and original capture SHA-256 values are in
`framerail/tests/fixtures/wikidot-standard-page-chrome.json`. Their authorities
are the EN and JP `live.dom.html` files in
`install/local/theme-lab/evidence/wikidot-runtime-surface-20260929.normal/`.
The two sites agree on order and DOM identifiers, differ in localized labels,
and have no Layout action. The native Wikijump layout retains its own action
order, Layout action, identifiers and translations.

### Read-only standard footer modules (2026-10-12)

Anonymous EN and JA BugReportModule reads return the standard problem-report
window linking to `https://feedback.wikidot.com/`. Anonymous FlagPageModule
returns `no_permission`; the English callback builds the Permission error
window, focuses its close button, and includes the sign-in/account message.
The frozen module bodies are in `framerail/tests/fixtures/wikidot-footer-dialogs.json`.
Source callback DOM and screenshots are retained under
`/home/roku/.local/state/wikijump/scp9506-final-evidence/browser-final-20261012-source-footer-dialogs-callback`.
This callback replay proves rendering, not a server-side report submission.
OZONE retains an empty hidden `odialog-container` after closing, removes the
shader and blank iframe after 200ms, and does not bind Escape/outside dismissal.
Bug opening retains focus on its invoking control; closing remains visible during that delay; ErrorDialog fades out for 100ms.
Unobserved authenticated flag reporting and JA error chrome remain unsupported.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms
- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:1` through line 46 (canonical)

## Documentation-derived behavioral evidence

### doc:layout-reference (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:1` through line 46  
SHA-256 of complete source file: `bdb2ffc85a5b5e200b2df4a63c32fe5a86a2699a5c8ce58678103af949ab93ba`

```wikidot
L0001 + Page layout
L0002 
L0003 The tree below represents the structure of pages that is valid for all wikidot-powered sites. This reference should help all these who wish to develop custom CSS themes.
L0004 
L0005 
L0006 * {{div#container}}
L0007  * {{div#header}}
L0008   * {{h1}}
L0009    * {{a}}
L0010     * {{span}} (with the name of the site)
L0011   * {{h2}} (if subtitle exists)
L0012    * {{span}} (with the subtitle of the site)
L0013   * {{div#top-bar}} (if top-bar navigation element exists)
L0014   * {{div#login-status}}
L0015   * {{div#header-extra-div-1}} (extra divs for CSS design)
L0016    * {{span}}
L0017   * {{div#header-extra-div-2}} (extra divs for CSS design)
L0018    * {{span}}
L0019   * {{div#header-extra-div-3}} (extra divs for CSS design)
L0020    * {{span}}
L0021  * {{div#content-wrap}}
L0022   * {{div#side-bar}} (if side-bar navigation element exists)
L0023   * {{div#main-content}}
L0024    * {{div#action-area-top}}
L0025    * {{div#page-title}}
L0026    * {{div#breadcrumbs}}
L0027    * {{div#page-content}} (main content of the page)
L0028    * {{div#page-info}}
L0029    * {{div#page-options-bottom.page-options-bottom}}
L0030    * {{div#page-options-bottom-2.page-options-bottom}}
L0031    * {{div#action-area}}
L0032  * {{div#footer}}
L0033   * {{div.options}}
L0034  * {{div#license-area}}
L0035 * {{div#extra-div-1}} (extra divs for CSS design)
L0036  * {{span}}
L0037 * {{div#extra-div-2}}
L0038  * {{span}}
L0039 * {{div#extra-div-3}}
L0040  * {{span}}
L0041 * {{div#extra-div-4}}
L0042  * {{span}}
L0043 * {{div#extra-div-5}}
L0044  * {{span}}
L0045 * {{div#extra-div-6}}
L0046 
```
