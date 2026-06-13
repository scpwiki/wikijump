<script lang="ts">
  let { data, form } = $props()

  const defaultTitle = "UI Authoring Basic"
  const defaultTags = "ui-authoring verification"
  const defaultSource =
    "+ UI Authoring Basic\n\nThis page was created through the local authoring lab.\n\n[[include ui-authoring-fragment-alpha]]\n"

  const formTagText = () =>
    Array.isArray(form?.tags) ? form.tags.join(" ") : form?.tags
  const currentPage = () => form?.saved?.page ?? form?.page ?? data.lab.page
  const currentParents = () => form?.saved?.parents ?? form?.parents ?? data.lab.parents
  const currentHistory = () => form?.history ?? data.lab.history

  let slug = $state((() => data.lab.selectedSlug)())
  let title = $state((() => data.lab.page?.title ?? defaultTitle)())
  let tags = $state(
    (() => data.lab.page?.tags?.join(" ") ?? defaultTags)()
  )
  let parent = $state((() => data.lab.parents?.[0] ?? "")())
  let wikitext = $state(
    (() => data.lab.page?.wikitext ?? defaultSource)()
  )

  $effect(() => {
    slug = form?.slug ?? data.lab.selectedSlug
    title = form?.title ?? currentPage()?.title ?? defaultTitle
    tags = formTagText() ?? currentPage()?.tags?.join(" ") ?? defaultTags
    parent = form?.parent ?? currentParents()?.[0] ?? ""
    wikitext = form?.wikitext ?? currentPage()?.wikitext ?? defaultSource
  })
</script>

<svelte:head>
  <title>Local Wikidot Verification Lab</title>
</svelte:head>

