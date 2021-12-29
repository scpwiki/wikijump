<!--
  @component Generic floating card.
-->
<script lang="ts">
  /** Text to display prominently. */
  export let title = ""

  /** Small text displayed below the title. Can be displayed by itself, as well. */
  export let subtitle = ""

  /** Determines color scheme. */
  export let theme: "auto" | "light" | "dark" = "auto"

  /** Sets the width of the card, if desired. Can be any valid CSS `width` value. */
  export let width = "auto"
</script>

<section
  style="width: {width};"
  class="card {theme !== 'auto' ? theme : ''}"
  {...$$restProps}
>
  {#if title || subtitle}
    <div class="card-title" role="presentation">
      {#if title}<h1>{title}</h1>{/if}
      {#if subtitle}<small>{subtitle}</small>{/if}
    </div>
  {/if}

  <slot />

  <div class="card-actions" role="presentation"><slot name="actions" /></div>
</section>

<style global lang="scss">
  @import "../../../resources/css/abstracts";

  .card {
    max-width: 90vw;
    padding: 0.75rem 1rem;
    padding-top: 0.5rem;
    font-size: 1rem;
    background: var(--col-background);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.5rem;
    contain: content;
    @include shadow(4);
  }

  .card-title {
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
