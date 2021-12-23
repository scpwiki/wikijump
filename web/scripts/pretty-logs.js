const readline = require("readline")
const { execSync, spawn } = require("child_process")
const chalk = require("chalk")

function linebreak() {
  console.log("")
}

function separator() {
  console.log(chalk.gray("─────────────────────────"))
}

function section(title, breakBefore = false, breakAfter = breakBefore) {
  if (breakBefore) linebreak()
  const dashes = Math.round((24 - (title.length + 2)) / 2)
  const chrs = "─".repeat(dashes)
  console.log(chalk.blueBright(`${chrs} ${title} ${chrs}`))
  if (breakAfter) linebreak()
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

function shell(command, pipe = true) {
  const child = spawn(command, { shell: true })
  if (pipe) {
    child.stdout.pipe(process.stdout)
    child.stderr.pipe(process.stderr)
  }
  return child
}

function question(question) {
  return new Promise((resolve, reject) => {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    })

    rl.question(chalk.magentaBright(question), answer => {
      // breaks using readline for anything else
      // so we'll just leave the interface alone (technically a memory leak)
      // rl.close()
      resolve(answer)
    })
  })
}

function answerYesOrNo(answer, def = false) {
  answer = answer.toLowerCase().trim()
  if (answer === "y" || answer === "yes") return true
  if (answer === "n" || answer === "no") return false
  return def
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
  shell,
  question,
  answerYesOrNo
}
