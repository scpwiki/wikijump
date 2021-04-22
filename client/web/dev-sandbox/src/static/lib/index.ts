import { SheafCore } from "sheaf-core"
import { FTMLLanguage } from "cm-lang-ftml"
import { perfy } from "wj-util"
import * as FTML from "ftml-wasm-worker"

window.addEventListener("DOMContentLoaded", async () => {
  const editor = new SheafCore()
  const res = await fetch("/static/misc/ftml-test.ftml")
  if (!res) return
  const src = await res.text()
  await editor.init(document.querySelector(".editor-container")!, src, [
    FTMLLanguage.load()
  ])
  editor.subscribe(({ value }) => {
    ;(async () => {
      const log = perfy("ftml-perf", 5)
      console.log(await FTML.render(value))
      log()
    })()
  })
})
