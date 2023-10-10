import { sveltekit } from "@sveltejs/kit/vite"
import { statSync } from "fs"
import { resolve } from "path"
import type { UserConfig } from "vite"

function resolveAssets() {
  try {
    let globalAssets = statSync(resolve(__dirname, "../assets"))
    if (globalAssets.isDirectory()) return resolve(__dirname, "../assets")
    else return resolve(__dirname, "src/assets")
  } catch (error) {
    return resolve(__dirname, "src/assets")
  }
}

const config: UserConfig = {
  server: {
    host: "::",
    port: 3000,
    strictPort: true
  },
  plugins: [sveltekit()],
  resolve: {
    alias: {
      "$static": resolve(__dirname, "static"),
      "$assets": resolveAssets()
    }
  }
}

export default config
