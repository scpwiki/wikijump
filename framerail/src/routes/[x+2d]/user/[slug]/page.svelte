<script lang="ts">
  import { page } from "$app/stores"
</script>

<h1>UNTRANSLATED: Loaded user profile</h1>

<textarea class="debug">{JSON.stringify($page, null, 2)}</textarea>

<div class="user-info" data-id={$page.data.user.userId}>
  {#if $page.data.user.name}
    <h2 class="user-attribute name">
      {$page.data.user.name}
    </h2>
  {/if}

  {#if $page.data.user.realName}
    <div class="user-attribute real-name">
      {$page.data.internationalization?.["user-profile-info.name"]} {$page.data.user.realName}
    </div>
  {/if}

  {#if $page.data.user.gender}
    <div class="user-attribute gender">
      {$page.data.internationalization?.["user-profile-info.gender"]} {$page.data.user.gender}
    </div>
  {/if}

  {#if $page.data.user.birthday}
    <div class="user-attribute birthday">
      {$page.data.internationalization?.["user-profile-info.birthday"]} {$page.data.user.birthday}
    </div>
  {/if}

  {#if $page.data.user.location}
    <div class="user-attribute location">
      {$page.data.internationalization?.["user-profile-info.location"]} {$page.data.user.location}
    </div>
  {/if}

  {#if $page.data.user.userPage}
    <div class="user-attribute user-page">
      {$page.data.internationalization?.["user-profile-info.user-page"]} {$page.data.user.userPage}
    </div>
  {/if}

  {#if $page.data.user.biography}
    <div class="user-attribute biography">
      {$page.data.internationalization?.["user-profile-info.biography"]} {$page.data.user.biography}
    </div>
  {/if}
</div>

<style global lang="scss">
  .debug {
    width: 80vw;
    height: 60vh;
  }
</style>