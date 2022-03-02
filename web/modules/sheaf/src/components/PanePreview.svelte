<!--
  @component Sheaf Editor: Preview Pane.
-->
<script lang="ts">
  import { Card, Spinny, Tab, Tabview } from "@wikijump/components"
  import { anim } from "@wikijump/components/lib"
  import Locale, { unit } from "@wikijump/fluent"
  import { Timeout } from "@wikijump/util"
  import { getContext } from "svelte"
  import type { SheafContext } from "../context"
  import CodeDisplay from "./CodeDisplay.svelte"
  import MorphingWikitext from "./MorphingWikitext.svelte"

  const t = Locale.makeComponentFormatter("sheaf")

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

  let timeTotal = 0
  let timeCompile = 0
  let timePatch = 0

  let rendering = false
  let showRendering = false

  $: debug = $settings.debug

  let timer = new Timeout(500, () => (showRendering = true), false)

  $: if (rendering === true && !timer.running) timer.reset()
  $: if (rendering === false) {
    timer.clear()
    showRendering = false
  }
</script>

<div class="sheaf-preview-container">
  <Tabview noborder contained conditional>
    <Tab>
      <span slot="button">{$t("#-preview.result")}</span>

      {#if showRendering}
        <div
          class="sheaf-preview-loading-panel"
          transition:anim={{ duration: 250, css: t => `opacity: ${t}` }}
        >
          <Spinny inline size="1.25rem" description={$t("#-preview.rendering")} />
        </div>
      {/if}

      {#if debug}
        <div class="sheaf-preview-perf-panel">
          <Card title={$t("#-preview.performance")} theme="dark" width="12rem">
            <div>
              <strong>{$t("#-preview.compile")}</strong>
              <code>{unit(timeCompile, "millisecond", { unitDisplay: "narrow" })}</code>
            </div>
            <div>
              <strong>{$t("#-preview.patch")}</strong>
              <code>{unit(timePatch, "millisecond", { unitDisplay: "narrow" })}</code>
            </div>
            <div>
              <strong>{$t("#-preview.total")}</strong>
              <code>{unit(timeTotal, "millisecond", { unitDisplay: "narrow" })}</code>
            </div>
          </Card>
        </div>
      {/if}

      <div class="sheaf-preview">
        <MorphingWikitext
          bind:timeCompile
          bind:timePatch
          bind:timeTotal
          bind:rendering
          wikitext={$editor}
        />
      </div>
    </Tab>

    <Tab>
      <span slot="button">{$t("#-preview.html")}</span>
      <CodeDisplay content={$editor.html(true)} lang="html" />
    </Tab>

    <Tab>
      <span slot="button">{$t("#-preview.css")}</span>
      <CodeDisplay content={$editor.style()} lang="css" />
    </Tab>

    {#if debug}
      <Tab>
        <span slot="button">{$t("#-preview.ast")}</span>
        <CodeDisplay content={$editor.prettyAST()} lang="json" />
      </Tab>

      <Tab>
        <span slot="button">{$t("#-preview.tokens")}</span>
        <CodeDisplay content={$editor.inspectTokens()} lang="FTMLTokens" />
      </Tab>

      <Tab>
        <span slot="button">{$t("#-preview.editor-ast")}</span>
        <CodeDisplay content={$editor.prettyEditorAST()} lang="PrintTree" />
      </Tab>
    {/if}
  </Tabview>
</div>

<style global lang="scss">
  .sheaf-preview-container {
    height: 100%;
  }

  .sheaf-preview-loading-panel {
    position: absolute;
    top: 1rem;
    left: 1rem;
    z-index: $z-above;
    padding: 0.25rem 0.5rem;
    color: var(--col-con-text);
    background: var(--col-con-background);
    border: solid 0.125rem var(--col-con-border);
    border-radius: 0.5rem;
    @include shadow(6);
  }

  .sheaf-preview-perf-panel {
    position: absolute;
    top: 1rem;
    right: 1rem;
    z-index: $z-above;
  }

  .sheaf-preview {
    width: 100%;
    height: 100%;
    padding: 0 1rem;
    overflow-x: hidden;
    overflow-y: auto;
    contain: strict;
  }
</style>
