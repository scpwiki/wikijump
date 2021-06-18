const decoder = new TextDecoder()

/** Wrapper for manipulating a document separated by lines. */
export class Reader {
  declare lines: string[]
  declare index: number
  declare line: string

  /** @param input - Document to read. */
  constructor(input: string | Uint8Array) {
    // clean input
    if (typeof input !== "string") input = decoder.decode(input)
    input = input.replaceAll("\r\n", "\n")

    this.lines = input.split("\n")
    this.index = 0
    this.line = this.lines[0]
  }

  /** True if the reader has reached the end of the document. */
  get done() {
    return this.index >= this.lines.length
  }

  /** Advances the reader one line. */
  next() {
    if (this.done) return null
    this.index++
    this.line = this.lines[this.index]
    return true
  }

  /**
   * Advances the reader a given number of steps, calling a callback
   * function for each line it steps through.
   *
   * @param steps - The number of lines to step through.
   * @param cb - The callback function to call. Return false to further advancing.
   */
  for(steps: number, cb: (line: string) => void | boolean) {
    if (!steps) return
    const startIndex = this.index + 1
    for (let idx = 0; idx < steps; idx++) {
      if (this.done) break
      this.index = startIndex + idx
      this.line = this.lines[this.index]
      if (cb(this.line) === false) break
    }
  }
}
