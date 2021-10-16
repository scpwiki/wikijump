import WikijumpAPI, { isAuthenticated } from "@wikijump/api"
import { toast } from "@wikijump/components"
import App from "../lib/EditorTest.svelte"

window.addEventListener("DOMContentLoaded", async () => {
  const container = document.querySelector("#app-editor")!
  const app = new App({ target: container })

  setTimeout(async () => {
    try {
      if (!isAuthenticated()) {
        await WikijumpAPI.authLogin({ login: "admin", password: "admin1" })
      }
      const response = await WikijumpAPI.authCheck()
      console.log("API response:")
      console.log(response)
    } catch (err) {
      console.warn(err)
    }

    toast("success", "TOAST!")
    toast("info", "Toast?", 2000)
  }, 1000)
})
