<script lang="ts">
  import { format as t } from "@wikijump/fluent"
  import { TextInput } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"

  const dispatch = createEventDispatcher()

  let value = ""

  $: if (value) update()

  async function update() {
    dispatch("update", value)
  }

  async function search() {
    dispatch("search", value)
  }
</script>

<div class="search">
  <TextInput
    type="search"
    bind:value
    on:enter={search}
    icon="ic:round-search"
    placeholder={t("search.placeholder")}
    autocomplete="off"
    clearable
    novalidate
    wide
  />
</div>

<style lang="scss">
  .search {
    width: 100%;
  }
</style>
