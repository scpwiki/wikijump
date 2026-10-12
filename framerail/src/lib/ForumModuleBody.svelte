<script lang="ts">
  import { wikidotForumThread } from "$lib/wikidot/wikidot-forum-thread.js"

  interface Props {
    body: string
    /** Set for a thread body so its read-only Wikidot controls work. */
    threadId?: number | null
  }

  let { body, threadId = undefined }: Props = $props()
</script>

{#if threadId === undefined}
  <div id="page-content">{@html body}</div>
{:else}
  <div id="page-content" use:wikidotForumThread={{ threadId }}>{@html body}</div>
{/if}
