<script lang="ts">
  import Page from "./page.svelte"

  import type { userEditSchema } from "$lib/server/load/user"
  import type { PageProps } from "./$types"
  import type { InferOutput } from "valibot"

  let { data }: PageProps = $props()

  const userData: InferOutput<typeof userEditSchema> = $derived({
    name: data.user?.name ?? "",
    realName: data.user?.real_name ?? "",
    email: data.user?.email ?? "",
    gender: data.user?.gender ?? "",
    birthday: data.user?.birthday ?? "",
    location: data.user?.location ?? "",
    website: data.user?.website ?? "",
    userPage: data.user?.user_page ?? "",
    biography: data.user?.biography ?? "",
    locales: data.user?.locales?.join(" ") ?? ""
  })
</script>

<Page {data} {userData} />
