<!--
  @component Generic floating card.
-->
<script lang="ts">
  /** Text to display prominently. */
  export let title = ""
  /** Small text displayed below the title. Can be displayed by itself, as well. */
  export let subtitle = ""
  /** Determines color scheme. */
  export let theme: "light" | "dark" = "light"
  /** Sets the width of the card, if desired. Can be any valid CSS `width` value. */
  export let width = "auto"
</script>

<section style="width: {width};" class="card {theme}" {...$$restProps}>
  {#if title || subtitle}
    <div class="card-title" role="presentation">
      {#if title}<h1>{title}</h1>{/if}
      {#if subtitle}<small>{subtitle}</small>{/if}
    </div>
  {/if}

  <slot />

  <div class="card-actions" role="presentation"><slot name="actions" /></div>
</section>

<style lang="scss">
  @import "../../css/src/abstracts";

  .card {
    padding: 0.5rem;
    font-size: 1rem;
    background: var(--col-background);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.5rem;
    contain: content;
    @include shadow(4);
  }

  .card-title {
    padding-bottom: 0.25rem;

    > h1 {
      font-size: 1.25em;
    }

    > small {
      display: block;
      color: var(--col-subtle);
    }
  }

  .card-actions {
    display: block;
  }
</style>
