<script lang="ts">
  import type { PageData } from "./$types"

  interface Props {
    data: PageData
  }

  let { data }: Props = $props()
</script>

<h1>{data.internationalization?.["forum-category"]}</h1>
<p>{data.internationalization?.["forum-category.loaded"]}</p>
