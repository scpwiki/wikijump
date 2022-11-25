<script lang="ts">
  import type { PageData } from "./$types"

  export let data: PageData
</script>

<h1>Loaded forum category</h1>
<p>This is the forum category with ID {data.categoryId}</p>
