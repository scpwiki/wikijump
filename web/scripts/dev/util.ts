import type { ChildProcessWithoutNullStreams as ChildProcess } from "child_process"
import { cmdAsync as cmd, pc, shell } from "../pretty-logs"

const args = process.argv.slice(2)

export const isServe = args.includes("serve")
export const isSudo = args.includes("sudo")
export const isBuild = args.includes("build")

export async function pnpm(args: string, pipe = true, cd?: string) {
  if (cd) await cmd(`cd ${cd} && pnpm -s ${args}`, pipe)
  else await cmd(`pnpm -s ${args}`, pipe)
}

export async function compose(args: string): Promise<void>
export async function compose(args: string, asShell: true): Promise<ChildProcess>
export async function compose(args: string, asShell = false) {
  const str = `compose${isSudo ? "-sudo" : ""} -- ${args}`
  return asShell ? shell(str, false) : await pnpm(str)
}

export class ProgressLine {
  constructor(line?: string) {
    if (line) this.update(line)
  }

  update(line: string) {
    process.stdout.clearLine(-1)
    process.stdout.cursorTo(0)
    process.stdout.write(line)
  }

  break(line?: string) {
    if (line) this.update(line)
    process.stdout.write("\n")
  }
}

export async function starting<T>(name: string, promise: Promise<T>) {
  const progress = new ProgressLine(`Starting ${name} ...`)
  try {
    const result = await promise
    progress.break(`Starting ${name} ... ${pc.green("done")}`)
    return result
  } catch (error) {
    progress.break(`Starting ${name} ... ${pc.red("failed")}}}`)
    throw error
  }
}

export async function closing<T>(name: string, promise: Promise<T>) {
  const progress = new ProgressLine(`Stopping ${name} ...`)
  try {
    const result = await promise
    progress.break(`Stopping ${name} ... ${pc.green("done")}`)
    return result
  } catch (error) {
    progress.break(`Stopping ${name} ... ${pc.red("failed")}}}`)
    throw error
  }
}
