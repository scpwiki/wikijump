import WikijumpAPI from "@wikijump/api"
import App from "../lib/EditorTest.svelte"

window.addEventListener("DOMContentLoaded", async () => {
  const container = document.querySelector("#app-editor")!
  const app = new App({ target: container })

  setTimeout(async () => {
    try {
      await WikijumpAPI.authLogin({ login: "admin", password: "admin1" })
      const response = await WikijumpAPI.userClientGet()
      console.log("mocked API response:")
      console.log(response)
    } catch (err) {
      console.warn(err)
      console.log("API probably isn't being mocked")
    }
  }, 1000)
})
