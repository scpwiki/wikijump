<script lang="ts">
  import { onMount } from "svelte"

  let thisElement: HTMLElement
  let tabs: string[] = []
  let tabContents: string[] = []
  let selectedTabNumber = 0

  onMount(() => {
    // Find all the tab stubs
    const contents =
      thisElement.parentElement?.querySelectorAll<HTMLElement>(".wj-tabview-content")
    if (!contents) return
    contents.forEach(contentElement => {
      // Move them from the DOM into an array
      contentElement.remove()
      tabs = [...tabs, contentElement.dataset.title ?? ""]
      tabContents = [...tabContents, contentElement.innerHTML]
      // The contents of the tab is stored as innerHTML as a string, which is
      // not super-performant. Ideally, the elements would be stored as
      // DOMElement objects and then bound to the tab content elements in
      // their 'this' property. However, in Svelte, bind:this is one-way,
      // used only to get a reference to a given element, not to set that
      // reference.
    })
  })

  function selectTab(tabIndex: number) {
    console.log("Switching to tab", tabIndex)
    selectedTabNumber = tabIndex
  }
</script>

<div class="tabview" bind:this={thisElement}>
  <div class="tabs">
    {#each tabs as tab, tabIndex}
      <div class="tab-selector" on:click={() => selectTab(tabIndex)}>
        {tab}
      </div>
    {/each}
  </div>
  <div class="tab-contents">
    {#each tabContents as tabContent, tabNumber}
      <div class="tab-content {tabNumber === selectedTabNumber ? 'selected' : ''}">
        {@html tabContent}
      </div>
    {/each}
  </div>
</div>

<style>
  .tabview {
    color: green;
  }

  .tab-selector {
    display: inline-block;
    padding: 5px;
    border: 1px solid black;
  }

  .tab-content:not(.selected) {
    display: none;
  }
</style>
