import pkg from "./package.json"

import { sveltekit } from "@sveltejs/kit/vite"
import { execSync } from "child_process"

import type { UserConfig } from "vite"

let pnpmVersion = null
try {
  pnpmVersion = execSync("pnpm -v").toString("utf-8").trim()
} catch {
  // ignore pnpm version if there are errors
}

const config: UserConfig = {
  server: {
    host: "::",
    port: 3393,
    strictPort: true,

    // This setting was added to avoid a security issue:
    // https://github.com/vitejs/vite/security/advisories/GHSA-vg6x-rcgg-rjx6
    //
    // Normally this should be a list but setting it to "true" disables the check.
    // After discussion, this is acceptable because:
    // 1. Vite is only used in development, not in deployed instances.
    // 2. In our stack, Caddy receives requests and reverse proxies them
    //    to our web servers, wws and framerail. The configuration already
    //    handles domain -> site routing logic, so hostile domains are not
    //    able to utilize this configuration field since they will not be
    //    in the generated Caddyfile.
    //
    //    Essentially, caddy is acting as our "allowedHosts" for us.
    allowedHosts: true
  },
  plugins: [sveltekit()],
  define: {
    // also update $lib/vite-env.d.ts if these defines are changed
    serverInfo: {
      pnpmVersion,
      frontendName: pkg.name ?? null,
      frontendVersion: pkg.version ?? null,
      frontendDescription: pkg.description ?? null,
      frontendRepository: pkg.repository ?? null,
      frontendLicense: pkg.license ?? null
    }
  }
}

export default config
