<script lang="ts">
  export let data
</script>

<h1>UNTRANSLATED:Loaded slug page</h1>
<p class="spin-yay">UNTRANSLATED:This is some other page on the site</p>
<p>UNTRANSLATED:Response <textarea class="debug">{JSON.stringify(data)}</textarea></p>

{@html data.compiledHtml}

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

  .debug {
    width: 80vw;
    height: 60vh;
  }
</style>
