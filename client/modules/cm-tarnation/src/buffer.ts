import type { Tree } from "lezer-tree"
import { search, SearchOpts } from "wj-util"
import { ParserElementStack, ParserStack, SerializedEmbedded } from "./parser"
import { SerializedTokenizerStack, TokenizerStack } from "./tokenizer"

// ---- CONTEXT

/**
 * State storage class for Tarnation. The parser and tokenizer should be
 * able to restart if given the information available in this object.
 */
export class Context {
  constructor(
    public pos: number,
    public tokenizer: TokenizerStack = new TokenizerStack(),
    public parser: ParserStack = new ParserStack(),
    public embed: SerializedEmbedded = { pending: [], parsers: [] }
  ) {}
}

// ---- BUFFER

/**
 * Represents a Lezer token. The `tree` value is for storing a reusable
 * form of this token and its children.
 */
type TokenData = [id: number, from: number, to: number, children: number, tree?: Tree]

/** @see {@link Buffer} */
type BufferElement = BufferToken | Checkpoint

/**
 * Stores the token and checkpoint information about the previous parse.
 * New tokens, and context snapshots ({@link Checkpoint}), are stored in this buffer.
 */
export class Buffer {
  /** The latest (by position) checkpoint in the buffer. */
  declare lastCheckpoint?: Checkpoint

  constructor(
    /** The raw array storing the buffer's data. */
    public buffer: BufferElement[] = []
  ) {}

  /** Number of elements in the buffer. */
  get length() {
    return this.buffer.length
  }

  /** The last node in the buffer. */
  get last() {
    return this.buffer[this.buffer.length - 1]
  }

  /**
   * Fully compiles the buffer's data into a {@link Tree | `Tree.build`}
   * compatible format.
   */
  compile() {
    // verbose approach to maximize speed
    const buffer: number[] = []
    const reused: Tree[] = []
    for (let i = this.buffer.length - 1; i >= 0; i--) {
      const node = this.buffer[i]
      if (node instanceof Checkpoint) continue
      const token = node.compile()
      if (!token[4]) {
        buffer.push(token[3], token[2], token[1], token[0])
      } else {
        const [, start, end, size, tree] = token
        reused.push(tree)
        const idx = reused.length - 1
        buffer.push(-1, start + tree.length, start, idx)
        // skip past cached tree
        if (size >= 8) {
          let left = (size - 4) / 4
          while (left !== 0) {
            i--
            if (this.buffer[i] instanceof BufferToken) {
              left--
              // add a filler/repeat node using context hash
              buffer.push(-2, end, start, 0)
            }
          }
        }
      }
    }
    // our buffer is inverted (push+reverse is faster than unshift usually)
    buffer.reverse()
    return { buffer, reused }
  }

  /** Add a new token or context snapshot to the buffer. */
  add(node: TokenData | Context) {
    if (node instanceof Context) {
      const checkpoint = new Checkpoint(node, this.lastCheckpoint)
      this.buffer.push(checkpoint)
      this.lastCheckpoint = checkpoint
      return checkpoint
    } else {
      const token = new BufferToken(node, this.lastCheckpoint)
      this.buffer.push(token)
      return token
    }
  }

  /** Get an element from the buffer. */
  get(index: number) {
    return this.buffer[index]
  }

  /** {@link search} comparator function. */
  private static _searchComparator = ({ pos }: BufferElement, target: number) =>
    pos === target ? true : pos - target

  /** Searches for the closest element to the given position. */
  search(pos: number, opts?: SearchOpts) {
    return search(this.buffer, pos, Buffer._searchComparator, opts)
  }

  /**
   * Finds the closest {@link Context} behind the given `before` value, but
   * after the given `start` value.
   */
  findContext(start: number, before: number) {
    // binary search for closest position to `before`
    let idx = this.search(before, { precise: false })?.index ?? this.buffer.length
    for (; idx >= start; idx--) {
      const node = this.buffer[idx]
      if (node instanceof Checkpoint && node.pos <= before) {
        return { context: node.context, index: idx }
      }
    }
    return null
  }

  /**
   * Cuts the buffer at the given position (or index). All elements after
   * the given position will be removed.
   */
  cut(at: number, indexed = false) {
    let idx: number | undefined
    if (indexed) idx = at
    else idx = this.search(at)?.index

    if (idx) {
      const node = this.buffer[idx]
      if (node instanceof Checkpoint) this.lastCheckpoint = node
      else this.lastCheckpoint = node.checkpoint
      this.buffer = this.buffer.slice(0, idx + (node instanceof BufferToken ? 0 : 1))
      return this
    }

    throw new Error(`Failed to cut precisely! (${at})`)
  }

