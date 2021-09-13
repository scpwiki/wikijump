const { execSync } = require("child_process")

const arg = process.argv[2]

if (arg) {
  execSync(`pnpm pack-module -- ${arg}`, { stdio: "inherit" })
  process.chdir(`modules/${arg}/dist`)
  execSync(`npm publish --access public`, { stdio: "inherit" })
}
