<script lang="ts">
  import { goto } from "$app/navigation"
  import { onMount } from "svelte"
  export let redirectPage = "/"

  onMount(() => {
    goto(redirectPage, {
      replaceState: true
    })
  })
</script>
