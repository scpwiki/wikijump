<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import Locale, { LOCALE_LIST } from "@wikijump/fluent"
  import { Select, Form, Toggle, TextInput, Button } from "@wikijump/components"
  import type { Settings } from "../SettingsPanel.svelte"
  import { toast } from "@wikijump/components/lib"

  const t = Locale.makeComponentFormatter("dashboard")

  export let data: Settings

  const locales: { code: string; option: string }[] = []

  function displayName(basis: string, locale: string) {
    return new Intl.DisplayNames(basis, {
      type: "language",
      // @ts-ignore - outdated typings
      languageDisplay: "standard"
    }).of(locale)
  }

  for (const code of LOCALE_LIST) {
    const native = displayName(code, code)
    const option = `${code.toUpperCase()} ─ ${native}`
    locales.push({ code, option })
  }

  async function onsubmit(values: {
    username: string
    language: string[]
    allowMessages: ("registered" | "co-members" | "nobody")[]
    acceptsInvites: boolean
  }) {
    const { acceptsInvites } = values

    const language = values.language[0]

    const allowMessages = values.allowMessages[0]

    await WikijumpAPI.accountUpdateSettings({ language, allowMessages, acceptsInvites })

    toast("success", $t("#-account.toast"))
  }
</script>

<div class="dashboard-settings-account">
  <Form {onsubmit} let:busy>
    <div class="dashboard-settings-account-inputs">
      <!-- TODO -->
      <TextInput
        name="username"
        type="text"
        label={$t("#-account.username")}
        value={data.profile.username}
        maxLength={30}
        novalidate
        required
        wide
      />

      <Select
        name="language"
        label={$t("#-account.language")}
        value={data.account.language}
        required
        wide
      >
        {#each locales as { code, option }}
          <option value={code}>{option}</option>
        {/each}
      </Select>

      <Select
        name="allowMessages"
        label={$t("#-account.allow-messages")}
        value={data.account.allowMessages}
        required
        wide
      >
        <option value="registered">{$t("#-account-allow-messages.registered")}</option>
        <option value="co-members">{$t("#-account-allow-messages.co-members")}</option>
        <option value="nobody">{$t("#-account-allow-messages.nobody")}</option>
      </Select>

      <Toggle name="acceptsInvites" type="checkbox" toggled={data.account.acceptsInvites}>
        {$t("#-account.accepts-invites")}
      </Toggle>
    </div>

    <Button submit disabled={busy} primary>
      {$t("#-account.save")}
    </Button>
  </Form>
</div>

<style global lang="scss">
  .dashboard-settings-account {
    max-width: 30rem;
  }

  .dashboard-settings-account-inputs {
    display: flex;
    flex-direction: column;

    .toggleinput {
      margin: 1rem 0;
    }
  }
</style>