  /**
   * "Shifts" the buffer to the index, removing all previous nodes from the buffer.
   *
   * @param idx - The index to shift to. Indexed element must be a {@link Checkpoint}.
   */
  shift(idx: number) {
    const checkpoint = this.buffer[idx]
    if (!(checkpoint instanceof Checkpoint)) {
      throw new Error("Must shift to checkpoints!")
    }

    this.buffer = this.buffer.slice(idx)
    checkpoint.prev = undefined
    checkpoint.pos = 0
    this.lastCheckpoint?.update()

    return this
  }

  /**
   * Splits the buffer into two pieces, centered at the provided index.
   * This will mutate the buffer into the left side buffer, while creating
   * a new right side buffer.
   *
   * @param idx - The buffer to split on. Indexed element must be a
   *   {@link Checkpoint}.
   */
  split(idx: number) {
    const rightCheckpoint = this.buffer[idx]
    if (!(rightCheckpoint instanceof Checkpoint)) {
      throw new Error("Must split on checkpoints!")
    }

    const rightLast = this.lastCheckpoint ?? rightCheckpoint

    // excludes the checkpoint at end, so we'll need to add one back
    const left = this.buffer.slice(0, idx)
    const leftCheckpoint = rightCheckpoint.clone()
    left.push(leftCheckpoint)

    const right = this.buffer.slice(idx)

    this.buffer = left
    this.lastCheckpoint = leftCheckpoint

    const rightBuffer = new Buffer(right)
    rightBuffer.lastCheckpoint = rightLast

    rightCheckpoint.prev = undefined
    rightCheckpoint.pos = 0
    rightLast.update()

    return { left: this, right: rightBuffer }
  }

  /**
   * Links another buffer onto the end of this one. The other buffer must
   * begin with a {@link Checkpoint}.
   */
  link(right: Buffer) {
    const rightCheckpoint = right.buffer[0]
    if (rightCheckpoint instanceof BufferToken) {
      throw new Error("Linked buffer must start with a checkpoint!")
    }

    const endPos = this.last.pos

    if (this.last instanceof Checkpoint) this.buffer.pop()
    if (this.last instanceof Checkpoint) {
      throw new Error("Buffer couldn't be linked due to a double checkpoint!")
    }

    rightCheckpoint.prev = this.last.checkpoint
    rightCheckpoint.pos = endPos

    this.buffer = this.buffer.concat(right.buffer)

    this.lastCheckpoint = right.lastCheckpoint
    this.lastCheckpoint?.update()

    return this
  }

  /** Counts the number of checkpoints (context snapshots) in the buffer. */
  countCheckpoints() {
    if (!this.lastCheckpoint) return 0
    let count = 0
    let checkpoint: Checkpoint | undefined = this.lastCheckpoint
    while (checkpoint) {
      count++
      checkpoint = checkpoint.prev
    }
    return count
  }

  /** Returns a copy of this buffer. */
  clone() {
    return new Buffer([...this.buffer])
  }

  // cursor(index?: number) {
  //   return new BufferCursor(this, index)
  // }
}

/*
 * This chunk of code currently won't work with the current Buffer implementation, but
 * it still may prove useful later. Currently, the functionality provided by this class
 * is provided by Buffer's compile() function, but if it ever proves that it's desirable
 * for the Lezer tree builder to use a cursor, rather than a precompiled list, this code
 * can be adapted for that.
 */

// export class BufferCursor {

//   declare private buffer: BufferToken[]
//   declare private index: number
//   declare private token: TokenData

//   constructor(buffer: Buffer | BufferToken[], index?: number) {
//     if (buffer instanceof Buffer) this.buffer = buffer.tokens
//     else this.buffer = buffer
//     this.index = (index ?? this.buffer.length - 1) + 1
//     this.next()
//   }

//   get id()    { return this.token[0] }
//   get start() { return this.token[1] }
//   get end()   { return this.token[2] }
//   get size()  { return this.token[3] }
//   get pos()   { return this.index * 4 }

//   next() {
//     this.index -= 1
//     const node = this.buffer[this.index]
//     if (node) this.token = node.token
//   }

//   fork() {
//     return new BufferCursor(this.buffer, this.index)
//   }
// }

/**
 * {@link WeakMap} wrapper for accessing a cached {@link Buffer} from a
 * CodeMirror syntax tree.
 */
export class BufferCache {
  private map: WeakMap<Tree, Buffer> = new WeakMap()

  /** Associates the given {@link Buffer} to the given tree. */
  attach(buffer: Buffer, tree: Tree) {
    this.map.set(tree, buffer)
    return buffer
  }

