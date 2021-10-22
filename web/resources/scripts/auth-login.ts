import LoginForm from "../lib/components/LoginForm.svelte"
import "./auth.scss"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  const form = new LoginForm({ target: container })

  form.$on("login", () => {
    // TODO: redirect to where the user came from
    // redirect to home
    location.href = "/"
  })
})
