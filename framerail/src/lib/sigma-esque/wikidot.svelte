<script lang="ts">
  import { wikidotCollapsibles } from "$lib/wikidot/wikidot-collapsibles"
  import { wikidotFooterControls } from "$lib/wikidot/wikidot-footer"

  import type { HTMLAnchorAttributes } from "svelte/elements"

  const pageTopAnchorAttributes: HTMLAnchorAttributes & { name: string } = {
    name: "page-top"
  }

  let {
    header,
    topBar,
    loginStatus,
    sideBar,
    content,
    footer,
    footerStyle = undefined,
    license,
    bottomToolbar
  } = $props()
</script>

<div id="skrollr-body" data-sveltekit-reload use:wikidotCollapsibles>
  <a {...pageTopAnchorAttributes}></a>
  <div id="container-wrap-wrap">
    <div id="container-wrap">
      <div id="container">
        <div id="header">
          {@render header?.()}
          <div id="top-bar">
            {@render topBar?.()}
          </div>
          {@render loginStatus?.()}
          <div id="header-extra-div-1"><span></span></div>
          <div id="header-extra-div-2"><span></span></div>
          <div id="header-extra-div-3"><span></span></div>
        </div>
        <div id="content-wrap">
          <div id="side-bar">
            {@render sideBar?.()}
            <a class="close-menu" href="##">
              <br />
              <img
                alt="black.png"
                class="image"
                src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAAMSURBVBhXY2BgYAAAAAQAAVzN/2kAAAAASUVORK5CYII="
                style="z-index:-1; opacity: 0.3;"
              />
              <br />
            </a>
          </div>
          <div id="main-content">
            <div id="action-area-top"></div>
            {@render content?.()}
          </div>
        </div>
        <div id="footer" style={footerStyle} use:wikidotFooterControls>
          {@render footer?.()}
        </div>
        <div id="license-area" class="license-area">
          {@render license?.()}
        </div>
        <div id="extrac-div-1"><span></span></div>
        <div id="extrac-div-2"><span></span></div>
        <div id="extrac-div-3"><span></span></div>
        {@render bottomToolbar?.()}
      </div>
      <div id="extra-div-1"><span></span></div>
      <div id="extra-div-2"><span></span></div>
      <div id="extra-div-3"><span></span></div>
      <div id="extra-div-4"><span></span></div>
      <div id="extra-div-5"><span></span></div>
      <div id="extra-div-6"><span></span></div>
    </div>
  </div>
</div>
<div id="dummy-ondomready-block" style:display="none"></div>

<!-- Ignoring the styling as being a theme it will inevitably style other elements in the entire layout -->
<!-- svelte-ignore css_unused_selector -->
<style global lang="scss">
  $tablet-max-width: 767px;

  :root {
    /* Fallback colors */
    --text: #111;
    --background: #fff;
    --border: #0006;
    --mild-text: #f0f0f0;
    --accent: #0066cc;
    --error: #b42d0a;
    --col-accent-1: #19a9ff;
    --col-accent-2: #0068b5;
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

  // Wikidot hides legacy equations until its MathJax loader completes. The
  // local imported shell does not run that remote loader, so keep the exact
  // server-rendered equation body visible instead of losing it to the base
  // stylesheet's display rule.
  #page-content .math-equation {
    display: block;
  }

  // Wikidot's base stylesheet expects legacy JavaScript to open this menu.
  // Imported themes such as Basalt already implement the hover/focus state in
  // CSS; this fallback keeps the same account menu usable on ordinary themes.
  #login-status:hover > #account-options,
  #login-status:focus-within > #account-options {
    display: block;
  }

  details.collapsible-block:not([open]) .collapsible-block-unfolded-link {
    display: none;
  }

  details.collapsible-block[open]
    > summary
    .collapsible-block-link:not(.collapsible-block-unfolded-link) {
    display: none;
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

    // Wikidot's Sigma-9 base stylesheet owns this shell's mobile navigation
    // positioning, and the shell must not override it. Real Wikidot anchors
    // `.mobile-top-bar` to the bottom left of the header box with
    // `position: absolute; left: 1em; bottom: 0`, because `#header` is itself
    // `position: relative`, and pins `.open-menu a` to a fixed bottom-left
    // viewport button. Only navigation bars rendered inside page content are
    // `position: static`, via a `#page-content` scoped rule. This DOM is
    // imported navbar HTML rather than Svelte markup, so any shell rule here
    // wins over the theme's own cascade and must leave the bar alone.
  }
</style>
