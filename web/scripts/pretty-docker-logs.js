const pc = require("picocolors")

const LEFT_LENGTH = 10

/*
 * This script's `formatLogs` function reformats and recolorizes
 * `docker-compose` logging. This is a regex abomination, but it
 * ultimately makes the CLI output much more readable. If this
 * starts breaking something important, feel free to disable/delete
 * this script.
 */

/**
 * Formats logs emitted on `stdout` by `docker-compose log`.
 *
 * @param {Buffer} data
 */
function formatLogs(data) {
  const logs = parseLogs(data)

  const lines = []

  for (let { name, time, message, followup, level } of logs) {
    let prefix = name.padEnd(LEFT_LENGTH)

    prefix = prefix.replace(name, colorName(name))

    let line = `${prefix}${pc.gray("│")} `

    if (time) line += `${pc.dim(`[${time}]`)} `

    if (level) {
      // prettier-ignore
      switch (level.toLowerCase()) {
        case "info":    line += pc.bold(pc.green(level))  ; break
        case "notice":  line += pc.bold(pc.green(level))  ; break
        case "warn":    line += pc.bold(pc.yellow(level)) ; break
        case "warning": line += pc.bold(pc.yellow(level)) ; break
        case "error":   line += pc.bold(pc.red(level))    ; break
        case "debug":   line += pc.bold(pc.blue(level))   ; break
        default:        line += pc.bold(pc.gray(level))   ; break
      }
      line += " "
    }

    if (message && level) {
      // prettier-ignore
      switch (level.toLowerCase()) {
        case "warn":    line += pc.yellow(message) ; break
        case "warning": line += pc.yellow(message) ; break
        case "error":   line += pc.red(message)    ; break
        default:        line += message            ; break
      }
    } else {
      line += message
    }

    if (followup) {
      line += "\n"
      line += followup
        .map(line => `${prefix}${pc.gray("╎")}     ${pc.green(line)}`)
        .join("\n")
    }

    lines.push(line)
  }

  return lines.join("\n")
}

function parseLogs(data) {
  return data
    .toString()
    .split(/\r?\n/)
    .filter(line => line.length > 0)
    .map(parseLine)
    .filter(line => line !== null)
}

function parseLine(line) {
  let [name, log] = line.split(/\s*\|\s*/)

  if (!name || !log) return null

  let time = ""
  let message = ""
  let followup = null
  let level = ""

  if (name.endsWith("_1")) name = name.slice(0, -2)
  if (name.endsWith("-1")) name = name.slice(0, -2)

  if (log.startsWith("{") && log.endsWith("}")) {
    try {
      const parsed = JSON.parse(log)
      time = parsed.time || parsed.timestamp || parsed.datetime || ""
      message = parsed.msg || parsed.message || ""
      level = parsed.level_name || ""

      // replace double line breaks in message
      message = message.replaceAll(/\r?\n\r?\n/g, "\n")
    } catch {
      message = log
    }
  }
  // starting timestamp
  else if (/^\[.*?\]/.test(log)) {
    time = log.match(/^\[(.*?)\]/)[1]
    message = log.replace(/^\[.*?\]\s*/, "")
  } else {
    message = log
  }

  if (message && /\r?\n/.test(message)) {
    let [first, ...rest] = message.split(/\r?\n/)
    rest = rest.filter(line => line.length !== 0)
    message = first
    followup = rest
  }

  if (time) {
    try {
      const date = new Date(time)
      if (date.toTimeString() === "Invalid Date") throw new Error()
      time = date.toTimeString().split(" ")[0]
    } catch {
      // couldn't parse, so we'll try to clean it a little bit instead
      time = time.replace(" +0000", "")
    }
  }

  return { name, time, message, followup, level }
}

function colorName(name) {
  // prettier-ignore
  switch (name) {
    case "nginx":    return pc.white(name)
    case "php-fpm":  return pc.blue(name)
    case "api":      return pc.cyan(name)
    case "cache" :   return pc.yellow(name)
    case "database": return pc.magenta(name)
  }

  return name
}

module.exports = { formatLogs }
