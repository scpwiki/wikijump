import { svelte } from "@sveltejs/vite-plugin-svelte"
import { defineConfig } from "laravel-vite"
import { SASS_OPTIONS, SVELTE_OPTIONS } from "./scripts/vite-config"
import vitePluginToml from "./scripts/vite-plugin-toml"
import vitePluginYaml from "./scripts/vite-plugin-yaml"

// HACK
// This is a _hard-coded return value_ from the following Artisan command:
// php artisan vite:config
// We need this JSON or else `laravel-vite` will be forced to run the command itself,
// if it does this, it will fail on build, because it will be unable to use PHP in
// the container it runs in.
// TODO: remove this if possible
const PHP_CONFIG = {
  "entrypoints": ["resources/scripts/index.ts"],
  "ignore_patterns": ["/\\.d\\.ts$/"],
  "aliases": { "@": "resources" },
  "public_directory": "resources/static",
  "ping_timeout": 10,
  "ping_url": null,
  "build_path": "build",
  "dev_url": "http://localhost:3000",
  "commands": []
}

export default defineConfig({}, PHP_CONFIG)
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
