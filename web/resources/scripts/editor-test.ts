import WikijumpAPI, { isAuthenticated } from "@wikijump/api"
import { toast } from "@wikijump/components"
import App from "../lib/EditorTest.svelte"

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

  toast("danger", "dangerous toast!", 0)
  toast("info", "Toast?", 0)
  toast("success", "Toast!", 0)
  toast("warning", "oh no... toast", 0)
  setTimeout(() => toast("info", "see ya later"), 1000)
}, 1000)
