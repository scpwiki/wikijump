import "@wikijump/ftml-components"
import App from "./App.svelte"

window.addEventListener("DOMContentLoaded", async () => {
  const container = document.querySelector("#app")!
  const app = new App({ target: container })
})
