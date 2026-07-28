# Using the data in ListPages modules

- Feature ID: `data-forms-dataforms-and-listpages`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Using the data in ListPages modules”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

## Implementation contract

- Category templates MUST recognize the documented field and layout syntax.
- Create and edit flows MUST validate, normalize, store, and redisplay field values as documented.
- Page rendering, template variables, CSS hooks, ListPages selection, and ordering MUST expose stored values as documented.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### Ratings, last comments, and data-form variables depend on runtime metadata

- Observation ID: `listpages-rating-comment-and-data-form-variables`
- Classification: `documentation-discrepancy`
- Observed at: `2026-07-28`
- Analysis: The documentation lists rating, comment, and data-form variables but omits their exact rating-mode markup, treats form values too uniformly, and does not define missing fields on an actual data-form page. Controlled run-owned pages, two independent voters, a last comment, a temporarily enabled five-star category, and a data-form template establish the runtime contract. The five-star category was restored to its prior disabled configuration and every run-owned page was removed after capture.

Normative behavior:

- rating_votes renders the number of votes for both plus/minus and five-star rating categories; it is not limited to five-star ratings.
- On a plus/minus category, rating renders the numeric net score and rating_percent remains literal.
- On a five-star category, rating renders a span.page-rate-list-pages-start whose data-rating attribute and text are the arithmetic mean, including a fractional mean and the zero-vote value 0.
- On a five-star category, rating_percent renders the arithmetic mean divided by five and multiplied by 100, without a percent-sign suffix; the observed values include 0, 80, and 90.
- For a page with comments, comments renders the count; commented_by renders the last commenter's display name; commented_by_unix renders the account unix name; commented_by_id renders the numeric Wikidot user ID; commented_by_linked renders printuser avatar/profile markup; and commented_at renders the standard odate span.
- On a data-form page, form_raw renders the stored scalar. form_data renders the display label for a select value and the stored scalar for an ordinary text value.
- form_label renders the field label. form_hint renders a supported field hint, an empty string when the field type does not expose its authored hint, and an empty string when no hint is authored.
- An empty field on a data-form page still resolves form_data, form_raw, form_label, and form_hint: the value variables are empty while label and supported hint metadata remain available.
- A missing field on an actual data-form page resolves every form variable to an empty string. This differs from an ordinary non-data-form page, where a missing form variable remains literal.

Evidence:

- `install/local/wikidot-verification/artifacts/listpages-campaign-rating-comment-data-form-live.json` (SHA-256 `df42b383b81eeac1c00c25fe54a59dcf2015ed622baea0752e9481d8bfe7708c`), cases: `lp-live-plus-minus-rating-and-last-comment`, `lp-live-five-star-rating`, `lp-live-five-star-fractional-rating`, `lp-live-five-star-zero-rating`, `lp-live-data-form-values-labels-and-hints`

### Data-form ListPages selection and ordering use stored field properties

- Observation ID: `dataforms-listpages-selection-sorting-live`
- Classification: `documentation-clarification`
- Observed at: `2026-07-29`
- Analysis: The data-form documentation states that ListPages can select and order by data-form fields, but the public examples also exercise an undocumented template composition path: a data-form category template can place current-page %%form_raw{field}%% variables inside a ListPages module head, and live Wikidot resolves those variables before evaluating _field selectors. A read-only capture of the live Vineyard demo confirms the ordinary data-form behavior. A run-owned sandbox probe also showed that raw source writes through the normal page-create path do not populate live Wikidot's data-form query/index state, so that route is recorded as an API/source-write limitation rather than the ordinary data-form UI oracle.

Normative behavior:

- ListPages arguments inside a data-form category template can use current-page %%form_raw{field}%% variables.
- Live Wikidot resolves current-page data-form variables in the ListPages module head before applying _field selectors.
- Multiple _field selectors combine with AND semantics.
- order="_field desc" sorts by the stored data-form field property while %%form_data{field}%% in the row template displays the field label/display value.
- Source-created sandbox pages with raw data-form-looking source did not participate in live Wikidot data-form selector or ordering indexes; this is an observed source-write limitation, not the ordinary data-form page behavior.

Evidence:

- `install/local/wikidot-verification/artifacts/dataforms-listpages-selection-sorting-live.json` (SHA-256 `70ffe68197540fe292f8343e98d64fe76fbadf73533a3537b12b4a7ea185fd6f`), cases: `vineyard-current-page-form-variables-drive-data-form-selectors`, `vineyard-data-form-order-desc-uses-stored-field-properties`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Data-form template parsing and saved page rendering
- Public create/edit/view flow and ListPages query behavior where documented

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:dataforms-and-listpages/source.wikidot.txt:1` through line 9 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:dataforms-and-listpages (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:dataforms-and-listpages/source.wikidot.txt:1` through line 9  
SHA-256 of complete source file: `2b6da73c430f0723d90bd259c8ac295a26ece7c7f016f8d4e4f865cd0553f3de`

```wikidot
L0001 The data that is produced by data forms can be used in the ListPages module (*http://www.wikidot.com/doc:listpages-module). With the band example, a ListPages module could look like this:
L0002 
L0003 [[code]]
L0004 [[module ListPages category="band" order="name"  separate="false" prependLine="||~ Band||~ Type ||~ Current ||" appendLine="||||||||~ ||"]]
L0005 || %%title_linked%% || %%form_data{type}%% || %%form_data{current}%% ||
L0006 [[/module]]
L0007 [[/code]]
L0008 
L0009 [[image df_bandlist.jpg]]
```
