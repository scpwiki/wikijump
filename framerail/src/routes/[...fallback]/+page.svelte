<!-- //[...fallback] should not be a valid page, therefore redirects to corresponding page -->

<script lang="ts">
  import { page } from "$app/stores"
  import Redirect from "$lib/redirect.svelte"
  let redirectPage =
    `/${ 
    $page.params.fallback
      .split("/")
      .filter((v) => v.length)
      .join("/")}`
</script>

<Redirect {redirectPage} />
