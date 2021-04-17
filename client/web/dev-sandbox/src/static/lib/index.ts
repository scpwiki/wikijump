import { EditorCore } from "cm-editor-core"

window.addEventListener("DOMContentLoaded", async event => {
  const editor = new EditorCore()
  const res = await fetch("/static/misc/ftml-test.ftml")
  if (!res) return
  const src = await res.text()
  await editor.init(document.querySelector(".editor-container")!, src)
})
