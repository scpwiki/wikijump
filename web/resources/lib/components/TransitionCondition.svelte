<script lang="ts">
  export let condition: boolean
  export let state = condition
  const update = () => (state = condition)
  update()
  function detectDestroy(_node?: Element) {
    return { destroy: update }
  }
</script>

<slot />

{#if state}
  {#if condition}
    <div use:detectDestroy><slot name="true" /></div>
  {/if}
{:else if !condition}
  <div use:detectDestroy><slot name="false" /></div>
{/if}
