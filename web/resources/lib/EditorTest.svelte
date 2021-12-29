<script lang="ts">
  import { asset } from "@wikijump/api"
  import { Sheaf } from "@wikijump/sheaf"
  import { onMount } from "svelte"

  let ready = false
  let doc = ""

  onMount(async () => {
    if (!doc) {
      const res = await asset("/misc/ftml-test-cases-export.ftml").text()
      if (!res) return
      doc = res
    }
    ready = true
  })
</script>

<div>
  {#if ready}<Sheaf {doc} height="100vh" />{/if}
</div>
