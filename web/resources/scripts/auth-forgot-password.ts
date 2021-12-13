import ForgotPasswordForm from "../lib/components/ForgotPasswordForm.svelte"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  new ForgotPasswordForm({ target: container })
})
