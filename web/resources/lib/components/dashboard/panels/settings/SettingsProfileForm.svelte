<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import Locale from "@wikijump/fluent"
  import { Form, TextBox, TextInput, Button } from "@wikijump/components"
  import type { Settings } from "../SettingsPanel.svelte"
  import { toast } from "@wikijump/components/lib"

  export let data: Settings

  const t = Locale.makeComponentFormatter("dashboard")

  async function onsubmit(values: {
    realname: string | null
    pronouns: string | null
    birthday: Date | null
    location: string | null
    signature: string | null
    about: string | null
  }) {
    const { realname, pronouns, location, signature, about } = values

    const birthday = values.birthday
      ? values.birthday.toISOString().substring(0, 10)
      : null

    await WikijumpAPI.userClientUpdateProfile({
      realname,
      pronouns,
      birthday,
      location,
      signature,
      about
    })

    toast("success", $t("#-profile.toast"))
  }
</script>

<div class="dashboard-settings-profile">
  <Form {onsubmit} let:busy let:error>
    <div class="dashboard-settings-profile-inputs">
      <TextInput
        name="realname"
        type="text"
        label={$t("#-profile.name")}
        value={data.profile.realname ?? ""}
        maxLength={30}
        novalidate
        clearable
        wide
      />

      <TextInput
        name="pronouns"
        type="text"
        label={$t("#-profile.pronouns")}
        value={data.profile.pronouns ?? ""}
        maxLength={30}
        novalidate
        clearable
        wide
      />

      <TextInput
        name="birthday"
        type="date"
        label={$t("#-profile.birthday")}
        value={data.profile.birthday?.substring(0, 10) ?? ""}
        novalidate
        wide
      />

      <TextInput
        name="location"
        type="text"
        label={$t("#-profile.location")}
        value={data.profile.location ?? ""}
        maxLength={30}
        novalidate
        clearable
        wide
      />

      <TextBox
        name="signature"
        label={$t("#-profile.signature")}
        value={data.profile.signature ?? ""}
        max={80}
        wide
      />

      <TextBox
        name="about"
        label={$t("#-profile.about")}
        value={data.profile.about ?? ""}
        max={2000}
        wide
      />
    </div>

    <Button submit disabled={busy} primary>
      {$t("#-profile.save")}
    </Button>
  </Form>
</div>

<style global lang="scss">
  .dashboard-settings-profile {
    max-width: 30rem;
  }

  .dashboard-settings-profile-inputs {
    display: flex;
    flex-direction: column;
    margin-bottom: 1rem;
  }
</style>
