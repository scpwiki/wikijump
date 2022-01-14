/* eslint-disable @typescript-eslint/unbound-method */
import { info, rl } from "../pretty-logs"

const _log = console.log

export type CommandFunction = (...args: (string | undefined)[]) => void | Promise<void>

export class Terminal {
  commands = new Map<string, CommandFunction>()

  busy = false

  constructor() {
    this.log = this.log.bind(this)
    this.handle = this.handle.bind(this)
    console.log = this.log
    rl.on("line", this.handle)

    this.addCommand("help", () => {
      info("Available commands:")
      for (const name of this.commands.keys()) {
        console.log(`  ${name}`)
      }
    })
  }

  log(...args: any[]) {
    rl.pause()
    process.stdout.clearLine(-1)
    process.stdout.cursorTo(0)
    _log(...args)
    if (!this.busy) rl.prompt(true)
  }

  addCommand(name: string, fn: CommandFunction) {
    this.commands.set(name, fn)
  }

  async handle(line: string) {
    const args = line.split(/\s+/)
    const command = args.shift()
    const fn = this.commands.get(command!)
    if (fn) {
      this.busy = true
      await fn(...args)
      this.busy = false
      rl.prompt(true)
    } else {
      console.log(`Unknown command: ${command}`)
    }
  }

  close() {
    console.log = _log
    rl.off("line", this.handle)
    this.busy = false
  }
}
