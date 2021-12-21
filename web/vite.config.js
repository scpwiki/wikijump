const { defineConfig } = require("laravel-vite")
const { PHP_CONFIG, BASE_CONFIG } = require("./scripts/vite-utils.js")

const config = defineConfig({}, PHP_CONFIG).merge(BASE_CONFIG)

export default config
