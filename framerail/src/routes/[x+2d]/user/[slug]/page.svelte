<script lang="ts">
  import { page } from "$app/stores"
</script>

<h1>UNTRANSLATED: Loaded user profile</h1>

<textarea class="debug">{JSON.stringify($page, null, 2)}</textarea>

<div class="user-info" data-id={$page.data.user.user_id}>
  {#if $page.data.user.name}
    <h2 class="user-attribute name">
      {$page.data.user.name}
    </h2>
  {/if}

  {#if $page.data.user.real_name}
    <div class="user-attribute real-name">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.real-name"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.real_name}</span>
    </div>
  {/if}

  {#if $page.data.user.gender}
    <div class="user-attribute gender">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.gender"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.gender}</span>
    </div>
  {/if}

  {#if $page.data.user.birthday}
    <div class="user-attribute birthday">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.birthday"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.birthday}</span>
    </div>
  {/if}

  {#if $page.data.user.location}
    <div class="user-attribute location">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.location"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.location}</span>
    </div>
  {/if}

  {#if $page.data.user.user_page}
    <div class="user-attribute user-page">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.user-page"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.user_page}</span>
    </div>
  {/if}

  {#if $page.data.user.biography}
    <div class="user-attribute biography">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.biography"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.biography}</span>
    </div>
  {/if}

  {#if $page.data.user.locales}
    <div class="user-attribute locales">
      <span class="user-attribute-label"
        >{$page.data.internationalization?.["user-profile-info.locales"]}</span
      >
      <span class="user-attribute-value">{$page.data.user.locales?.join(", ")}</span>
    </div>
  {/if}
</div>

<style global lang="scss">
  .debug {
    width: 80vw;
    height: 60vh;
  }
</style>