  /** Checks if a {@link Buffer} is associated with the given tree. */
  has(tree: Tree) {
    return this.map.has(tree)
  }

  /** Gets the {@link Buffer} associated with the tree, if it exists. */
  get(tree: Tree) {
    return this.map.get(tree)
  }
}

/**
 * `Checkpoint` objects take in a {@link Context} and represent a parsing "snapshot".
 *
 * In a {@link Buffer} all positional data is relative to the previous `Checkpoint`.
 */
export class Checkpoint {
  declare offset: number
  declare prev?: Checkpoint

  private declare last: number
  private declare _tokenizer: SerializedTokenizerStack
  private declare _parser: ParserElementStack
  private declare _embed: SerializedEmbedded

  constructor(context: Context, prev?: Checkpoint) {
    this.prev = prev
    this.last = prev ? prev.pos : 0
    if (context) {
      this.pos = context.pos
      this.tokenizer = context.tokenizer
      this.parser = context.parser
      this.embed = context.embed
    }
  }

  /**
   * Ensures that all positions in the checkpoint chain are fresh by
   * recalculating them.
   */
  update() {
    if (this.prev) {
      // unfortunately, if we just recursively call `update` we'll get errors
      // so we assemble a list instead and work our way up
      const previous = new Set<Checkpoint>([this.prev])

      let current = this.prev
      while (current.prev) {
        if (previous.has(current.prev)) {
          console.error(previous)
          throw new Error("Recursive list!")
        }
        previous.add(current.prev)
        current = current.prev
      }

      for (const checkpoint of [...previous].reverse()) {
        checkpoint.last = checkpoint.prev?.pos ?? 0
      }
      this.last = this.prev.pos
    } else {
      this.last = 0
    }
  }

  clone() {
    // TODO: see about making this more efficient
    const clone = new Checkpoint(this.context, this.prev)
    clone.pos = this.pos
    return clone
  }

  get pos() {
    return this.last + this.offset
  }
  set pos(pos: number) {
    if (!this.prev) this.last = 0
    const offset = pos - this.last
    if (offset < 0) throw new Error(`Bad offset position set (${pos} -> ${offset})`)
    this.offset = offset
  }

  get tokenizer() {
    return new TokenizerStack(this._tokenizer)
  }
  set tokenizer(tokenizer: TokenizerStack) {
    this._tokenizer = tokenizer.serialize()
  }

  get parser() {
    const stack = this._parser.map(element => {
      const clone = [...element]
      clone[1] = this.last + clone[1]
      return clone
    }) as ParserElementStack

    return new ParserStack(stack)
  }
  set parser(parser: ParserStack) {
    const stack = parser.serialize()
    stack.forEach(element => (element[1] -= this.last))
    this._parser = stack
  }

  get embed(): SerializedEmbedded {
    const { pending, parsers } = this._embed
    const pos = this.pos
    return {
      pending: [...pending],
      parsers: parsers.map(([token, { lang, start, end }]) => [
        token,
        { lang, start: start + pos, end: end + pos }
      ])
    }
  }
  set embed(embed: SerializedEmbedded) {
    const { pending, parsers } = embed
    const last = this.last
    this._embed = {
      pending: [...pending],
      parsers: parsers.map(([token, { lang, start, end }]) => [
        token,
        { lang, start: start - last, end: end - last }
      ])
    }
  }

  get context() {
    return new Context(this.pos, this.tokenizer, this.parser, this.embed)
  }
}

/**
 * Represents a parsed token in the {@link Buffer}. Tokens are positionally
 * relative to the previous {@link Checkpoint} object.
 */
export class BufferToken {
  declare checkpoint?: Checkpoint

  private declare token: TokenData
  private declare lastOffset?: number
  private declare lastCompile?: TokenData

  constructor(token: TokenData, checkpoint?: Checkpoint) {
    if (checkpoint) this.checkpoint = checkpoint
    this.token = this.compile(token, -(this.checkpoint?.pos ?? 0))
  }

  get pos() {
    return this.token[1] + (this.checkpoint?.pos ?? 0)
  }

  set tree(tree: Tree | undefined) {
    this.token[4] = tree
    this.lastOffset = -1
  }

  compile(_token = this.token, offset = this.checkpoint?.pos ?? 0) {
    // verbose, fast, and cached approach. this function is executed _very_ often
    if (offset === this.lastOffset && this.lastCompile) return this.lastCompile
    const token: TokenData = [
      _token[0],
      _token[1] + offset,
      _token[2] + offset,
      _token[3]
    ]
    if (_token[4]) token[4] = _token[4]
    this.lastOffset = offset
    this.lastCompile = token
    return token
  }
}
