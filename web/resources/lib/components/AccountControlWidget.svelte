<script lang="ts">
  import WikijumpAPI, { t, authed } from "@wikijump/api"
  import { Button, Card, DetailsMenu } from "@wikijump/components"
  import UserInfo from "./UserInfo.svelte"
</script>

<div class="account-control">
  {#if !$authed}
    <a href="/user--services/login">{$t("auth.LOGIN")}</a>
    <a href="/user--services/register">{$t("auth.CREATE_ACCOUNT")}</a>
  {:else}
    <DetailsMenu hoverable>
      <Button slot="button">
        <UserInfo />
      </Button>

      <Card>
        <Button>{$t("auth.LOGOUT")}</Button>
      </Card>
    </DetailsMenu>
  {/if}
</div>

<style lang="scss">
</style>
