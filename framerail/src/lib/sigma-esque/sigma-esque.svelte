<script lang="ts">
  let { header, topBar, content, footer } = $props()
</script>

<div class="sigma-esque-container">
  <header class="header">
    {@render header?.()}
  </header>

  <nav class="top-bar">
    {@render topBar?.()}
  </nav>

  <article class="page-content-container">
    {@render content?.()}
  </article>

  <footer class="footer">
    {@render footer?.()}
  </footer>
</div>

<!-- Ignoring the styling as being a theme it will inevitably style other elements in the entire layout -->
<!-- svelte-ignore css_unused_selector -->
<style global lang="scss">
  @use "../css/main.scss";

  $tablet-max-width: 767px;

  :root {
    /* Fallback colors */
    --text: #111;
    --background: #fff;
    --border: #0006;
    --mild-text: #f0f0f0;
    --accent: #0066cc;
    --error: #b42d0a;

    --size-body-font: 16px;
    --size-action-button-font: 14px;
    --size-svg: 24px;
    --size-border-radius: 0.5em;
  }

  body {
    margin: 0;
    font-family: var(--font-sans), sans-serif;
    font-size: var(--size-body-font);
    font-weight: normal;
    line-height: 1.5;
  }

  .clickable {
    cursor: pointer;
    user-select: none;

    &:disabled {
      cursor: auto;
    }
  }

  .hidden {
    display: none;
  }

  .sigma-esque-container {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: stretch;
    min-height: 100vh;
    color: var(--text);
    background-color: var(--background);

    textarea,
    input[type="text"],
    input[type="date"],
    input[type="password"],
    select,
    fieldset {
      padding: 0.5em 1em;
      font-family: var(--font-sans), sans-serif;
      line-height: 1.5;
      color: var(--text);
      background-color: var(--background);
      border: 1px solid var(--border);
      border-radius: var(--size-border-radius);

      &.error {
        outline: 1px solid var(--error);
      }
    }

    input[type="radio"] {
      accent-color: var(--accent);
    }

    .debug {
      font-family: var(--font-mono);
      line-height: normal;
    }

    .user-attribute.avatar img {
      max-width: 200px;
      max-height: 200px;
    }
  }

  .header {
    box-sizing: border-box;
    width: 100%;
    height: 10em;
    padding: 1em 2em;
    color: #fff;
    background: linear-gradient(180deg, var(--col-accent-2) 0%, var(--col-accent-1) 100%);
  }

  .sigma-esque-container > .top-bar {
    box-sizing: border-box;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    height: 3em;
    padding: 0 2em;
    color: var(--col-dark-text-subtle);
    background-color: var(--col-dark-background);

    a {
      font-family: var(--font-display);
      font-weight: 700;
      color: var(--col-dark-text-subtle);
      text-decoration: none;
    }

    ul,
    ol {
      padding-left: unset;
      list-style-type: none;
    }
  }

  .page-content-container {
    flex: 1;
    width: 80vw;
    margin: 2em auto;

    hr {
      color: var(--mild-text);
    }
    a {
      color: var(--accent);
    }

    .page-title {
      font-size: 2em;
      font-weight: 700;
    }
  }

  .footer {
    box-sizing: border-box;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    min-height: 3em;
    padding: 0 2em;
    color: var(--col-dark-text-subtle);
    background-color: var(--col-dark-background);

    .footer-items .footer-item a {
      font-family: var(--font-display);
      font-weight: bold;
      color: var(--col-dark-text-subtle);
    }
  }

  .action-row,
  .revision-attribute.action,
  .file-attribute.action {
    .action-button {
      padding: 0.5em 1em;
      font-family: var(--font-display);
      font-size: var(--size-action-button-font);
      font-weight: 550;
      color: var(--text);
      background-color: var(--background);
      border: 1px solid var(--border);
      border-radius: var(--size-border-radius);

      &.cast-vote,
      &.vote-rating {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        padding: 0;

        .vote-desc {
          display: inline-block;
          padding: 0.5em 1em;
          font-family: inherit;
          font-size: inherit;
          font-weight: inherit;
        }
        .vote-subbutton,
        .vote-rating-number {
          display: inline-block;
          flex: 1;
          padding: 0.5em 1em;
          font-family: inherit;
          font-size: inherit;
          font-weight: inherit;
          color: var(--text);
          background-color: var(--background);
          border: 0;
          border-radius: 0;
          &:first-child {
            border-radius: var(--size-border-radius) 0 0 var(--size-border-radius);
          }
          &:last-child {
            border-radius: 0 var(--size-border-radius) var(--size-border-radius) 0;
          }
        }
        .vote-rating-number {
          font-family: var(--font-mono), monospace;
        }
      }
    }
  }

  @media (prefers-color-scheme: light) {
    :root {
      /* Fallback colors */
      --text: #111;
      --background: #fff;
      --border: #0006;
      --mild-text: #f0f0f0;
      --accent: #0066cc;
    }
  }

  @media (prefers-color-scheme: dark) {
    :root {
      /* Fallback colors */
      --text: #b6c2cf;
      --background: #222;
      --border: #b6c2cf;
      --mild-text: #bbb;
      --accent: #44aaff;
    }
  }

  @media (max-width: $tablet-max-width) {
    .header {
      height: initial;
    }
  }
</style>
