<script lang="ts">
  import type { PageData } from "./$types"

  export let data: PageData
</script>

<h1>Loaded forum thread</h1>
<p>This is the forum thread with ID {data.threadId}</p>
