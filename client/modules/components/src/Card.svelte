<script lang="ts">
  export let title = ""
  export let subtitle = ""
  export let theme: "light" | "dark" = "light"
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
  @import "../../wj-css/src/abstracts";

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
