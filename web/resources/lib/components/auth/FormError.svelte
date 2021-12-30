<!--
  @component Simple form error message.
  Provide the error message using the `error` prop, and not with a slot.
-->
<script lang="ts">
  /** The form error to display. */
  export let error = ""
</script>

{#if error}
  <div class="form-error">
    {error}
  </div>
{/if}

<style global lang="scss">
  .form-error {
    margin: 0.5rem 0;
    font-size: 0.75rem;
    color: var(--col-danger);
    text-align: center;
  }
</style>
