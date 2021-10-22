import RegisterForm from "../lib/components/RegisterForm.svelte"
import "./auth.scss"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  const form = new RegisterForm({ target: container })

  form.$on("register", () => {
    // redirect to home
    location.href = "/"
  })
})
