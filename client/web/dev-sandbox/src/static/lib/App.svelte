<script lang="ts">
  import { Sheaf } from "@wikijump/sheaf"
  import { onMount } from "svelte"

  let ready = false
  let doc = ""

  onMount(async () => {
    if (!doc) {
      const res = await fetch("/static/misc/ftml-test-cases-export.ftml")
      if (!res) return
      doc = await res.text()
    }
    ready = true
  })
</script>

<div>
  {#if ready}<Sheaf {doc} height="100vh" />{/if}
</div>
