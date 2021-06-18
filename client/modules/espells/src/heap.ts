export class HeapQueue<T> {
  declare cmp: (a: T, b: T) => number
  declare data: T[]

  constructor(cmp: (a: T, b: T) => number) {
    this.cmp = cmp
    this.data = []
  }

  get length() {
    return this.data.length
  }

  peek() {
    return this.data[0]
  }

  push(val: T) {
    this.data.push(val)

    let pos = this.data.length - 1

    while (pos > 0) {
      const parent = (pos - 1) >>> 1
      if (this.cmp(this.data[pos], this.data[parent]) < 0) {
        const x = this.data[parent]
        this.data[parent] = this.data[pos]
        this.data[pos] = x
        pos = parent
      } else {
        break
      }
    }

    return this.data.length
  }

  pop() {
    const lastValue = this.data.pop()!
    let ret = this.data[0]

    if (this.data.length > 0) {
      this.data[0] = lastValue
      let pos = 0
      let last = this.data.length - 1

      while (true) {
        const left = (pos << 1) + 1
        const right = left + 1

        let minIndex = pos
        if (left <= last && this.cmp(this.data[left], this.data[minIndex]) < 0) {
          minIndex = left
        }
        if (right <= last && this.cmp(this.data[right], this.data[minIndex]) < 0) {
          minIndex = right
        }

        if (minIndex !== pos) {
          const x = this.data[minIndex]
          this.data[minIndex] = this.data[pos]
          this.data[pos] = x
          pos = minIndex
        } else {
          break
        }
      }
    } else {
      ret = lastValue
    }

    return ret
  }

  sort(cmp = this.cmp) {
    return this.data.sort(cmp)
  }
}
