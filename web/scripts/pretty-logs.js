const readline = require("readline")
const { spawn, spawnSync, execSync } = require("child_process")
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

function infoline(...msgs) {
  linebreak()
  const msg = msgs.join("\n")
  console.log(chalk.green(msg))
  linebreak()
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

function shellParams(command) {
  let file, args
  let options = { stdio: "inherit" }
  if (process.platform === "win32") {
    file = "cmd.exe"
    args = ["/s", "/c", `"${command}"`]
    options.windowsVerbatimArguments = true
  } else {
    file = "/bin/sh"
    args = ["-c", command]
  }
  return [file, args, options]
}

function shellSync(command) {
  return spawnSync(...shellParams(command))
}
function shell(command) {
  return spawn(...shellParams(command))
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
  infoline,
  look,
  warn,
  error,
  cmd,
  shellSync,
  shell,
  question
}
