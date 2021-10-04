import { svelte } from "@sveltejs/vite-plugin-svelte"
import { defineConfig } from "laravel-vite"
import { SASS_OPTIONS, SVELTE_OPTIONS } from "./scripts/vite-config"
import vitePluginToml from "./scripts/vite-plugin-toml"
import vitePluginYaml from "./scripts/vite-plugin-yaml"

export default defineConfig()
  .withPlugins(vitePluginToml, vitePluginYaml)
  .withPlugin(svelte(SVELTE_OPTIONS))
  .merge({
    css: {
      preprocessorOptions: {
        scss: SASS_OPTIONS
      }
    },

    build: {
      target: "esnext",
      minify: "esbuild",
      brotliSize: false,
      cssCodeSplit: false
    }
  })
