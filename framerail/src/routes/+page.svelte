<script lang="ts">
  import { ping } from '$lib/deepwell.server.ts';
</script>

<style global lang="scss">
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .spin-yay {
    display: inline-block;
    animation: spin 2s linear infinite;
  }
</style>

<h1>Loaded main page</h1>
<p class="spin-yay">This is the page loaded with the start page for the site</p>
<p>
  DEEPWELL ping:
  {#await ping()}
    <code><pre>Loading...</pre></code>
  {:then result}
    <code><pre>{result}</pre></code>
  {:catch error}
    <span style="color: red;">{error.message}</span>
  {/await}
</p>
