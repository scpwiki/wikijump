import { svelte } from "@sveltejs/vite-plugin-svelte"
import { defineConfig } from "laravel-vite"
import { SASS_OPTIONS, SVELTE_OPTIONS } from "../scripts/vite-config.js"
import vitePluginToml from "../scripts/vite-plugin-toml.js"
import vitePluginYaml from "../scripts/vite-plugin-yaml.js"

// This is a modified return value from the following Artisan command:
// php artisan vite:config
// We need this JSON or else `laravel-vite` will be forced to run the command itself,
// if it does this, it will fail on build, because it will be unable to use PHP in
// the container it runs in.
const PHP_CONFIG = {
  "entrypoints": ["scripts/index.ts"],
  "ignore_patterns": ["/\\.d\\.ts$/"],
  "aliases": { "@": "resources" },
  "public_directory": "resources/static",
  "ping_timeout": 1000,
  "ping_url": "http://host.docker.internal:3000",
  "build_path": "build",
  "dev_url": "http://localhost:3000",
  "commands": []
}

export default defineConfig({}, PHP_CONFIG)
  .withPlugins(vitePluginToml, vitePluginYaml, svelte(SVELTE_OPTIONS))
  .merge({
    root: "../",

    server: {
      fs: { strict: false }
    },

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
