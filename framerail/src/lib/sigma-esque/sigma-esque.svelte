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
    justify-content: stretch;
    align-items: stretch;
    min-height: 100vh;
    background-color: #fff;

    textarea, input[type="text"] {
      border-radius: 0.5em;
      padding: 0.5em 1em;
    }
  }

  .header {
    box-sizing: border-box;
    height: 10em;
    width: 100%;
    background: linear-gradient(180deg, #0068b5 0%, #19a9ff 100%);
    color: #fff;
    padding: 1em 2em;
  }

  .top-bar {
    box-sizing: border-box;
    height: 3em;
    width: 100%;
    background-color: #2b2b32;
    color: #fff;
    padding: 0 2em;
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
  }

  .page-content-container {
    flex: 1;
    margin: 2em auto;
    width: 80vw;
  }

  .footer {
    box-sizing: border-box;
    height: 3em;
    width: 100%;
    background-color: #2b2b32;
    color: #fff;
    padding: 0 2em;
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
  }

  .editor-actions {
    .editor-button {
      border-radius: 0.5em;
      padding: 0.5em 1em;
    }
  }

  @media (prefers-color-scheme: light) {
    .sigma-esque-container {
      background-color: #fff;
      color: #111;

      textarea, input[type="text"] {
        background-color: #fff;
        color: #111;
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

    .editor-actions {
      .editor-button {
        background-color: #fff;
        color: #111;
        border: 1px solid #0006;
      }
    }
  }

  @media (prefers-color-scheme: dark) {
    .sigma-esque-container {
      background-color: #222;
      color: #b6c2cf;

      textarea, input[type="text"] {
        background-color: #222;
        color: #b6c2cf;
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

  .editor-actions {
    .editor-button {
      background-color: #222;
      color: #b6c2cf;
      border: 1px solid #b6c2cf;
    }
  }
  }
</style>
