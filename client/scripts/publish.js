const { section, linebreak, info, question, cmd } = require("./pretty-logs.js")

const arg = process.argv[2]

;(async () => {
  if (arg) {
    cmd(`pnpm pack-module -- ${arg}`)

    section("PUBLISH")
    linebreak()

    const answer = await question("Do you want to publish this package? [y/n] -> ")

    linebreak()

    if (answer.trim().toLowerCase() === "y") {
      process.chdir(`modules/${arg}/dist`)
      cmd("npm publish --access public")

      linebreak()
      info(`Published "${arg}"`)
    } else {
      info(`Aborted publishing "${arg}"`)
    }

    linebreak()
  }
})()
