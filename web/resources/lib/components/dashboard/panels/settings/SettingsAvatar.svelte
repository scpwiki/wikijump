<script lang="ts">
  import Uppy from "@uppy/core"
  import Dashboard from "@uppy/dashboard"
  import ImageEditor from "@uppy/image-editor"
  import XHRUpload from "@uppy/xhr-upload"
  import { onDestroy, onMount } from "svelte"
  import { format as t } from "@wikijump/fluent"
  import { Button } from "@wikijump/components"
  import UserAvatar from "@/lib/components/UserAvatar.svelte"
  import WikijumpAPI from "@wikijump/api"

  let uppy: Uppy

  onMount(() => {
    const modals = document.getElementById("modals")!

    uppy = new Uppy({
      allowMultipleUploads: false,
      restrictions: {
        maxFileSize: 1024000, // 1024 kb
        maxNumberOfFiles: 1,
        minNumberOfFiles: 1,
        allowedFileTypes: [".jpg", ".jpeg", ".png", ".webp"]
      }
    })
      .use(Dashboard, {
        target: modals,
        trigger: "#show_dashboard",
        closeAfterFinish: true
      })
      .use(ImageEditor, { target: Dashboard })
      .use(XHRUpload, {
        method: "POST",
        endpoint: "/api--v0/user/avatar",
        formData: true,
        fieldName: "avatar",
        bundle: true,
        headers: {
          "Accept": "application/json",
          ...WikijumpAPI.getSecurityHeaders()
        }
      })
  })

  onDestroy(() => {
    if (uppy) uppy.close()
  })
</script>

<div class="dashboard-settings-avatar">
  <h2>{t("avatar")}</h2>
  <UserAvatar size="8rem" />
  <Button id="show_dashboard" primary wide>{t("change")}</Button>
</div>

<style global lang="scss">
  .dashboard-settings-avatar {
    display: flex;
    flex-direction: column;
    justify-content: center;
    width: 8rem;

    @include media(">=normal") {
      position: absolute;
      right: -12rem;
    }

    h2 {
      color: var(--col-text-subtle);
    }

    .button {
      width: auto;
      margin: 0.5rem 1rem;
    }
  }
</style>
