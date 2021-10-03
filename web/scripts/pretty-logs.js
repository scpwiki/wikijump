const readline = require("readline")
const { execSync } = require("child_process")
const chalk = require("chalk")

function linebreak() {
  console.log("")
}

function separator() {
  console.log(chalk.gray("─────────────────────────"))
}

function section(title) {
  const dashes = Math.round((24 - (title.length + 2)) / 2)
  const chrs = "─".repeat(dashes)
  console.log(chalk.blueBright(`${chrs} ${title} ${chrs}`))
}

function info(...msgs) {
  const msg = msgs.join("\n")
  console.log(chalk.green(msg))
}

function look(...msgs) {
  const msg = msgs.join("\n")
  console.log(chalk.magentaBright(msg))
}

function warn(...msgs) {
  const msg = msgs.join("\n")
  console.warn(chalk.yellow("──────── WARNING ────────"))
  console.warn(chalk.yellow(msg))
}

function error(...msgs) {
  const msg = msgs.join("\n")
  console.warn(chalk.redBright("───────── ERROR ─────────"))
  console.warn(chalk.redBright(msg))
}

function cmd(command) {
  execSync(command, { stdio: "inherit" })
}

function question(question) {
  return new Promise((resolve, reject) => {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    })

    rl.question(chalk.magentaBright(question), answer => {
      rl.close()
      resolve(answer)
    })
  })
}

module.exports = {
  chalk,
  linebreak,
  separator,
  section,
  info,
  look,
  warn,
  error,
  cmd,
  question
}
