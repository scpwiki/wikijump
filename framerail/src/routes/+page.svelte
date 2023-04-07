<script lang="ts">
  export let data;
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
<p>DEEPWELL ping: {JSON.stringify(data.ping)}</p>
