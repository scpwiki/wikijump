<div class="sigma-esque-container">
  <div class="header">
    <slot name="header" />
  </div>

  <div class="top-bar">
    <slot name="top-bar" />
  </div>

  <div class="page-content-container">
    <slot name="content" />
  </div>

  <div class="footer">
    <slot name="footer" />
  </div>
</div>

<!-- Ignoring the styling as being a theme it will inevitably style other elements in the entire layout -->
<!-- svelte-ignore css-unused-selector -->
<style global lang="scss">
  :root {
    /* Fallback colors */
    --text: #111;
    --background: #fff;
    --border: #0006;
    --mild-text: #f0f0f0;
    --accent: #0066cc;
    --error: red;
  }

  body {
    margin: 0;
  }

  .clickable {
    cursor: pointer;
    user-select: none;

    &:disabled {
      cursor: auto;
    }
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
    input[type="password"] {
      padding: 0.5em 1em;
      color: var(--text);
      background-color: var(--background);
      border: 1px solid var(--border);
      border-radius: 0.5em;

      &.error {
        outline: 1px solid var(--error);
      }
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
    background: linear-gradient(180deg, #0068b5 0%, #19a9ff 100%);
  }

  .top-bar {
    box-sizing: border-box;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    height: 3em;
    padding: 0 2em;
    color: #fff;
    background-color: #2b2b32;
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
    color: #fff;
    background-color: #2b2b32;
  }

  .action-row,
  .revision-attribute.action {
    .action-button {
      padding: 0.5em 1em;
      color: var(--text);
      background-color: var(--background);
      border: 1px solid var(--border);
      border-radius: 0.5em;

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
        }
        .vote-subbutton,
        .vote-rating-number {
          display: inline-block;
          flex: 1;
          padding: 0.5em 1em;
          color: var(--text);
          background-color: var(--background);
          border: 0;
          border-left: 1px solid var(--border);
          border-radius: 0;
          &:last-child {
            border-radius: 0 0.5em 0.5em 0;
          }
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
</style>
