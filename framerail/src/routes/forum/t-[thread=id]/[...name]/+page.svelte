<script lang="ts">
  import type { PageData } from "./$types"

  interface Props {
    data: PageData
  }

  let { data }: Props = $props()
</script>

<h1>{data.internationalization?.["forum-thread"]}</h1>
<p>{data.internationalization?.["forum-thread.loaded"]}</p>
