import ResetPasswordForm from "../lib/components/ResetPasswordForm.svelte"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  const form = new ResetPasswordForm({ target: container })

  form.$on("reset", () => {
    window.location.href = "/user--services/login"
  })
})
