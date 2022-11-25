<script lang="ts">
  import type { PageData } from "./$types"

  export let data: PageData
</script>

<h1>Loaded page</h1>
<p>This is the page loaded with slug <tt>{data.slug}</tt> (<tt>{data.options}</tt>)</p>
