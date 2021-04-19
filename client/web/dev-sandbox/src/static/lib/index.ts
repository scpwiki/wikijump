import { SheafCore } from "sheaf-core"

window.addEventListener("DOMContentLoaded", async () => {
  const editor = new SheafCore()
  const res = await fetch("/static/misc/ftml-test.ftml")
  if (!res) return
  const src = await res.text()
  await editor.init(document.querySelector(".editor-container")!, src)
})
