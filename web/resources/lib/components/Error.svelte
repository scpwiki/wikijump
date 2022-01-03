<script lang="ts">
  export let disabled = false
</script>

{#if !disabled}
  <div class="errorbox">
    <slot />
  </div>
{/if}

<style global lang="scss">
  .errorbox {
    margin: 0.5rem 0;
    font-size: 0.75rem;
    color: var(--col-danger);
    text-align: center;
  }
</style>
