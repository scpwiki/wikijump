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
    background-color: #fff;

    textarea,
    input[type="text"] {
      padding: 0.5em 1em;
      border-radius: 0.5em;
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
  }

  .footer {
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

  .action-row {
    .action-button {
      padding: 0.5em 1em;
      border-radius: 0.5em;
    }
  }

  @media (prefers-color-scheme: light) {
    .sigma-esque-container {
      color: #111;
      background-color: #fff;

      textarea,
      input[type="text"] {
        color: #111;
        background-color: #fff;
        border: 1px solid #0006;
      }
    }

    .page-content-container {
      hr {
        color: #f0f0f0;
      }
      a {
        color: #0066cc;
      }
    }

    .action-row {
      .action-button {
        color: #111;
        background-color: #fff;
        border: 1px solid #0006;
      }
    }
  }

  @media (prefers-color-scheme: dark) {
    .sigma-esque-container {
      color: #b6c2cf;
      background-color: #222;

      textarea,
      input[type="text"] {
        color: #b6c2cf;
        background-color: #222;
        border: 1px solid #b6c2cf;
      }
    }

    .page-content-container {
      hr {
        color: #bbb;
      }
      a {
        color: #44aaff;
      }
    }

    .action-row {
      .action-button {
        color: #b6c2cf;
        background-color: #222;
        border: 1px solid #b6c2cf;
      }
    }
  }
</style>
