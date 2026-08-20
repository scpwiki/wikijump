<script lang="ts">
  import type { userEditSchema } from "$lib/server/load/user"
  import type { InferOutput } from "valibot"
  import type { PageData } from "./$types"

  let {
    data,
    userData
  }: { data: PageData; userData: InferOutput<typeof userEditSchema> } = $props()

  // svelte-ignore state_referenced_locally
  let avatar = $state(
    data.user?.avatar_s3_hash
      ? `https://${data.site_file_domain}/-/avatar/${data.user.user_id}`
      : null
  )

  $effect(() => {
    let url: string | undefined
    // If the user has edited their avatar, use the new avatar
    if (userData?.avatar) {
      url = URL.createObjectURL(userData.avatar)
      avatar = url
    }

    return () => {
      if (url) {
        URL.revokeObjectURL(url)
      }
    }
  })
</script>

<svelte:head>
  <title>{data.user?.name ?? data.user?.slug} | {data?.site?.name}</title>
</svelte:head>

<h1>UNTRANSLATED: Loaded user profile</h1>

<textarea class="debug">{JSON.stringify(data, null, 2)}</textarea>

<div class="user-info" data-id={data.user?.user_id}>
  {#if userData?.name}
    <h2 class="user-attribute name">
      {userData.name}
    </h2>
  {/if}

  {#if userData?.realName}
    <div class="user-attribute real-name">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.real-name"]}</span
      >
      <span class="user-attribute-value">{userData.realName}</span>
    </div>
  {/if}

  {#if userData?.gender}
    <div class="user-attribute gender">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.gender"]}</span
      >
      <span class="user-attribute-value">{userData.gender}</span>
    </div>
  {/if}

  {#if avatar}
    <div class="user-attribute avatar">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.avatar"]}</span
      >
      <img
        class="user-attribute-value"
        alt={data.internationalization?.avatar}
        src={avatar}
      />
    </div>
  {/if}

  {#if userData?.birthday}
    <div class="user-attribute birthday">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.birthday"]}</span
      >
      <span class="user-attribute-value">{userData.birthday}</span>
    </div>
  {/if}

  {#if userData?.location}
    <div class="user-attribute location">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.location"]}</span
      >
      <span class="user-attribute-value">{userData.location}</span>
    </div>
  {/if}

  {#if userData?.website}
    <div class="user-attribute website">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.website"]}</span
      >
      <span class="user-attribute-value">{userData.website}</span>
    </div>
  {/if}

  {#if userData?.userPage}
    <div class="user-attribute user-page">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.user-page"]}</span
      >
      <span class="user-attribute-value">{userData.userPage}</span>
    </div>
  {/if}

  {#if userData?.biography}
    <div class="user-attribute biography">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.biography"]}</span
      >
      <span class="user-attribute-value">{userData.biography}</span>
    </div>
  {/if}

  {#if userData?.locales}
    <div class="user-attribute locales">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.locales"]}</span
      >
      <span class="user-attribute-value">{userData.locales.split(" ").join(", ")}</span>
    </div>
  {/if}
</div>

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }
</style>
