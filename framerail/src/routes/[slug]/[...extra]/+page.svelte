<script lang="ts">
  import type { PageData } from "./$types"

  export let data: PageData
</script>

<h1>UNTRANSLATED:Loaded page</h1>
<p>UNTRANSLATED:This is the page loaded with slug <tt>{data.slug}</tt> (<tt>{data.options}</tt>)</p>
