<script lang="ts">
  import { Button, Card, DetailsMenu, Toggle } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import { getContext } from "svelte"
  import type { SheafContext } from "../context"

  const t = Locale.makeComponentFormatter("sheaf")

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")
</script>

<DetailsMenu let:open>
  <Button
    slot="button"
    i="carbon:settings"
    size="1.5rem"
    active={open}
    tip={$t("#-settings")}
    baseline
  />
  <Card width="12rem">
    <div class="sheaf-settings-menu">
      <div class="sheaf-settings-menu-sep">
        <span class="sheaf-settings-menu-title">{$t("general")}</span>
        <span class="sheaf-settings-menu-sep-line" />
      </div>

      <Toggle size="1.125rem" bind:toggled={$settings.debug} wide flipped>
        {$t("#-settings.debug-mode")}
      </Toggle>

      <div class="sheaf-settings-menu-sep">
        <span class="sheaf-settings-menu-title">{$t("editor")}</span>
        <span class="sheaf-settings-menu-sep-line" />
      </div>

      <Toggle size="1.125rem" wide flipped bind:toggled={$settings.editor.darkmode}>
        {$t("#-settings.dark-mode")}
      </Toggle>

      <Toggle size="1.125rem" wide flipped bind:toggled={$settings.editor.spellcheck}>
        {$t("#-settings.spellcheck")}
      </Toggle>

      <div class="sheaf-settings-menu-sep">
        <span class="sheaf-settings-menu-title">{$t("preview")}</span>
        <span class="sheaf-settings-menu-sep-line" />
      </div>

      <Toggle size="1.125rem" wide flipped bind:toggled={$settings.preview.darkmode}>
        {$t("#-settings.dark-mode")}
      </Toggle>
    </div>
  </Card>
</DetailsMenu>

<style global lang="scss">
  .sheaf-settings-menu {
    display: flex;
    flex-direction: column;
    row-gap: 0.5rem;
  }

  .sheaf-settings-menu-sep {
    display: flex;
    column-gap: 0.5rem;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.75rem;

    &:first-child {
      margin-top: 0;
    }
  }

  .sheaf-settings-menu-title {
    font-family: var(--font-display);
    font-size: 0.875rem;
    font-weight: bold;
    color: var(--col-text-subtle);
  }

  .sheaf-settings-menu-sep-line {
    flex-grow: 1;
    height: 0.175rem;
    background-color: var(--col-border);
    border-radius: 0.5rem;
  }
</style>
