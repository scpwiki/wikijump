import LoginForm from "../lib/components/LoginForm.svelte"

window.addEventListener("DOMContentLoaded", () => {
  const container = document.getElementById("auth_form_container")
  if (!container) throw new Error("No container found")
  const form = new LoginForm({ target: container })

  form.$on("login", () => {
    const backURL =
      document.getElementById("app_auth")?.getAttribute("data-back-url") || "/"

    location.href = backURL
  })
})
