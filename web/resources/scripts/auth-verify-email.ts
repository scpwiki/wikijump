import ResendButton from "../lib/components/ResendVerificationEmail.svelte"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  new ResendButton({ target: container })
})