<main class="lab-shell" data-testid="local-wikidot-lab">
  <header class="lab-header">
    <div>
      <p class="eyebrow">Local-only Wikidot Verification Lab</p>
      <h1>Authoring and verification</h1>
    </div>
    <a class="page-link" href={`/${slug}`} data-testid="rendered-page-link"
      >Open rendered page</a
    >
  </header>

  {#if form?.message}
    <p class="status error" data-testid="lab-error">{form.message}</p>
  {:else if form}
    <p class="status" data-testid="lab-status">Last action: {form.type}</p>
  {/if}

  <section class="lab-grid">
    <form
      method="POST"
      action="?/preview"
      class="panel editor-panel"
      data-testid="preview-form"
    >
      <h2>Source</h2>
      <label>
        Slug
        <input name="slug" bind:value={slug} data-testid="slug-input" />
      </label>
      <label>
        Title
        <input name="title" bind:value={title} data-testid="title-input" />
      </label>
      <label>
        Tags
        <input name="tags" bind:value={tags} data-testid="tags-input" />
      </label>
      <label>
        Parent
        <input name="parent" bind:value={parent} data-testid="parent-input" />
      </label>
      <label class="source-label">
        Wikidot source
        <textarea
          name="wikitext"
          bind:value={wikitext}
          rows="18"
          data-testid="source-input"
        ></textarea>
      </label>
      <div class="button-row">
        <button type="submit" data-testid="preview-button">Preview</button>
        <button formaction="?/savePage" data-testid="save-button">Save page</button>
      </div>
    </form>

    <section class="panel preview-panel" data-testid="preview-panel">
      <h2>Preview</h2>
      {#if form?.type === "preview"}
        {#if form.preview?.slug}
          <p class="metadata" data-testid="preview-source">
            Renderer preview page:
            <a class="text-link" href={`/${form.preview.slug}`}>{form.preview.slug}</a>
          </p>
        {/if}
        <div class="rendered" data-testid="preview-html">{@html form.previewHtml}</div>
        {#if form.warnings?.length}
          <ul class="warnings" data-testid="preview-warnings">
            {#each form.warnings as warning}
              <li>{warning}</li>
            {/each}
          </ul>
        {/if}
      {:else if currentPage()?.compiled_body_html}
        <div class="rendered" data-testid="current-rendered-html">
          {@html currentPage()?.compiled_body_html}
        </div>
      {:else}
        <p class="empty">No saved page is selected yet.</p>
      {/if}
    </section>
  </section>

  <section class="tools-grid">
    <form method="POST" action="?/updateTags" class="panel" data-testid="tag-form">
      <h2>Tags</h2>
      <input type="hidden" name="slug" value={slug} />
      <label>
        Tag set
        <input name="tags" bind:value={tags} data-testid="tag-editor-input" />
      </label>
      <button data-testid="tag-save-button">Save tags</button>
      <p class="metadata" data-testid="current-tags">
        Current: {currentPage()?.tags?.join(", ") || "none"}
      </p>
    </form>

    <form
      method="POST"
      action="?/uploadFile"
      enctype="multipart/form-data"
      class="panel"
      data-testid="file-form"
    >
      <h2>Files</h2>
      <input type="hidden" name="slug" value={slug} />
      <label>
        Display name
        <input name="name" data-testid="file-name-input" />
      </label>
      <label>
        Upload
        <input name="file" type="file" data-testid="file-input" />
      </label>
      <button data-testid="file-upload-button">Upload file</button>
      {#if form?.type === "uploadFile"}
        <pre data-testid="file-result">{JSON.stringify(form.files, null, 2)}</pre>
      {/if}
    </form>

    <section class="panel" data-testid="history-panel">
      <h2>History</h2>
      {#if currentHistory()?.length}
        <ol>
          {#each currentHistory() as revision}
            <li>
              <span>#{revision.revision_number}</span>
              <code>{revision.revision_id}</code>
              <span>{revision.comments}</span>
            </li>
          {/each}
        </ol>
      {:else}
        <p class="empty">No revisions.</p>
      {/if}
    </section>

    <section class="panel" data-testid="diagnostics-panel">
      <h2>Diagnostics</h2>
      <dl>
        <div>
          <dt>Page ID</dt>
          <dd>{currentPage()?.page_id ?? "missing"}</dd>
        </div>
        <div>
          <dt>Revision</dt>
          <dd>{currentPage()?.revision_number ?? "missing"}</dd>
        </div>
        <div>
          <dt>Parents</dt>
          <dd>{currentParents()?.join(", ") || "none"}</dd>
        </div>
      </dl>
    </section>
  </section>

  <section class="tools-grid dependency-grid">
    <form
      method="POST"
      action="?/createDependencies"
      class="panel"
      data-testid="dependency-form"
    >
      <h2>Dependencies</h2>
      <p class="metadata">
        Creates a fragment, a component page, and a host page that includes both.
      </p>
      <button data-testid="dependency-create-button">Create include pages</button>
      {#if form?.type === "createDependencies"}
        <div class="rendered compact" data-testid="dependency-result">
          {@html form.dependencies?.hostHtml ?? ""}
        </div>
        <a class="text-link" href="/ui-authoring-include-host">Open include host</a>
      {/if}
    </form>

    <form
      method="POST"
      action="?/createListPages"
      class="panel"
      data-testid="listpages-form"
    >
      <h2>ListPages</h2>
      <p class="metadata">Creates three tagged targets and an index module.</p>
      <button data-testid="listpages-create-button">Create ListPages set</button>
      {#if form?.type === "createListPages"}
        <div class="rendered compact" data-testid="listpages-result">
          {@html form.listPages?.indexHtml ?? ""}
        </div>
        <a class="text-link" href="/ui-authoring-listpages-index">Open ListPages index</a>
      {/if}
    </form>

    <form
      method="POST"
      action="?/removeListPagesGamma"
      class="panel"
      data-testid="listpages-remove-form"
    >
      <h2>Tag Update Check</h2>
      <p class="metadata">
        Removes the ListPages tag from the gamma target and rerenders the index.
      </p>
      <button data-testid="listpages-remove-button">Remove gamma tag</button>
      {#if form?.type === "removeListPagesGamma"}
        <div class="rendered compact" data-testid="listpages-after-remove">
          {@html form.indexHtml}
        </div>
      {/if}
    </form>

    <form
      method="POST"
      action="?/createThemeNavCss"
      class="panel"
      data-testid="theme-nav-css-form"
    >
      <h2>Theme / Nav / CSS</h2>
      <p class="metadata">
        Updates local nav pages and creates a CSS proof page rendered by the shell.
      </p>
      <button data-testid="theme-nav-css-button">Create theme proof</button>
      {#if form?.type === "createThemeNavCss"}
        <div class="rendered compact" data-testid="theme-nav-css-result">
          {@html form.themeNavCss?.proofHtml ?? ""}
        </div>
        <a class="text-link" href="/ui-authoring-theme-nav-css">Open theme proof</a>
      {/if}
    </form>
  </section>

  <section class="tools-grid proof-grid">
    <section class="panel" data-testid="page-list-panel">
      <h2>Page List</h2>
      <ul class="page-list">
        {#each data.lab.scenarioPages as page}
          <li>
            <a class:missing={!page.exists} href={`/__local-wikidot-verify?slug=${page.slug}`}>
              {page.slug}
            </a>
            <span>{page.exists ? `r${page.revisionNumber}` : "missing"}</span>
          </li>
        {/each}
      </ul>
    </section>

    <form
      method="POST"
      action="?/runProofSummary"
      class="panel"
      data-testid="proof-runner-form"
    >
      <h2>Proof Runner</h2>
      <input type="hidden" name="slug" value={slug} />
      <p class="metadata">Checks the selected page and generated local scenarios.</p>
      <button data-testid="proof-runner-button">Run proof summary</button>
      {#if form?.type === "runProofSummary"}
        <div class="proof-summary" data-testid="proof-runner-result">
          <p>
            {form.proofSummary?.passed ?? 0}/{form.proofSummary?.checks.length ?? 0}
            checks passed
          </p>
          <ul>
            {#each form.proofSummary?.checks ?? [] as check}
              <li class:failed={!check.pass}>
                <span>{check.pass ? "PASS" : "FAIL"}</span>
                <code>{check.name}</code>
                <span>{check.detail}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </form>
  </section>

  <section class="tools-grid bundle-grid">
    <form method="POST" action="?/exportBundle" class="panel" data-testid="export-form">
      <h2>Export</h2>
      <label>
        Slugs
        <input name="slugs" value={slug} data-testid="export-slugs-input" />
      </label>
      <button data-testid="export-button">Export bundle</button>
      {#if form?.type === "exportBundle"}
        <textarea readonly rows="12" data-testid="bundle-output"
          >{form.bundleText}</textarea
        >
      {/if}
    </form>

    <form method="POST" action="?/importBundle" class="panel" data-testid="import-form">
      <h2>Import</h2>
      <label>
        Prefix
        <input
          name="prefix"
          value="ui-authoring-import-"
          data-testid="import-prefix-input"
        />
      </label>
      <label>
        Bundle JSON
        <textarea name="bundle" rows="12" data-testid="bundle-input"
          >{form?.bundleText ?? ""}</textarea
        >
      </label>
      <button data-testid="import-button">Import bundle</button>
      {#if form?.type === "importBundle"}
        <pre data-testid="import-result">{JSON.stringify(form.imported, null, 2)}</pre>
      {/if}
    </form>
  </section>
</main>

<style lang="scss">
  .lab-shell {
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 1320px;
    margin: 0 auto;
    padding: 18px;
    color: #1f2933;
  }

  .lab-header,
  .button-row,
  .tools-grid,
  .lab-grid {
    display: grid;
    gap: 14px;
  }

  .lab-header {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    padding-bottom: 12px;
    border-bottom: 1px solid #c8d0d8;
  }

  .eyebrow {
    margin: 0 0 4px;
    color: #557086;
    font-size: 0.8rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  h1,
  h2 {
    margin: 0;
    line-height: 1.2;
  }

  h1 {
    font-size: 2rem;
  }

  h2 {
    font-size: 1.1rem;
  }

  .page-link,
  button {
    min-height: 38px;
    padding: 8px 12px;
    border: 1px solid #2d5874;
    border-radius: 4px;
    background: #2d5874;
    color: #fff;
    font-weight: 700;
    text-decoration: none;
    cursor: pointer;
  }

  .status {
    margin: 0;
    padding: 10px 12px;
    border-left: 4px solid #4d7c0f;
    background: #edf6df;
  }

  .status.error {
    border-left-color: #b42318;
    background: #fff1f0;
  }

  .lab-grid {
    grid-template-columns: minmax(360px, 0.9fr) minmax(360px, 1.1fr);
  }

  .tools-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .bundle-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .dependency-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .proof-grid {
    grid-template-columns: minmax(320px, 0.8fr) minmax(420px, 1.2fr);
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border: 1px solid #c8d0d8;
    border-radius: 6px;
    background: #f8fafc;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-weight: 700;
  }

  input,
  textarea {
    width: 100%;
    min-height: 36px;
    padding: 7px 9px;
    border: 1px solid #97a6b2;
    border-radius: 4px;
    background: #fff;
    color: #111827;
    font: inherit;
    font-weight: 400;
  }

  textarea {
    min-height: 120px;
    font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
    resize: vertical;
  }

  .source-label textarea {
    min-height: 380px;
  }

  .button-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .rendered {
    min-height: 420px;
    padding: 14px;
    border: 1px solid #d8dee4;
    background: #fff;
    overflow: auto;
  }

  .rendered.compact {
    min-height: 180px;
  }

  .text-link {
    color: #2d5874;
    font-weight: 700;
  }

  .page-list,
  .proof-summary ul {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .page-list li,
  .proof-summary li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 8px;
    align-items: center;
  }

  .proof-summary li {
    grid-template-columns: 48px minmax(160px, 0.45fr) minmax(0, 1fr);
  }

  .proof-summary li > span:first-child {
    color: #3f6d0b;
    font-weight: 700;
  }

  .proof-summary li.failed > span:first-child,
  .missing {
    color: #b42318;
  }

  .warnings {
    margin: 0;
    padding-left: 22px;
    color: #9a3412;
  }

  .metadata,
  .empty {
    margin: 0;
    color: #52616e;
  }

  pre {
    max-height: 240px;
    overflow: auto;
    padding: 10px;
    background: #111827;
    color: #f8fafc;
  }

  dl {
    display: grid;
    gap: 8px;
    margin: 0;
  }

  dl div {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    gap: 8px;
  }

  dt {
    font-weight: 700;
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  @media (max-width: 980px) {
    .lab-header,
    .lab-grid,
    .tools-grid,
    .bundle-grid,
    .proof-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
