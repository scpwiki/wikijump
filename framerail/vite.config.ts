import { sveltekit } from "@sveltejs/kit/vite"
import type { UserConfig } from "vite"

const config: UserConfig = {
  server: {
    host: "::",
    port: 3000,
    strictPort: true
  },
  plugins: [sveltekit()]
}

export default config
