import LoginForm from "../lib/components/LoginForm.svelte"
import "./login.scss"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("login_form_container")
  if (!container) throw new Error("No container found")
  const form = new LoginForm({ target: container })
})
