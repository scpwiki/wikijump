<!--
  @component Generic floating card.
-->
<script lang="ts">
  let {
    title = "",
    subtitle = "",
    theme = "auto",
    width = "auto",
    actions,
    children,
    ...others
  }: {
    /** Text to display prominently. */
    title?: string

    /**
     * Small text displayed below the title. Can be displayed by itself, as
     * well.
     */
    subtitle?: string

    /** Determines color scheme. */
    theme?: "auto" | "light" | "dark"

    /**
     * Sets the width of the card, if desired. Can be any valid CSS `width`
     * value.
     */
    width?: string

    actions?: any
    children?: any
    [key: string]: any
  } = $props()
</script>

<section style:width class="card {theme !== 'auto' ? theme : ''}" {...others}>
  {#if title || subtitle}
    <div class="card-title" role="presentation">
      {#if title}<h1>{title}</h1>{/if}
      {#if subtitle}<small>{subtitle}</small>{/if}
    </div>
  {/if}

  {@render children?.()}

  <div class="card-actions" role="presentation">
    {@render actions?.()}
  </div>
</section>

<style global lang="scss">
  @use "../css/abstracts/mixins" as *;

  .card {
    max-width: 90vw;
    padding: 0.75rem 1rem;
    padding-top: 0.5rem;
    contain: content;
    font-size: 1rem;
    background: var(--col-background);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.5rem;
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
