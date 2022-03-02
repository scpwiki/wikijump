<!--
  @component Sheaf Editor: Editor Pane Topbar.
-->
<script lang="ts">
  import { textByteLength } from "@wikijump/codemirror"
  import { Button } from "@wikijump/components"
  import Locale, { number, unit } from "@wikijump/fluent"
  import { throttle } from "@wikijump/util"
  import { getContext } from "svelte"
  import type { SheafContext } from "../context"
  import SettingsMenu from "./SettingsMenu.svelte"

  const t = Locale.makeComponentFormatter("sheaf")

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

  // -- STATS

  let chars = 0
  let bytes = 0
  let words = 0
  let lines = 0

  // we really don't want to call this function often
  // it has to get the editor's value, which means it has to stringify
  // the document contents, which is expensive and memory intensive
  const updateWordCount = throttle(async () => {
    words = await $editor.wordCount()
  }, 1000)

  // seems a bit excessive to call this function every time the editor changes,
  // but it's actually very cheap. still, it's probably a good idea to throttle it.
  // it's in kilobytes anyways, so you won't notice the throttling because you'd
  // have to be typing insanely fast.
  const updateByteCount = throttle(async () => {
    bytes = Math.round(textByteLength($editor.doc) / 1000)
  }, 1000)

  $: if ($editor.doc) {
    // we'll update chars and lines here because that's cheap, and it fools the user
    // into thinking that the stats display is super responsive :)
    chars = $editor.doc.length
    lines = $editor.doc.lines
    updateWordCount()
    updateByteCount()
  }
</script>

<div class="sheaf-pane-editor-topbar">
  <div class="sheaf-title" aria-hidden="true">
    <span class="sheaf-title-text">{$t("#.title")}</span>
    <span class="sheaf-title-version">{$t("#.version")}</span>
  </div>

  <div class="sheaf-topbar-sep" />

  <!-- TODO: figure out how you're actually supposed to make a sideways table -->
  <div class="sheaf-stats">
    <table class="sheaf-stats-column">
      <tr>
        <td>{$t("#-stats.chars")}</td>
        <td>{number(chars, { useGrouping: false })}</td>
      </tr>
      <tr>
        <td>{$t("#-stats.bytes")}</td>
        <td>
          {unit(bytes, "kilobyte", { useGrouping: false, unitDisplay: "narrow" })}
        </td>
      </tr>
    </table>
    <table class="sheaf-stats-column">
      <tr>
        <td>{$t("#-stats.words")}</td>
        <td>{number(words, { useGrouping: false })}</td>
      </tr>
      <tr>
        <td>{$t("#-stats.lines")}</td>
        <td>{number(lines, { useGrouping: false })}</td>
      </tr>
    </table>
  </div>

  <div class="sheaf-topbar-sep" />

  <div class="sheaf-actions">
    <SettingsMenu />
  </div>

  <!-- Preview open, close button -->
  {#if !$small}
    <div class="sheaf-button-toggle-preview">
      {#if $settings.preview.enabled}
        <Button
          i="fluent:caret-right-24-filled"
          tip={$t("#-preview.close")}
          size="1.75rem"
          baseline
          on:click={() => ($settings.preview.enabled = false)}
        />
      {:else}
        <Button
          i="fluent:caret-left-24-filled"
          tip={$t("#-preview.open")}
          size="1.75rem"
          baseline
          on:click={() => ($settings.preview.enabled = true)}
        />
      {/if}
    </div>
  {/if}
</div>

<style global lang="scss">
  .sheaf-pane-editor-topbar {
    position: relative;
    z-index: $z-above;
    display: flex;
    grid-area: "topbar";
    gap: 0.75rem;
    align-items: center;
    padding: 0;
    background: var(--colcode-background);
    box-shadow: -0.5rem 0 0.25rem rgba(black, 0.25);
  }

  .sheaf-topbar-sep {
    width: 0.125rem;
    height: 1rem;
    background: var(--col-border);
  }

  .sheaf-stats {
    display: flex;
    gap: 0.5rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
  }

  .sheaf-stats-column {
    > tr {
      height: 0.825rem;
      line-height: 0;
    }
    > tr > td:nth-child(1) {
      padding-right: 0.25rem;
      color: var(--col-text-dim);
    }
    > tr > td:nth-child(2) {
      color: var(--col-text-subtle);
    }
  }

  .sheaf-title {
    display: inline-flex;
    flex-direction: row;
    gap: 0.25rem;
    align-items: center;
    padding-left: 0.5rem;
  }

  .sheaf-title-text {
    font-family: var(--font-display);
    font-size: 1rem;
    font-style: italic;
    font-weight: 700;
    color: var(--col-text);
  }

  .sheaf-title-version {
    font-family: var(--font-display);
    font-size: 1rem;
    font-style: italic;
    font-weight: 400;
    color: var(--col-text-subtle);
  }

  .sheaf-button-toggle-preview {
    position: absolute;
    top: 50%;
    right: 0.5rem;
    transform: translateY(-50%);
  }
</style>
