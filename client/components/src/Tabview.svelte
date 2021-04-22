<script>
  import { onMount } from "svelte"

  let thisElement
  let tabs = []
  let tabContents = []
  let selectedTabNumber = 0

  onMount(() => {
    // Find all the tab stubs
    const contents = thisElement.parentElement.querySelectorAll(".wj-tabview-content")
    for (const contentElement of contents) {
      // Move them from the DOM into an array
      contentElement.remove()
      tabs = [...tabs, contentElement.dataset.title]
      tabContents = [...tabContents, contentElement.innerHTML]
      // The contents of the tab is stored as innerHTML as a string, which is
      // not super-performant. Ideally, the elements would be stored as
      // DOMElement objects and then bound to the tab content elements in
      // their 'this' property. However, in Svelte, bind:this is one-way,
      // used only to get a reference to a given element, not to set that
      // reference.
    }
  })

  function selectTab(tabIndex) {
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
    padding: 5px;
    border: 1px solid black;
    display: inline-block;
  }

  .tab-content:not(.selected) {
    display: none;
  }
</style>
