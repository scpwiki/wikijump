<script lang="ts">
  import { contain } from "./lib/actions"
  import Clickable from "./Clickable.svelte"

  const DEFAULT_TAB_TITLE = "Unnamed tab"
  const SELECTED_TAB_HOVER_TEXT = "active"

  let thisElement: HTMLElement
  let tabTitles: string[] = []
  let tabContents: HTMLElement[] = []
  let selectedTabNumber = 0

  /**
   * Sets the current tab selection.
   *
   * @param tabNumber - The tab index to select.
   */
  function selectTab(tabNumber: number) {
    selectedTabNumber = tabNumber
  }

  /**
   * Action that extracts the tab names and contents from the stub component.
   *
   * @param stub - The root element of the stubbed tabview.
   */
  const extractFromStub: SvelteAction = stub => {
    const contents = stub.parentElement?.querySelectorAll<HTMLElement>(".tabview-content")
    if (!contents) return
    contents.forEach(contentElement => {
      // Move them from the DOM into an array
      contentElement.remove()
      tabTitles = [...tabTitles, contentElement.dataset.title ?? DEFAULT_TAB_TITLE]
      tabContents = [...tabContents, contentElement]
    })
  }
</script>

<!-- was #wiki-tabview-[hash].yui-navset.yui-navset-top -->
<div class="tabview" use:extractFromStub>
  <!-- was .yui-nav -->
  <ul class="tabview-selectors">
    {#each tabTitles as tabTitle, tabNumber}
      <li
        class="tabview-selector {tabNumber === selectedTabNumber ? 'selected' : ''}"
        title={tabNumber === selectedTabNumber ? SELECTED_TAB_HOVER_TEXT : undefined}
      >
        <Clickable on:click={() => selectTab(tabNumber)}>
          <!-- was em -->
          {tabTitle}
        </Clickable>
      </li>
    {/each}
  </ul>
  <!-- was .yui-content -->
  <div class="tabview-contents">
    {#each tabContents as tabContent, tabNumber}
      <!-- was #wiki-tab-[id]-[id] -->
      <div
        use:contain={tabContent}
        class="tabview-content {tabNumber === selectedTabNumber ? 'selected' : ''}"
      />
    {/each}
  </div>
</div>

<style>
  .tabview {
    color: green;
  }

  .tabview-selector {
    padding: 5px;
    border: 1px solid black;
    display: inline-block;
  }

  .tabview-content:not(.selected) {
    display: none;
  }
</style>
