<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"
  import {
    addLocalePreference,
    moveLocalePreference,
    removeLocalePreference
  } from "$lib/user-locale-order.js"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()
  let savedLocales = $state(
    untrack(() => [...(data.displaySettingsForm.data.locales ?? ["en"])])
  )
  let savedSignature = $state(
    untrack(() => data.user_session?.user.forum_signature ?? "")
  )
  let localeToAdd = $state("")
  let localeOrderAnnouncement = $state("")
  let availableLocales = $derived(
    data.userInterfaceLocales.filter(({ value }) => !$form.locales.includes(value))
  )

  const localeLabel = (value: string) =>
    data.userInterfaceLocales.find(({ value: localeValue }) => localeValue === value)
      ?.label ?? `${value} (current)`

  const announceLocaleOrder = () => {
    localeOrderAnnouncement = `Language order: ${$form.locales
      .map(
        (value, index) =>
          `${index === 0 ? "Primary" : `Fallback ${index}`} ${localeLabel(value)}`
      )
      .join(", ")}`
  }

  const { form, enhance } = superForm(
    untrack(() => data.displaySettingsForm),
    {
      onResult: async ({ result }) => {
        if (result.type === "success") {
          savedLocales = [...$form.locales]
          savedSignature = $form.signature
          await invalidateAll()
        } else if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data
          }
        }
      }
    }
  )
</script>

<h1>{data.internationalization?.settings}</h1>

<form id="user-settings-form" action="?/display" method="POST" use:enhance>
  <p id="user-display-locales-label">
    {data.internationalization?.["user-profile-info.locales"]}
  </p>
  <ol id="user-display-locales" aria-labelledby="user-display-locales-label">
    {#each $form.locales as localeValue, index (localeValue)}
      <li class="locale-preference">
        <input type="hidden" name="locales" value={localeValue} />
        <span>
          {index === 0 ? "Primary" : `Fallback ${index}`}: {localeLabel(localeValue)}
        </span>
        <button
          class="action-button button-order clickable"
          type="button"
          aria-label={`Move ${localeLabel(localeValue)} up`}
          disabled={index === 0}
          onclick={() => {
            $form.locales = moveLocalePreference($form.locales, localeValue, -1)
            announceLocaleOrder()
          }}>↑</button
        >
        <button
          class="action-button button-order clickable"
          type="button"
          aria-label={`Move ${localeLabel(localeValue)} down`}
          disabled={index === $form.locales.length - 1}
          onclick={() => {
            $form.locales = moveLocalePreference($form.locales, localeValue, 1)
            announceLocaleOrder()
          }}>↓</button
        >
        <button
          class="action-button button-order clickable"
          type="button"
          aria-label={`Remove ${localeLabel(localeValue)}`}
          disabled={$form.locales.length === 1}
          onclick={() => {
            $form.locales = removeLocalePreference($form.locales, localeValue)
            announceLocaleOrder()
          }}>×</button
        >
      </li>
    {/each}
  </ol>
  <div class="locale-add-row">
    <label for="user-display-locale-add">Add a display language</label>
    <select id="user-display-locale-add" bind:value={localeToAdd}>
      <option value="">Choose a language</option>
      {#each availableLocales as locale (locale.value)}
        <option value={locale.value}>{locale.label}</option>
      {/each}
    </select>
    <button
      class="action-button button-order clickable"
      type="button"
      disabled={!localeToAdd}
      onclick={() => {
        $form.locales = addLocalePreference($form.locales, localeToAdd)
        localeToAdd = ""
        announceLocaleOrder()
      }}>Add</button
    >
  </div>
  <p class="visually-hidden" role="status" aria-live="polite">
    {localeOrderAnnouncement}
  </p>
  <label for="forum-signature-source"> Forum signature </label>
  <textarea
    id="forum-signature-source"
    name="signature"
    maxlength="400"
    rows="4"
    bind:value={$form.signature}></textarea>
  <p class="settings-note">
    400 characters maximum. Only 4 lines. Wiki syntax is supported.
  </p>
  <div class="action-row user-settings-actions">
    <button
      class="action-button button-cancel clickable"
      onclick={() => {
        $form.locales = [...savedLocales]
        localeToAdd = ""
        $form.signature = savedSignature
      }}
      type="button"
    >
      {data.internationalization?.cancel}
    </button>
    <button class="action-button button-save clickable" type="submit">
      {data.internationalization?.save}
    </button>
  </div>
</form>

<style lang="scss">
  #user-settings-form {
    display: grid;
    gap: 0.75rem;
    max-width: 40rem;
  }

  #forum-signature-source {
    min-height: 7rem;
  }

  .user-settings-actions {
    display: flex;
    gap: 0.5rem;
  }

  #user-display-locales {
    display: grid;
    gap: 0.5rem;
    list-style: decimal;
    margin: 0;
    padding-inline-start: 1.75rem;
  }

  .locale-preference,
  .locale-add-row {
    align-items: center;
    display: flex;
    gap: 0.5rem;
  }

  .locale-preference span {
    flex: 1;
  }

  .visually-hidden {
    border: 0;
    clip: rect(0, 0, 0, 0);
    height: 1px;
    margin: -1px;
    overflow: hidden;
    padding: 0;
    position: absolute;
    white-space: nowrap;
    width: 1px;
  }
</style>
