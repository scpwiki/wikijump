const fs = require("fs")
const path = require("path")
import { svelte } from "@sveltejs/vite-plugin-svelte"
import { defineConfig } from "laravel-vite"
import { SASS_OPTIONS, SVELTE_OPTIONS } from "./scripts/vite-config.js"
import vitePluginToml from "./scripts/vite-plugin-toml.js"
import vitePluginYaml from "./scripts/vite-plugin-yaml.js"

const ROOT = process.cwd()

const entrypoints = fs
  .readdirSync("resources/scripts")
  .filter(ent => fs.statSync(`resources/scripts/${ent}`).isFile())
  .map(file => path.resolve(`${ROOT}/resources/scripts/${file}`))

const modules = fs
  .readdirSync(`${ROOT}/modules`)
  .filter(dir => fs.statSync(`${ROOT}/modules/${dir}`).isDirectory())
  .map(dir => `@wikijump/${dir}`)

// This is a modified return value from the following Artisan command:
// php artisan vite:config
// We need this JSON or else `laravel-vite` will be forced to run the command itself,
// if it does this, it will fail on build, because it will be unable to use PHP in
// the container it runs in.
const PHP_CONFIG = {
  "entrypoints": entrypoints,
  "ignore_patterns": ["/\\.d\\.ts$/"],
  "aliases": { "@": "resources", "@root": "../" },
  "public_directory": "resources/static",
  "ping_timeout": 1000,
  "ping_url": "http://host.docker.internal:3000",
  "build_path": "files--built",
  "dev_url": "",
  "commands": []
}

const config = defineConfig({}, PHP_CONFIG)
  .withPlugins(vitePluginToml, vitePluginYaml, svelte(SVELTE_OPTIONS))
  .merge({
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
      sourcemap: true,
      brotliSize: false,
      cssCodeSplit: true,
      rollupOptions: {
        input: entrypoints
      }
    },

    optimizeDeps: {
      entries: [...entrypoints, "./modules/*/src/**/*.{svelte,js,jsx,ts,tsx}"],
      esbuildOptions: { tsconfig: `${ROOT}/tsconfig.json` },
      exclude: modules
    }
  })
  .merge(env => (env.NODE_ENV === "development" ? { base: "/files--dev/" } : {}))

export default config
