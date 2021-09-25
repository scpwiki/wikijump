import { WikijumpAPI } from "@wikijump/api"
import "@wikijump/ftml-components"
import App from "./App.svelte"

window.addEventListener("DOMContentLoaded", async () => {
  const container = document.querySelector("#app")!
  const app = new App({ target: container })

  setTimeout(async () => {
    try {
      const api = new WikijumpAPI()
      const response = await api.userGet("id", 1234)
      console.log("mocked API response:")
      console.log(response)
    } catch (err) {
      console.warn(err)
      console.log("API probably isn't being mocked")
    }
  }, 1000)
})
