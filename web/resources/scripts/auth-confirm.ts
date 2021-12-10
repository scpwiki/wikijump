import ConfirmForm from "../lib/components/ConfirmPasswordForm.svelte"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  const form = new ConfirmForm({ target: container })

  form.$on("confirm", () => {
    const backURL =
      document.getElementById("app_auth")?.getAttribute("data-back-url") || "/"

    location.href = backURL
  })
})
