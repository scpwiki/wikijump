const { defineConfig } = require("laravel-vite")
const { PHP_CONFIG, BaseConfig } = require("./scripts/vite-utils.js")

const config = defineConfig({}, PHP_CONFIG).merge(BaseConfig())

export default config
