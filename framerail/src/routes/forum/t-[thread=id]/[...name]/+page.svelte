<script lang="ts">
  import ForumModuleBody from "$lib/ForumModuleBody.svelte"

  import type { PageData } from "./$types"

  interface Props {
    data: PageData
  }

  let { data }: Props = $props()

  $effect(() => {
    const threadId = data.forumThreadId
    if (!Number.isSafeInteger(threadId) || threadId === null) return
    const globalObject = globalThis as typeof globalThis & {
      WIKIDOT?: { forumThreadId?: number }
    }
    const wikidot = globalObject.WIKIDOT ?? (globalObject.WIKIDOT = {})
    wikidot.forumThreadId = threadId
    return () => {
      if (wikidot.forumThreadId === threadId) delete wikidot.forumThreadId
    }
  })
</script>

<ForumModuleBody body={data.body} threadId={data.forumThreadId} />
